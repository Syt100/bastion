use time::OffsetDateTime;
use tracing::{debug, info, warn};

use bastion_core::job_spec;
use bastion_core::run_failure::RunFailedWithSummary;
use bastion_storage::jobs_repo;
use bastion_storage::run_artifacts_repo;
use bastion_storage::runs_repo::{self, RunStatus};

use crate::cancel_registry::global_cancel_registry;
use crate::run_events;

use super::super::execute::{ExecuteRunArgs, RunCanceled, execute_run};
use super::WorkerLoopCtx;
use super::notifications;

pub(super) async fn execute_and_complete(
    ctx: &WorkerLoopCtx<'_>,
    job: &jobs_repo::Job,
    run: &runs_repo::Run,
    started_at: OffsetDateTime,
    spec: job_spec::JobSpecV1,
) {
    let cancel_token = global_cancel_registry().register_run(&run.id);
    struct CancelRegistration {
        run_id: String,
    }
    impl Drop for CancelRegistration {
        fn drop(&mut self) {
            global_cancel_registry().unregister_run(&self.run_id);
        }
    }
    let _registration = CancelRegistration {
        run_id: run.id.clone(),
    };

    match execute_run(ExecuteRunArgs {
        db: ctx.db,
        secrets: ctx.secrets,
        run_events_bus: ctx.run_events_bus,
        data_dir: ctx.data_dir,
        job,
        run_id: &run.id,
        started_at,
        cancel_token,
        spec: spec.clone(),
    })
    .await
    {
        Ok(summary) => {
            info!(run_id = %run.id, "run ok");
            let ended_at = OffsetDateTime::now_utc().unix_timestamp();
            let completed = match runs_repo::complete_run(
                ctx.db,
                &run.id,
                RunStatus::Success,
                Some(summary),
                None,
            )
            .await
            {
                Ok(v) => v,
                Err(error) => {
                    warn!(run_id = %run.id, error = %error, "failed to complete run");
                    return;
                }
            };
            if !completed {
                warn!(run_id = %run.id, "run completion skipped (already finalized)");
            }

            let final_status = runs_repo::get_run(ctx.db, &run.id)
                .await
                .ok()
                .flatten()
                .map(|r| r.status)
                .unwrap_or(RunStatus::Success);

            if final_status == RunStatus::Canceled {
                let _ = run_events::append_and_broadcast(
                    ctx.db,
                    ctx.run_events_bus,
                    &run.id,
                    "info",
                    "canceled",
                    "canceled",
                    None,
                )
                .await;
                info!(run_id = %run.id, "run canceled");
                return;
            }

            if final_status != RunStatus::Success {
                warn!(
                    run_id = %run.id,
                    status = %final_status.as_str(),
                    "run completed but final status is not success"
                );
                return;
            }

            if let Err(error) =
                run_artifacts_repo::upsert_run_artifact_from_successful_run(ctx.db, &run.id).await
            {
                warn!(run_id = %run.id, error = %error, "failed to index run artifact");
            } else {
                let _ = run_events::append_and_broadcast(
                    ctx.db,
                    ctx.run_events_bus,
                    &run.id,
                    "info",
                    "complete",
                    "complete",
                    None,
                )
                .await;
                notifications::enqueue_for_run_spec(ctx, &spec, &run.id).await;
                debug!(run_id = %run.id, ended_at, "run completed");
            }
        }
        Err(error) => {
            let canceled = error.downcast_ref::<RunCanceled>().is_some();
            if canceled {
                info!(run_id = %run.id, "run canceled");
            } else {
                warn!(run_id = %run.id, error = %error, "run failed");
            }

            let soft = error.downcast_ref::<RunFailedWithSummary>();
            let requested_status = if canceled {
                RunStatus::Canceled
            } else {
                RunStatus::Failed
            };
            let summary = if canceled {
                None
            } else {
                soft.map(|e| e.summary.clone())
            };
            let error_code = if canceled {
                Some("canceled")
            } else {
                Some(soft.map(|e| e.code).unwrap_or("run_failed"))
            };

            let completed = match runs_repo::complete_run(
                ctx.db,
                &run.id,
                requested_status,
                summary,
                error_code,
            )
            .await
            {
                Ok(v) => v,
                Err(complete_error) => {
                    warn!(
                        run_id = %run.id,
                        error = %complete_error,
                        "failed to complete run after error"
                    );
                    return;
                }
            };
            if !completed {
                warn!(run_id = %run.id, "run completion skipped (already finalized)");
            }

            let final_status = runs_repo::get_run(ctx.db, &run.id)
                .await
                .ok()
                .flatten()
                .map(|r| r.status)
                .unwrap_or(requested_status);

            if final_status == RunStatus::Canceled {
                let _ = run_events::append_and_broadcast(
                    ctx.db,
                    ctx.run_events_bus,
                    &run.id,
                    "info",
                    "canceled",
                    "canceled",
                    None,
                )
                .await;
                return;
            }

            let message = format!("failed: {error}");
            let _ = run_events::append_and_broadcast(
                ctx.db,
                ctx.run_events_bus,
                &run.id,
                "error",
                "failed",
                &message,
                None,
            )
            .await;
            notifications::enqueue_for_run_spec(ctx, &spec, &run.id).await;
        }
    }
}
