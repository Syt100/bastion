use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use cron::Schedule;
use sqlx::Row;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use crate::agent_manager::AgentManager;
use crate::agent_protocol::{
    BackupRunTaskV1, EncryptionResolvedV1, HubToAgentMessageV1, JobSpecResolvedV1,
    PROTOCOL_VERSION, PipelineResolvedV1, TargetResolvedV1,
};
use crate::agent_tasks_repo;
use crate::backup;
use crate::backup_encryption;
use crate::job_spec;
use crate::jobs_repo::{self, OverlapPolicy};
use crate::notifications_repo;
use crate::run_events;
use crate::run_events_bus::RunEventsBus;
use crate::run_failure::RunFailedWithSummary;
use crate::runs_repo::{self, RunStatus};
use crate::secrets::SecretsCrypto;
use crate::secrets_repo;
use crate::targets;
use crate::webdav::WebdavCredentials;
use url::Url;

pub fn spawn(
    db: SqlitePool,
    data_dir: std::path::PathBuf,
    secrets: Arc<SecretsCrypto>,
    agent_manager: AgentManager,
    run_retention_days: i64,
    incomplete_cleanup_days: i64,
    run_events_bus: Arc<RunEventsBus>,
    run_queue_notify: Arc<Notify>,
    shutdown: CancellationToken,
) {
    tokio::spawn(run_cron_loop(
        db.clone(),
        run_events_bus.clone(),
        run_queue_notify.clone(),
        shutdown.clone(),
    ));
    tokio::spawn(run_worker_loop(
        db.clone(),
        data_dir,
        secrets.clone(),
        agent_manager,
        run_events_bus.clone(),
        run_queue_notify.clone(),
        shutdown.clone(),
    ));
    tokio::spawn(run_retention_loop(
        db.clone(),
        run_retention_days,
        shutdown.clone(),
    ));
    if incomplete_cleanup_days > 0 {
        tokio::spawn(run_incomplete_cleanup_loop(
            db.clone(),
            secrets,
            incomplete_cleanup_days,
            shutdown,
        ));
    }
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

async fn enqueue_run(
    db: &SqlitePool,
    run_events_bus: &RunEventsBus,
    run_queue_notify: &Notify,
    job: &jobs_repo::Job,
    source: &str,
) -> anyhow::Result<()> {
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
    run_events::append_and_broadcast(
        db,
        run_events_bus,
        &run.id,
        "info",
        status.as_str(),
        status.as_str(),
        Some(serde_json::json!({ "source": source })),
    )
    .await?;

    if status == RunStatus::Queued {
        run_queue_notify.notify_one();
    }

    Ok(())
}

async fn run_cron_loop(
    db: SqlitePool,
    run_events_bus: Arc<RunEventsBus>,
    run_queue_notify: Arc<Notify>,
    shutdown: CancellationToken,
) {
    let mut last_minute = OffsetDateTime::now_utc().unix_timestamp() / 60 - 1;

    loop {
        if shutdown.is_cancelled() {
            break;
        }

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
                        if let Err(error) = enqueue_run(
                            &db,
                            run_events_bus.as_ref(),
                            run_queue_notify.as_ref(),
                            &job,
                            "schedule",
                        )
                        .await
                        {
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

        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
        }
    }
}

async fn run_worker_loop(
    db: SqlitePool,
    data_dir: std::path::PathBuf,
    secrets: Arc<SecretsCrypto>,
    agent_manager: AgentManager,
    run_events_bus: Arc<RunEventsBus>,
    run_queue_notify: Arc<Notify>,
    shutdown: CancellationToken,
) {
    loop {
        if shutdown.is_cancelled() {
            break;
        }

        let run = match runs_repo::claim_next_queued_run(&db).await {
            Ok(v) => v,
            Err(error) => {
                warn!(error = %error, "failed to claim queued run");
                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    _ = run_queue_notify.notified() => {}
                    _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {}
                }
                continue;
            }
        };

        let Some(run) = run else {
            tokio::select! {
                _ = shutdown.cancelled() => break,
                _ = run_queue_notify.notified() => {}
                _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {}
            }
            continue;
        };

        info!(run_id = %run.id, job_id = %run.job_id, "run started");

        if let Err(error) = run_events::append_and_broadcast(
            &db,
            &run_events_bus,
            &run.id,
            "info",
            "start",
            "start",
            None,
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

        let spec = match job_spec::parse_value(&job.spec) {
            Ok(v) => v,
            Err(error) => {
                let message = format!("invalid spec: {error}");
                let _ = run_events::append_and_broadcast(
                    &db,
                    &run_events_bus,
                    &run.id,
                    "error",
                    "invalid_spec",
                    &message,
                    None,
                )
                .await;
                let _ = runs_repo::complete_run(
                    &db,
                    &run.id,
                    RunStatus::Failed,
                    None,
                    Some("invalid_spec"),
                )
                .await;
                continue;
            }
        };

        if let Err(error) = job_spec::validate(&spec) {
            let message = format!("invalid spec: {error}");
            let _ = run_events::append_and_broadcast(
                &db,
                &run_events_bus,
                &run.id,
                "error",
                "invalid_spec",
                &message,
                None,
            )
            .await;
            let _ = runs_repo::complete_run(
                &db,
                &run.id,
                RunStatus::Failed,
                None,
                Some("invalid_spec"),
            )
            .await;
            continue;
        }

        let started_at = OffsetDateTime::from_unix_timestamp(run.started_at)
            .unwrap_or_else(|_| OffsetDateTime::now_utc());

        if let Some(agent_id) = job.agent_id.as_deref() {
            if let Err(error) = dispatch_run_to_agent(
                &db,
                &secrets,
                &agent_manager,
                run_events_bus.as_ref(),
                &job,
                &run.id,
                started_at,
                spec,
                agent_id,
            )
            .await
            {
                warn!(run_id = %run.id, agent_id = %agent_id, error = %error, "dispatch failed");
                let message = format!("dispatch failed: {error}");
                let _ = run_events::append_and_broadcast(
                    &db,
                    &run_events_bus,
                    &run.id,
                    "error",
                    "dispatch_failed",
                    &message,
                    Some(serde_json::json!({ "agent_id": agent_id })),
                )
                .await;

                let _ = runs_repo::requeue_run(&db, &run.id).await;
                let _ = agent_tasks_repo::delete_task(&db, &run.id).await;
                run_queue_notify.notify_one();
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }

            // Wait for Agent to complete the run (single-worker, no parallel runs).
            let deadline = OffsetDateTime::now_utc()
                .checked_add(time::Duration::hours(24))
                .unwrap_or_else(OffsetDateTime::now_utc);
            loop {
                let Some(current) = runs_repo::get_run(&db, &run.id).await.unwrap_or(None) else {
                    break;
                };
                if current.status != RunStatus::Running {
                    if let Err(error) =
                        notifications_repo::enqueue_wecom_bots_for_run(&db, &run.id).await
                    {
                        warn!(run_id = %run.id, error = %error, "failed to enqueue wecom notifications");
                    }
                    if let Err(error) =
                        notifications_repo::enqueue_emails_for_run(&db, &run.id).await
                    {
                        warn!(run_id = %run.id, error = %error, "failed to enqueue email notifications");
                    }
                    info!(run_id = %run.id, "run completed (agent)");
                    break;
                }

                if OffsetDateTime::now_utc() >= deadline {
                    warn!(run_id = %run.id, agent_id = %agent_id, "agent run timed out");
                    let _ = run_events::append_and_broadcast(
                        &db,
                        &run_events_bus,
                        &run.id,
                        "error",
                        "timeout",
                        "timeout",
                        Some(serde_json::json!({ "agent_id": agent_id })),
                    )
                    .await;
                    let _ = runs_repo::complete_run(
                        &db,
                        &run.id,
                        RunStatus::Failed,
                        None,
                        Some("timeout"),
                    )
                    .await;
                    break;
                }

                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {}
                }
            }

            continue;
        }

        match execute_run(
            &db,
            &secrets,
            run_events_bus.as_ref(),
            &data_dir,
            &job,
            &run.id,
            started_at,
            spec,
        )
        .await
        {
            Ok(summary) => {
                if let Err(error) =
                    runs_repo::complete_run(&db, &run.id, RunStatus::Success, Some(summary), None)
                        .await
                {
                    warn!(run_id = %run.id, error = %error, "failed to complete run");
                    continue;
                }
                let _ = run_events::append_and_broadcast(
                    &db,
                    &run_events_bus,
                    &run.id,
                    "info",
                    "complete",
                    "complete",
                    None,
                )
                .await;
                if let Err(error) =
                    notifications_repo::enqueue_wecom_bots_for_run(&db, &run.id).await
                {
                    warn!(run_id = %run.id, error = %error, "failed to enqueue wecom notifications");
                }
                if let Err(error) = notifications_repo::enqueue_emails_for_run(&db, &run.id).await {
                    warn!(run_id = %run.id, error = %error, "failed to enqueue email notifications");
                }
                info!(run_id = %run.id, "run completed");
            }
            Err(error) => {
                warn!(run_id = %run.id, error = %error, "run failed");
                let message = format!("failed: {error}");
                let _ = run_events::append_and_broadcast(
                    &db,
                    &run_events_bus,
                    &run.id,
                    "error",
                    "failed",
                    &message,
                    None,
                )
                .await;

                let soft = error.downcast_ref::<RunFailedWithSummary>();
                let summary = soft.map(|e| e.summary.clone());
                let error_code = soft.map(|e| e.code).unwrap_or("run_failed");

                let _ = runs_repo::complete_run(
                    &db,
                    &run.id,
                    RunStatus::Failed,
                    summary,
                    Some(error_code),
                )
                .await;
                if let Err(error) =
                    notifications_repo::enqueue_wecom_bots_for_run(&db, &run.id).await
                {
                    warn!(run_id = %run.id, error = %error, "failed to enqueue wecom notifications");
                }
                if let Err(error) = notifications_repo::enqueue_emails_for_run(&db, &run.id).await {
                    warn!(run_id = %run.id, error = %error, "failed to enqueue email notifications");
                }
            }
        }
    }
}

async fn dispatch_run_to_agent(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    run_events_bus: &RunEventsBus,
    job: &jobs_repo::Job,
    run_id: &str,
    started_at: OffsetDateTime,
    spec: job_spec::JobSpecV1,
    agent_id: &str,
) -> Result<(), anyhow::Error> {
    if !agent_manager.is_connected(agent_id).await {
        anyhow::bail!("agent not connected");
    }

    run_events::append_and_broadcast(
        db,
        run_events_bus,
        run_id,
        "info",
        "dispatch",
        "dispatch",
        Some(serde_json::json!({ "agent_id": agent_id })),
    )
    .await?;

    let resolved = resolve_job_spec_for_agent(db, secrets, spec).await?;
    let task = BackupRunTaskV1 {
        run_id: run_id.to_string(),
        job_id: job.id.clone(),
        started_at: started_at.unix_timestamp(),
        spec: resolved,
    };

    // Use run_id as task_id for idempotency.
    let msg = HubToAgentMessageV1::Task {
        v: PROTOCOL_VERSION,
        task_id: run_id.to_string(),
        task,
    };

    let payload = serde_json::to_value(&msg)?;
    agent_tasks_repo::upsert_task(db, run_id, agent_id, run_id, "sent", &payload).await?;

    agent_manager.send_json(agent_id, &msg).await?;
    Ok(())
}

async fn resolve_job_spec_for_agent(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    spec: job_spec::JobSpecV1,
) -> Result<JobSpecResolvedV1, anyhow::Error> {
    match spec {
        job_spec::JobSpecV1::Filesystem {
            v,
            pipeline,
            source,
            target,
        } => Ok(JobSpecResolvedV1::Filesystem {
            v,
            pipeline: resolve_pipeline_for_agent(db, secrets, &pipeline).await?,
            source,
            target: resolve_target_for_agent(db, secrets, target).await?,
        }),
        job_spec::JobSpecV1::Sqlite {
            v,
            pipeline,
            source,
            target,
        } => Ok(JobSpecResolvedV1::Sqlite {
            v,
            pipeline: resolve_pipeline_for_agent(db, secrets, &pipeline).await?,
            source,
            target: resolve_target_for_agent(db, secrets, target).await?,
        }),
        job_spec::JobSpecV1::Vaultwarden {
            v,
            pipeline,
            source,
            target,
        } => Ok(JobSpecResolvedV1::Vaultwarden {
            v,
            pipeline: resolve_pipeline_for_agent(db, secrets, &pipeline).await?,
            source,
            target: resolve_target_for_agent(db, secrets, target).await?,
        }),
    }
}

async fn resolve_pipeline_for_agent(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    pipeline: &job_spec::PipelineV1,
) -> Result<PipelineResolvedV1, anyhow::Error> {
    let encryption = backup_encryption::ensure_payload_encryption(db, secrets, pipeline).await?;
    let encryption = match encryption {
        backup::PayloadEncryption::None => EncryptionResolvedV1::None,
        backup::PayloadEncryption::AgeX25519 {
            recipient,
            key_name,
        } => EncryptionResolvedV1::AgeX25519 {
            recipient,
            key_name,
        },
    };
    Ok(PipelineResolvedV1 { encryption })
}

async fn resolve_target_for_agent(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    target: job_spec::TargetV1,
) -> Result<TargetResolvedV1, anyhow::Error> {
    match target {
        job_spec::TargetV1::Webdav {
            base_url,
            secret_name,
            part_size_bytes,
        } => {
            let cred_bytes = secrets_repo::get_secret(db, secrets, "webdav", &secret_name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
            let credentials = WebdavCredentials::from_json(&cred_bytes)?;
            Ok(TargetResolvedV1::Webdav {
                base_url,
                username: credentials.username,
                password: credentials.password,
                part_size_bytes,
            })
        }
        job_spec::TargetV1::LocalDir {
            base_dir,
            part_size_bytes,
        } => Ok(TargetResolvedV1::LocalDir {
            base_dir,
            part_size_bytes,
        }),
    }
}

async fn execute_run(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    run_events_bus: &RunEventsBus,
    data_dir: &std::path::Path,
    job: &jobs_repo::Job,
    run_id: &str,
    started_at: OffsetDateTime,
    spec: job_spec::JobSpecV1,
) -> Result<serde_json::Value, anyhow::Error> {
    match spec {
        job_spec::JobSpecV1::Filesystem {
            pipeline,
            source,
            target,
            ..
        } => {
            run_events::append_and_broadcast(
                db,
                run_events_bus,
                run_id,
                "info",
                "packaging",
                "packaging",
                None,
            )
            .await?;

            let data_dir = data_dir.to_path_buf();
            let job_id = job.id.clone();
            let run_id_owned = run_id.to_string();
            let part_size = target.part_size_bytes();
            let error_policy = source.error_policy;
            let encryption =
                backup_encryption::ensure_payload_encryption(db, secrets, &pipeline).await?;
            let artifacts = tokio::task::spawn_blocking(move || {
                backup::filesystem::build_filesystem_run(
                    &data_dir,
                    &job_id,
                    &run_id_owned,
                    started_at,
                    &source,
                    &encryption,
                    part_size,
                )
            })
            .await??;

            if artifacts.issues.warnings_total > 0 || artifacts.issues.errors_total > 0 {
                let level = if artifacts.issues.errors_total > 0 {
                    "error"
                } else {
                    "warn"
                };
                let fields = serde_json::json!({
                    "warnings_total": artifacts.issues.warnings_total,
                    "errors_total": artifacts.issues.errors_total,
                    "sample_warnings": &artifacts.issues.sample_warnings,
                    "sample_errors": &artifacts.issues.sample_errors,
                });
                let _ = run_events::append_and_broadcast(
                    db,
                    run_events_bus,
                    run_id,
                    level,
                    "fs_issues",
                    "filesystem issues",
                    Some(fields),
                )
                .await;
            }

            let issues = artifacts.issues;
            let artifacts = artifacts.artifacts;

            run_events::append_and_broadcast(
                db,
                run_events_bus,
                run_id,
                "info",
                "upload",
                "upload",
                None,
            )
            .await?;
            let target_summary =
                store_run_artifacts_to_target(db, secrets, &job.id, run_id, &target, &artifacts)
                    .await?;

            let _ = tokio::fs::remove_dir_all(&artifacts.run_dir).await;

            let summary = serde_json::json!({
                "target": target_summary,
                "entries_count": artifacts.entries_count,
                "parts": artifacts.parts.len(),
                "filesystem": {
                    "warnings_total": issues.warnings_total,
                    "errors_total": issues.errors_total,
                },
            });

            if error_policy == job_spec::FsErrorPolicy::SkipFail && issues.errors_total > 0 {
                return Err(anyhow::Error::new(RunFailedWithSummary::new(
                    "fs_issues",
                    format!(
                        "filesystem backup completed with {} errors",
                        issues.errors_total
                    ),
                    summary,
                )));
            }

            Ok(summary)
        }
        job_spec::JobSpecV1::Sqlite {
            pipeline,
            source,
            target,
            ..
        } => {
            run_events::append_and_broadcast(
                db,
                run_events_bus,
                run_id,
                "info",
                "snapshot",
                "snapshot",
                None,
            )
            .await?;

            let sqlite_path = source.path.clone();
            let data_dir = data_dir.to_path_buf();
            let job_id = job.id.clone();
            let run_id_owned = run_id.to_string();
            let part_size = target.part_size_bytes();
            let encryption =
                backup_encryption::ensure_payload_encryption(db, secrets, &pipeline).await?;
            let build = tokio::task::spawn_blocking(move || {
                backup::sqlite::build_sqlite_run(
                    &data_dir,
                    &job_id,
                    &run_id_owned,
                    started_at,
                    &source,
                    &encryption,
                    part_size,
                )
            })
            .await??;

            if let Some(check) = build.integrity_check.as_ref() {
                let data = serde_json::json!({
                    "ok": check.ok,
                    "truncated": check.truncated,
                    "lines": check.lines,
                });
                let _ = run_events::append_and_broadcast(
                    db,
                    run_events_bus,
                    run_id,
                    if check.ok { "info" } else { "error" },
                    "integrity_check",
                    "integrity_check",
                    Some(data),
                )
                .await;

                if !check.ok {
                    let first = check.lines.first().cloned().unwrap_or_default();
                    anyhow::bail!("sqlite integrity_check failed: {}", first);
                }
            }

            run_events::append_and_broadcast(
                db,
                run_events_bus,
                run_id,
                "info",
                "upload",
                "upload",
                None,
            )
            .await?;
            let target_summary = store_run_artifacts_to_target(
                db,
                secrets,
                &job.id,
                run_id,
                &target,
                &build.artifacts,
            )
            .await?;

            let _ = tokio::fs::remove_dir_all(&build.artifacts.run_dir).await;

            Ok(serde_json::json!({
                "target": target_summary,
                "entries_count": build.artifacts.entries_count,
                "parts": build.artifacts.parts.len(),
                "sqlite": {
                    "path": sqlite_path,
                    "snapshot_name": build.snapshot_name,
                    "snapshot_size": build.snapshot_size,
                    "integrity_check": build.integrity_check.map(|check| serde_json::json!({
                        "ok": check.ok,
                        "truncated": check.truncated,
                        "lines": check.lines,
                    })),
                }
            }))
        }
        job_spec::JobSpecV1::Vaultwarden {
            pipeline,
            source,
            target,
            ..
        } => {
            run_events::append_and_broadcast(
                db,
                run_events_bus,
                run_id,
                "info",
                "snapshot",
                "snapshot",
                None,
            )
            .await?;

            let data_dir = data_dir.to_path_buf();
            let job_id = job.id.clone();
            let run_id_owned = run_id.to_string();
            let part_size = target.part_size_bytes();
            let vw_data_dir = source.data_dir.clone();
            let encryption =
                backup_encryption::ensure_payload_encryption(db, secrets, &pipeline).await?;
            let artifacts = tokio::task::spawn_blocking(move || {
                backup::vaultwarden::build_vaultwarden_run(
                    &data_dir,
                    &job_id,
                    &run_id_owned,
                    started_at,
                    &source,
                    &encryption,
                    part_size,
                )
            })
            .await??;

            run_events::append_and_broadcast(
                db,
                run_events_bus,
                run_id,
                "info",
                "upload",
                "upload",
                None,
            )
            .await?;
            let target_summary =
                store_run_artifacts_to_target(db, secrets, &job.id, run_id, &target, &artifacts)
                    .await?;

            let _ = tokio::fs::remove_dir_all(&artifacts.run_dir).await;

            Ok(serde_json::json!({
                "target": target_summary,
                "entries_count": artifacts.entries_count,
                "parts": artifacts.parts.len(),
                "vaultwarden": {
                    "data_dir": vw_data_dir,
                    "db": "db.sqlite3",
                }
            }))
        }
    }
}

async fn store_run_artifacts_to_target(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    job_id: &str,
    run_id: &str,
    target: &job_spec::TargetV1,
    artifacts: &backup::LocalRunArtifacts,
) -> Result<serde_json::Value, anyhow::Error> {
    match target {
        job_spec::TargetV1::Webdav {
            base_url,
            secret_name,
            ..
        } => {
            let cred_bytes = secrets_repo::get_secret(db, secrets, "webdav", secret_name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
            let credentials = WebdavCredentials::from_json(&cred_bytes)?;

            let run_url =
                targets::webdav::store_run(base_url, credentials, job_id, run_id, artifacts)
                    .await?;
            Ok(serde_json::json!({ "type": "webdav", "run_url": run_url.as_str() }))
        }
        job_spec::TargetV1::LocalDir { base_dir, .. } => {
            let base_dir = base_dir.to_string();
            let job_id = job_id.to_string();
            let run_id = run_id.to_string();
            let artifacts = artifacts.clone();
            let run_dir = tokio::task::spawn_blocking(move || {
                targets::local_dir::store_run(
                    std::path::Path::new(&base_dir),
                    &job_id,
                    &run_id,
                    &artifacts,
                )
            })
            .await??;
            Ok(serde_json::json!({
                "type": "local_dir",
                "run_dir": run_dir.to_string_lossy().to_string()
            }))
        }
    }
}

async fn run_retention_loop(db: SqlitePool, run_retention_days: i64, shutdown: CancellationToken) {
    loop {
        if shutdown.is_cancelled() {
            break;
        }

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

        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = tokio::time::sleep(std::time::Duration::from_secs(60 * 60)) => {}
        }
    }
}

async fn run_incomplete_cleanup_loop(
    db: SqlitePool,
    secrets: Arc<SecretsCrypto>,
    incomplete_cleanup_days: i64,
    shutdown: CancellationToken,
) {
    let cutoff_seconds = incomplete_cleanup_days.saturating_mul(24 * 60 * 60);
    if cutoff_seconds <= 0 {
        return;
    }

    loop {
        if shutdown.is_cancelled() {
            break;
        }

        let now = OffsetDateTime::now_utc().unix_timestamp();
        let cutoff_started_at = now.saturating_sub(cutoff_seconds);

        let mut deleted = 0_u64;
        loop {
            let candidates =
                match runs_repo::list_incomplete_cleanup_candidates(&db, cutoff_started_at, 100)
                    .await
                {
                    Ok(v) => v,
                    Err(error) => {
                        warn!(error = %error, "failed to list incomplete cleanup candidates");
                        break;
                    }
                };
            if candidates.is_empty() {
                break;
            }

            for run in candidates {
                debug!(
                    run_id = %run.id,
                    job_id = %run.job_id,
                    status = ?run.status,
                    started_at = run.started_at,
                    "incomplete cleanup candidate"
                );

                let Some(job) = jobs_repo::get_job(&db, &run.job_id).await.unwrap_or(None) else {
                    continue;
                };
                let spec = match job_spec::parse_value(&job.spec) {
                    Ok(v) => v,
                    Err(error) => {
                        warn!(job_id = %job.id, run_id = %run.id, error = %error, "invalid job spec while cleaning up incomplete run");
                        continue;
                    }
                };

                match extract_target(&spec) {
                    job_spec::TargetV1::LocalDir { base_dir, .. } => {
                        let base_dir = base_dir.clone();
                        let job_id = job.id.clone();
                        let run_id = run.id.clone();
                        let removed = tokio::task::spawn_blocking(move || {
                            cleanup_local_dir_run(&base_dir, &job_id, &run_id)
                        })
                        .await
                        .unwrap_or(Ok(false))
                        .unwrap_or(false);
                        if removed {
                            deleted = deleted.saturating_add(1);
                        }
                    }
                    job_spec::TargetV1::Webdav {
                        base_url,
                        secret_name,
                        ..
                    } => {
                        let removed = match cleanup_webdav_run(
                            &db,
                            &secrets,
                            base_url,
                            secret_name,
                            &job.id,
                            &run.id,
                        )
                        .await
                        {
                            Ok(v) => v,
                            Err(error) => {
                                warn!(job_id = %job.id, run_id = %run.id, error = %error, "failed to cleanup stale webdav run");
                                false
                            }
                        };
                        if removed {
                            deleted = deleted.saturating_add(1);
                        }
                    }
                }
            }
        }

        if deleted > 0 {
            info!(
                deleted,
                incomplete_cleanup_days, "cleaned up stale incomplete target runs"
            );
        }

        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = tokio::time::sleep(std::time::Duration::from_secs(60 * 60)) => {}
        }
    }
}

fn extract_target(spec: &job_spec::JobSpecV1) -> &job_spec::TargetV1 {
    match spec {
        job_spec::JobSpecV1::Filesystem { target, .. } => target,
        job_spec::JobSpecV1::Sqlite { target, .. } => target,
        job_spec::JobSpecV1::Vaultwarden { target, .. } => target,
    }
}

fn cleanup_local_dir_run(
    base_dir: &str,
    job_id: &str,
    run_id: &str,
) -> Result<bool, anyhow::Error> {
    use crate::backup::{COMPLETE_NAME, ENTRIES_INDEX_NAME, MANIFEST_NAME};

    let run_dir = std::path::Path::new(base_dir).join(job_id).join(run_id);
    if !run_dir.exists() {
        return Ok(false);
    }
    if run_dir.join(COMPLETE_NAME).exists() {
        return Ok(false);
    }

    let mut looks_like_bastion = false;
    if run_dir.join(MANIFEST_NAME).exists() || run_dir.join(ENTRIES_INDEX_NAME).exists() {
        looks_like_bastion = true;
    } else if let Ok(entries) = std::fs::read_dir(&run_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("payload.part") || name.ends_with(".partial") {
                looks_like_bastion = true;
                break;
            }
        }
    }
    if !looks_like_bastion {
        return Ok(false);
    }

    std::fs::remove_dir_all(&run_dir)?;
    Ok(true)
}

async fn cleanup_webdav_run(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    base_url: &str,
    secret_name: &str,
    job_id: &str,
    run_id: &str,
) -> Result<bool, anyhow::Error> {
    use crate::backup::COMPLETE_NAME;

    let cred_bytes = secrets_repo::get_secret(db, secrets, "webdav", secret_name)
        .await?
        .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
    let credentials = WebdavCredentials::from_json(&cred_bytes)?;

    let mut base_url = Url::parse(base_url)?;
    if !base_url.path().ends_with('/') {
        base_url.set_path(&format!("{}/", base_url.path()));
    }

    let client = crate::webdav::WebdavClient::new(base_url.clone(), credentials)?;
    let job_url = base_url.join(&format!("{job_id}/"))?;
    let run_url = job_url.join(&format!("{run_id}/"))?;
    let complete_url = run_url.join(COMPLETE_NAME)?;
    if client.head_size(&complete_url).await?.is_some() {
        return Ok(false);
    }

    Ok(client.delete(&run_url).await?)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;
    use crate::jobs_repo::{self, OverlapPolicy};
    use crate::run_events_bus::RunEventsBus;
    use crate::runs_repo::{self, RunStatus};
    use tokio::sync::Notify;

    use super::enqueue_run;

    #[tokio::test]
    async fn overlap_policy_reject_inserts_rejected_run() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        let job = jobs_repo::create_job(
            &pool,
            "job1",
            None,
            None,
            OverlapPolicy::Reject,
            serde_json::json!({
                "v": 1,
                "type": "filesystem",
                "source": { "root": "/" },
                "target": { "type": "local_dir", "base_dir": "/tmp" }
            }),
        )
        .await
        .expect("create job");

        // Existing running run triggers rejection.
        let _existing =
            runs_repo::create_run(&pool, &job.id, RunStatus::Running, 1, None, None, None)
                .await
                .expect("existing run");

        let bus = RunEventsBus::new_with_options(8, 60, 1);
        let notify = Notify::new();
        enqueue_run(&pool, &bus, &notify, &job, "cron")
            .await
            .expect("enqueue");

        let runs = runs_repo::list_runs_for_job(&pool, &job.id, 10)
            .await
            .expect("list runs");
        let newest = &runs[0];
        assert_eq!(newest.status, RunStatus::Rejected);
        assert!(newest.ended_at.is_some());
        assert_eq!(newest.error.as_deref(), Some("overlap_rejected"));
    }

    #[tokio::test]
    async fn overlap_policy_queue_inserts_queued_run() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        let job = jobs_repo::create_job(
            &pool,
            "job1",
            None,
            None,
            OverlapPolicy::Queue,
            serde_json::json!({
                "v": 1,
                "type": "filesystem",
                "source": { "root": "/" },
                "target": { "type": "local_dir", "base_dir": "/tmp" }
            }),
        )
        .await
        .expect("create job");

        let _existing =
            runs_repo::create_run(&pool, &job.id, RunStatus::Running, 1, None, None, None)
                .await
                .expect("existing run");

        let bus = RunEventsBus::new_with_options(8, 60, 1);
        let notify = Notify::new();
        enqueue_run(&pool, &bus, &notify, &job, "cron")
            .await
            .expect("enqueue");

        let runs = runs_repo::list_runs_for_job(&pool, &job.id, 10)
            .await
            .expect("list runs");
        let newest = &runs[0];
        assert_eq!(newest.status, RunStatus::Queued);
        assert!(newest.ended_at.is_none());
        assert!(newest.error.is_none());
    }
}
