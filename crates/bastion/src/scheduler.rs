use std::str::FromStr;

use chrono::{DateTime, Duration, Utc};
use cron::Schedule;
use sqlx::Row;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tracing::{debug, info, warn};

use crate::jobs_repo::{self, OverlapPolicy};
use crate::runs_repo::{self, RunStatus};

pub fn spawn(db: SqlitePool, run_retention_days: i64) {
    tokio::spawn(run_cron_loop(db.clone()));
    tokio::spawn(run_worker_loop(db.clone()));
    tokio::spawn(run_retention_loop(db, run_retention_days));
}

fn normalize_cron(expr: &str) -> Result<String, anyhow::Error> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    match parts.len() {
        5 => Ok(format!("0 {expr}")),
        6 => Ok(expr.to_string()),
        _ => Err(anyhow::anyhow!("invalid cron expression")),
    }
}

pub fn validate_cron(expr: &str) -> Result<(), anyhow::Error> {
    let expr = normalize_cron(expr)?;
    let _ = Schedule::from_str(&expr)?;
    Ok(())
}

fn cron_matches_minute(expr: &str, minute_start: DateTime<Utc>) -> Result<bool, anyhow::Error> {
    let expr = normalize_cron(expr)?;
    let schedule = Schedule::from_str(&expr)?;

    let prev = minute_start - Duration::seconds(1);
    let mut iter = schedule.after(&prev);
    let Some(next) = iter.next() else {
        return Ok(false);
    };

    Ok(next == minute_start)
}

async fn enqueue_run(db: &SqlitePool, job: &jobs_repo::Job, source: &str) -> anyhow::Result<()> {
    let running_count = sqlx::query(
        "SELECT COUNT(1) AS n FROM runs WHERE job_id = ? AND status IN ('running', 'queued')",
    )
    .bind(&job.id)
    .fetch_one(db)
    .await?
    .get::<i64, _>("n");

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let (status, ended_at, error) =
        if job.overlap_policy == OverlapPolicy::Reject && running_count > 0 {
            (RunStatus::Rejected, Some(now), Some("overlap_rejected"))
        } else {
            (RunStatus::Queued, None, None)
        };

    let run = runs_repo::create_run(db, &job.id, status, now, ended_at, None, error).await?;
    runs_repo::append_run_event(
        db,
        &run.id,
        "info",
        status.as_str(),
        status.as_str(),
        Some(serde_json::json!({ "source": source })),
    )
    .await?;

    Ok(())
}

async fn run_cron_loop(db: SqlitePool) {
    let mut last_minute = OffsetDateTime::now_utc().unix_timestamp() / 60 - 1;

    loop {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let minute = now / 60;
        if minute != last_minute {
            last_minute = minute;

            let minute_start = match DateTime::<Utc>::from_timestamp(minute * 60, 0) {
                Some(ts) => ts,
                None => {
                    warn!("invalid timestamp for scheduler minute_start");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            let jobs = match jobs_repo::list_jobs(&db).await {
                Ok(v) => v,
                Err(error) => {
                    warn!(error = %error, "failed to list jobs for scheduler");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    continue;
                }
            };

            for job in jobs {
                let Some(schedule) = job.schedule.as_deref() else {
                    continue;
                };

                match cron_matches_minute(schedule, minute_start) {
                    Ok(true) => {
                        debug!(job_id = %job.id, "cron due; enqueue run");
                        if let Err(error) = enqueue_run(&db, &job, "schedule").await {
                            warn!(job_id = %job.id, error = %error, "failed to enqueue scheduled run");
                        }
                    }
                    Ok(false) => {}
                    Err(error) => {
                        warn!(job_id = %job.id, error = %error, "invalid cron schedule; skipping");
                    }
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn run_worker_loop(db: SqlitePool) {
    loop {
        let run = match runs_repo::claim_next_queued_run(&db).await {
            Ok(v) => v,
            Err(error) => {
                warn!(error = %error, "failed to claim queued run");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
        };

        let Some(run) = run else {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        };

        info!(run_id = %run.id, job_id = %run.job_id, "run started (noop executor)");

        if let Err(error) = runs_repo::append_run_event(
            &db,
            &run.id,
            "info",
            "start",
            "start",
            Some(serde_json::json!({ "executor": "noop" })),
        )
        .await
        {
            warn!(run_id = %run.id, error = %error, "failed to write start event");
        }

        let job = match jobs_repo::get_job(&db, &run.job_id).await {
            Ok(Some(job)) => job,
            Ok(None) => {
                let _ = runs_repo::complete_run(
                    &db,
                    &run.id,
                    RunStatus::Failed,
                    None,
                    Some("job_not_found"),
                )
                .await;
                continue;
            }
            Err(error) => {
                warn!(run_id = %run.id, error = %error, "failed to load job");
                let _ = runs_repo::complete_run(
                    &db,
                    &run.id,
                    RunStatus::Failed,
                    None,
                    Some("job_load_failed"),
                )
                .await;
                continue;
            }
        };

        let spec_type = job
            .spec
            .as_object()
            .and_then(|o| o.get("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let _ = runs_repo::append_run_event(
            &db,
            &run.id,
            "info",
            "noop",
            "noop",
            Some(serde_json::json!({ "job_type": spec_type })),
        )
        .await;

        if let Err(error) =
            runs_repo::complete_run(&db, &run.id, RunStatus::Success, None, None).await
        {
            warn!(run_id = %run.id, error = %error, "failed to complete run");
            continue;
        }

        let _ =
            runs_repo::append_run_event(&db, &run.id, "info", "complete", "complete", None).await;
        info!(run_id = %run.id, "run completed");
    }
}

async fn run_retention_loop(db: SqlitePool, run_retention_days: i64) {
    loop {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let cutoff = now.saturating_sub(run_retention_days.saturating_mul(24 * 60 * 60));

        match runs_repo::prune_runs_ended_before(&db, cutoff).await {
            Ok(pruned) => {
                if pruned > 0 {
                    info!(pruned, run_retention_days, "pruned old runs");
                }
            }
            Err(error) => {
                warn!(error = %error, "failed to prune old runs");
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await;
    }
}
