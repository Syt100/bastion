use std::sync::Arc;

use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use bastion_core::HUB_NODE_ID;
use bastion_core::agent_protocol::{BackupRunTaskV1, HubToAgentMessageV1, PROTOCOL_VERSION};
use bastion_core::job_spec;
use bastion_core::run_failure::RunFailedWithSummary;
use bastion_storage::agent_tasks_repo;
use bastion_storage::jobs_repo;
use bastion_storage::runs_repo::{self, RunStatus};
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;
use bastion_targets::WebdavCredentials;

use crate::agent_manager::AgentManager;
use crate::run_events;
use crate::run_events_bus::RunEventsBus;
use bastion_backup as backup;
use bastion_backup::backup_encryption;
use bastion_targets as targets;

pub(super) struct WorkerLoopArgs {
    pub(super) db: SqlitePool,
    pub(super) data_dir: std::path::PathBuf,
    pub(super) secrets: Arc<SecretsCrypto>,
    pub(super) agent_manager: AgentManager,
    pub(super) run_events_bus: Arc<RunEventsBus>,
    pub(super) run_queue_notify: Arc<Notify>,
    pub(super) notifications_notify: Arc<Notify>,
    pub(super) shutdown: CancellationToken,
}

pub(super) async fn run_worker_loop(args: WorkerLoopArgs) {
    let WorkerLoopArgs {
        db,
        data_dir,
        secrets,
        agent_manager,
        run_events_bus,
        run_queue_notify,
        notifications_notify,
        shutdown,
    } = args;
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
            if let Err(error) = dispatch_run_to_agent(DispatchRunToAgentArgs {
                db: &db,
                secrets: &secrets,
                agent_manager: &agent_manager,
                run_events_bus: run_events_bus.as_ref(),
                job: &job,
                run_id: &run.id,
                started_at,
                spec: spec.clone(),
                agent_id,
            })
            .await
            {
                warn!(
                    run_id = %run.id,
                    agent_id = %agent_id,
                    error = %error,
                    "dispatch failed"
                );
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
                    match crate::notifications::enqueue_for_run_spec(&db, &spec, &run.id).await {
                        Ok(true) => notifications_notify.notify_one(),
                        Ok(false) => {}
                        Err(error) => {
                            warn!(
                                run_id = %run.id,
                                error = %error,
                                "failed to enqueue notifications"
                            );
                        }
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

                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }

            continue;
        }

        match execute_run(ExecuteRunArgs {
            db: &db,
            secrets: &secrets,
            run_events_bus: &run_events_bus,
            data_dir: &data_dir,
            job: &job,
            run_id: &run.id,
            started_at,
            spec: spec.clone(),
        })
        .await
        {
            Ok(summary) => {
                info!(run_id = %run.id, "run ok");
                let ended_at = OffsetDateTime::now_utc().unix_timestamp();
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
                match crate::notifications::enqueue_for_run_spec(&db, &spec, &run.id).await {
                    Ok(true) => notifications_notify.notify_one(),
                    Ok(false) => {}
                    Err(error) => {
                        warn!(
                            run_id = %run.id,
                            error = %error,
                            "failed to enqueue notifications"
                        );
                    }
                }
                debug!(run_id = %run.id, ended_at, "run completed");
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
                match crate::notifications::enqueue_for_run_spec(&db, &spec, &run.id).await {
                    Ok(true) => notifications_notify.notify_one(),
                    Ok(false) => {}
                    Err(error) => {
                        warn!(
                            run_id = %run.id,
                            error = %error,
                            "failed to enqueue notifications"
                        );
                    }
                }
            }
        }
    }
}

struct DispatchRunToAgentArgs<'a> {
    db: &'a SqlitePool,
    secrets: &'a SecretsCrypto,
    agent_manager: &'a AgentManager,
    run_events_bus: &'a RunEventsBus,
    job: &'a jobs_repo::Job,
    run_id: &'a str,
    started_at: OffsetDateTime,
    spec: job_spec::JobSpecV1,
    agent_id: &'a str,
}

async fn dispatch_run_to_agent(args: DispatchRunToAgentArgs<'_>) -> Result<(), anyhow::Error> {
    let DispatchRunToAgentArgs {
        db,
        secrets,
        agent_manager,
        run_events_bus,
        job,
        run_id,
        started_at,
        spec,
        agent_id,
    } = args;
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

    let resolved =
        crate::agent_job_resolver::resolve_job_spec_for_agent(db, secrets, agent_id, spec).await?;
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
        task: Box::new(task),
    };

    let payload = serde_json::to_value(&msg)?;
    agent_tasks_repo::upsert_task(db, run_id, agent_id, run_id, "sent", &payload).await?;

    agent_manager.send_json(agent_id, &msg).await?;
    Ok(())
}

struct ExecuteRunArgs<'a> {
    db: &'a SqlitePool,
    secrets: &'a SecretsCrypto,
    run_events_bus: &'a RunEventsBus,
    data_dir: &'a std::path::Path,
    job: &'a jobs_repo::Job,
    run_id: &'a str,
    started_at: OffsetDateTime,
    spec: job_spec::JobSpecV1,
}

async fn execute_run(args: ExecuteRunArgs<'_>) -> Result<serde_json::Value, anyhow::Error> {
    let ExecuteRunArgs {
        db,
        secrets,
        run_events_bus,
        data_dir,
        job,
        run_id,
        started_at,
        spec,
    } = args;
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
                "packaging",
                "packaging",
                None,
            )
            .await?;

            let data_dir = data_dir.to_path_buf();
            let job_id = job.id.clone();
            let run_id_owned = run_id.to_string();
            let vw_data_dir = source.data_dir.clone();
            let part_size = target.part_size_bytes();
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
            let cred_bytes =
                secrets_repo::get_secret(db, secrets, HUB_NODE_ID, "webdav", secret_name)
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
