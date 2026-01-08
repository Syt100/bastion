use std::sync::Arc;

use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use bastion_core::job_spec;
use bastion_core::run_failure::RunFailedWithSummary;
use bastion_storage::agent_tasks_repo;
use bastion_storage::jobs_repo;
use bastion_storage::runs_repo::{self, RunStatus};
use bastion_storage::secrets::SecretsCrypto;

use crate::agent_manager::AgentManager;
use crate::run_events;
use crate::run_events_bus::RunEventsBus;

mod dispatch;
mod execute;
mod target_store;

use dispatch::{DispatchRunToAgentArgs, dispatch_run_to_agent};
use execute::{ExecuteRunArgs, execute_run};

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
