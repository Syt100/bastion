use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use cron::Schedule;
use sqlx::Row;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tracing::{debug, info, warn};
use url::Url;

use crate::backup;
use crate::job_spec;
use crate::jobs_repo::{self, OverlapPolicy};
use crate::notifications_repo;
use crate::runs_repo::{self, RunStatus};
use crate::secrets::SecretsCrypto;
use crate::secrets_repo;
use crate::targets;
use crate::webdav::{WebdavClient, WebdavCredentials};

pub fn spawn(
    db: SqlitePool,
    data_dir: std::path::PathBuf,
    secrets: Arc<SecretsCrypto>,
    run_retention_days: i64,
) {
    tokio::spawn(run_cron_loop(db.clone()));
    tokio::spawn(run_worker_loop(db.clone(), data_dir, secrets));
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

async fn run_worker_loop(
    db: SqlitePool,
    data_dir: std::path::PathBuf,
    secrets: Arc<SecretsCrypto>,
) {
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

        info!(run_id = %run.id, job_id = %run.job_id, "run started");

        if let Err(error) =
            runs_repo::append_run_event(&db, &run.id, "info", "start", "start", None).await
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
                let _ = runs_repo::append_run_event(
                    &db,
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
            let _ =
                runs_repo::append_run_event(&db, &run.id, "error", "invalid_spec", &message, None)
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

        match execute_run(&db, &secrets, &data_dir, &job, &run.id, started_at, spec).await {
            Ok(summary) => {
                if let Err(error) =
                    runs_repo::complete_run(&db, &run.id, RunStatus::Success, Some(summary), None)
                        .await
                {
                    warn!(run_id = %run.id, error = %error, "failed to complete run");
                    continue;
                }
                let _ =
                    runs_repo::append_run_event(&db, &run.id, "info", "complete", "complete", None)
                        .await;
                if let Err(error) =
                    notifications_repo::enqueue_wecom_bots_for_run(&db, &run.id).await
                {
                    warn!(run_id = %run.id, error = %error, "failed to enqueue notifications");
                }
                info!(run_id = %run.id, "run completed");
            }
            Err(error) => {
                warn!(run_id = %run.id, error = %error, "run failed");
                let message = format!("failed: {error}");
                let _ =
                    runs_repo::append_run_event(&db, &run.id, "error", "failed", &message, None)
                        .await;

                let _ = runs_repo::complete_run(
                    &db,
                    &run.id,
                    RunStatus::Failed,
                    None,
                    Some("run_failed"),
                )
                .await;
                if let Err(error) =
                    notifications_repo::enqueue_wecom_bots_for_run(&db, &run.id).await
                {
                    warn!(run_id = %run.id, error = %error, "failed to enqueue notifications");
                }
            }
        }
    }
}

async fn execute_run(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    data_dir: &std::path::Path,
    job: &jobs_repo::Job,
    run_id: &str,
    started_at: OffsetDateTime,
    spec: job_spec::JobSpecV1,
) -> Result<serde_json::Value, anyhow::Error> {
    match spec {
        job_spec::JobSpecV1::Filesystem { source, target, .. } => {
            runs_repo::append_run_event(db, run_id, "info", "packaging", "packaging", None).await?;

            let data_dir = data_dir.to_path_buf();
            let job_id = job.id.clone();
            let run_id_owned = run_id.to_string();
            let part_size = target.part_size_bytes();
            let artifacts = tokio::task::spawn_blocking(move || {
                backup::filesystem::build_filesystem_run(
                    &data_dir,
                    &job_id,
                    &run_id_owned,
                    started_at,
                    &source,
                    part_size,
                )
            })
            .await??;

            runs_repo::append_run_event(db, run_id, "info", "upload", "upload", None).await?;
            let target_summary =
                store_run_artifacts_to_target(db, secrets, &job.id, run_id, &target, &artifacts)
                    .await?;

            let _ = tokio::fs::remove_dir_all(&artifacts.run_dir).await;

            Ok(serde_json::json!({
                "target": target_summary,
                "entries_count": artifacts.entries_count,
                "parts": artifacts.parts.len(),
            }))
        }
        job_spec::JobSpecV1::Sqlite { source, target, .. } => {
            runs_repo::append_run_event(db, run_id, "info", "snapshot", "snapshot", None).await?;

            let sqlite_path = source.path.clone();
            let data_dir = data_dir.to_path_buf();
            let job_id = job.id.clone();
            let run_id_owned = run_id.to_string();
            let part_size = target.part_size_bytes();
            let build = tokio::task::spawn_blocking(move || {
                backup::sqlite::build_sqlite_run(
                    &data_dir,
                    &job_id,
                    &run_id_owned,
                    started_at,
                    &source,
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
                let _ = runs_repo::append_run_event(
                    db,
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

            runs_repo::append_run_event(db, run_id, "info", "upload", "upload", None).await?;
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
        job_spec::JobSpecV1::Vaultwarden { source, target, .. } => {
            runs_repo::append_run_event(db, run_id, "info", "snapshot", "snapshot", None).await?;

            let data_dir = data_dir.to_path_buf();
            let job_id = job.id.clone();
            let run_id_owned = run_id.to_string();
            let part_size = target.part_size_bytes();
            let vw_data_dir = source.data_dir.clone();
            let artifacts = tokio::task::spawn_blocking(move || {
                backup::vaultwarden::build_vaultwarden_run(
                    &data_dir,
                    &job_id,
                    &run_id_owned,
                    started_at,
                    &source,
                    part_size,
                )
            })
            .await??;

            runs_repo::append_run_event(db, run_id, "info", "upload", "upload", None).await?;
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
            let run_url = upload_run_artifacts_to_webdav(
                db,
                secrets,
                job_id,
                run_id,
                base_url,
                secret_name,
                artifacts,
            )
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

async fn upload_run_artifacts_to_webdav(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    job_id: &str,
    run_id: &str,
    base_url: &str,
    secret_name: &str,
    artifacts: &backup::LocalRunArtifacts,
) -> Result<Url, anyhow::Error> {
    let cred_bytes = secrets_repo::get_secret(db, secrets, "webdav", secret_name)
        .await?
        .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
    let credentials = WebdavCredentials::from_json(&cred_bytes)?;

    let mut base_url = Url::parse(base_url)?;
    if !base_url.path().ends_with('/') {
        base_url.set_path(&format!("{}/", base_url.path()));
    }
    let client = WebdavClient::new(base_url.clone(), credentials)?;

    let job_url = base_url.join(&format!("{job_id}/"))?;
    client.ensure_collection(&job_url).await?;
    let run_url = job_url.join(&format!("{run_id}/"))?;
    client.ensure_collection(&run_url).await?;

    upload_artifacts(&client, &run_url, artifacts).await?;

    Ok(run_url)
}

async fn upload_artifacts(
    client: &WebdavClient,
    run_url: &Url,
    artifacts: &backup::LocalRunArtifacts,
) -> Result<(), anyhow::Error> {
    for part in &artifacts.parts {
        let url = run_url.join(&part.name)?;
        if let Some(existing) = client.head_size(&url).await? {
            if existing == part.size {
                continue;
            }
        }
        client
            .put_file_with_retries(&url, &part.path, part.size, 3)
            .await?;
    }

    let entries_size = tokio::fs::metadata(&artifacts.entries_index_path)
        .await?
        .len();
    let entries_url = run_url.join(backup::ENTRIES_INDEX_NAME)?;
    if let Some(existing) = client.head_size(&entries_url).await? {
        if existing != entries_size {
            client
                .put_file_with_retries(&entries_url, &artifacts.entries_index_path, entries_size, 3)
                .await?;
        }
    } else {
        client
            .put_file_with_retries(&entries_url, &artifacts.entries_index_path, entries_size, 3)
            .await?;
    }

    let manifest_size = tokio::fs::metadata(&artifacts.manifest_path).await?.len();
    let manifest_url = run_url.join(backup::MANIFEST_NAME)?;
    client
        .put_file_with_retries(&manifest_url, &artifacts.manifest_path, manifest_size, 3)
        .await?;

    let complete_size = tokio::fs::metadata(&artifacts.complete_path).await?.len();
    let complete_url = run_url.join(backup::COMPLETE_NAME)?;
    client
        .put_file_with_retries(&complete_url, &artifacts.complete_path, complete_size, 3)
        .await?;

    Ok(())
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
