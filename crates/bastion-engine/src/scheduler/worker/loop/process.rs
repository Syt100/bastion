use time::OffsetDateTime;
use tracing::{info, warn};

use bastion_core::HUB_NODE_ID;
use bastion_core::job_spec;
use bastion_storage::jobs_repo;
use bastion_storage::runs_repo::{self, RunStatus};

use crate::run_events;
use crate::scheduler::target_snapshot;

use super::WorkerLoopCtx;

pub(super) async fn process_run(ctx: &WorkerLoopCtx<'_>, run: runs_repo::Run) {
    info!(run_id = %run.id, job_id = %run.job_id, "run started");

    if let Err(error) = run_events::append_and_broadcast(
        ctx.db,
        ctx.run_events_bus,
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

    let job = match jobs_repo::get_job(ctx.db, &run.job_id).await {
        Ok(Some(job)) => job,
        Ok(None) => {
            let _ = runs_repo::complete_run(
                ctx.db,
                &run.id,
                RunStatus::Failed,
                None,
                Some("job_not_found"),
            )
            .await;
            return;
        }
        Err(error) => {
            warn!(run_id = %run.id, error = %error, "failed to load job");
            let _ = runs_repo::complete_run(
                ctx.db,
                &run.id,
                RunStatus::Failed,
                None,
                Some("job_load_failed"),
            )
            .await;
            return;
        }
    };

    let spec = match job_spec::parse_value(&job.spec) {
        Ok(v) => v,
        Err(error) => {
            let message = format!("invalid spec: {error}");
            fail_invalid_spec(ctx, &run.id, &message).await;
            return;
        }
    };

    if let Err(error) = job_spec::validate(&spec) {
        let message = format!("invalid spec: {error}");
        fail_invalid_spec(ctx, &run.id, &message).await;
        return;
    }

    let node_id = job.agent_id.as_deref().unwrap_or(HUB_NODE_ID);
    let snapshot = target_snapshot::build_run_target_snapshot(node_id, &spec);
    if let Err(error) = runs_repo::set_run_target_snapshot(ctx.db, &run.id, snapshot).await {
        warn!(
            run_id = %run.id,
            error = %error,
            "failed to persist run target snapshot"
        );
    }

    let started_at = OffsetDateTime::from_unix_timestamp(run.started_at)
        .unwrap_or_else(|_| OffsetDateTime::now_utc());

    if let Some(agent_id) = job.agent_id.as_deref() {
        super::agent::dispatch_and_wait(ctx, &job, &run, started_at, spec, agent_id).await;
        return;
    }

    super::local::execute_and_complete(ctx, &job, &run, started_at, spec).await;
}

async fn fail_invalid_spec(ctx: &WorkerLoopCtx<'_>, run_id: &str, message: &str) {
    let _ = run_events::append_and_broadcast(
        ctx.db,
        ctx.run_events_bus,
        run_id,
        "error",
        "invalid_spec",
        message,
        None,
    )
    .await;

    let _ = runs_repo::complete_run(
        ctx.db,
        run_id,
        RunStatus::Failed,
        None,
        Some("invalid_spec"),
    )
    .await;
}
