use time::OffsetDateTime;
use tracing::{debug, info, warn};

use bastion_core::job_spec;
use bastion_core::run_failure::RunFailedWithSummary;
use bastion_storage::jobs_repo;
use bastion_storage::runs_repo::{self, RunStatus};

use crate::run_events;

use super::super::execute::{ExecuteRunArgs, execute_run};
use super::WorkerLoopCtx;
use super::notifications;

pub(super) async fn execute_and_complete(
    ctx: &WorkerLoopCtx<'_>,
    job: &jobs_repo::Job,
    run: &runs_repo::Run,
    started_at: OffsetDateTime,
    spec: job_spec::JobSpecV1,
) {
    match execute_run(ExecuteRunArgs {
        db: ctx.db,
        secrets: ctx.secrets,
        run_events_bus: ctx.run_events_bus,
        data_dir: ctx.data_dir,
        job,
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
                runs_repo::complete_run(ctx.db, &run.id, RunStatus::Success, Some(summary), None)
                    .await
            {
                warn!(run_id = %run.id, error = %error, "failed to complete run");
                return;
            }
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
        Err(error) => {
            warn!(run_id = %run.id, error = %error, "run failed");
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

            let soft = error.downcast_ref::<RunFailedWithSummary>();
            let summary = soft.map(|e| e.summary.clone());
            let error_code = soft.map(|e| e.code).unwrap_or("run_failed");

            let _ = runs_repo::complete_run(
                ctx.db,
                &run.id,
                RunStatus::Failed,
                summary,
                Some(error_code),
            )
            .await;
            notifications::enqueue_for_run_spec(ctx, &spec, &run.id).await;
        }
    }
}
