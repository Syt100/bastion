use time::OffsetDateTime;
use tracing::{info, warn};

use bastion_core::job_spec;
use bastion_storage::agent_tasks_repo;
use bastion_storage::jobs_repo;
use bastion_storage::runs_repo::{self, RunStatus};

use crate::run_events;

use super::super::dispatch::{DispatchRunToAgentArgs, dispatch_run_to_agent};
use super::WorkerLoopCtx;
use super::notifications;

pub(super) async fn dispatch_and_wait(
    ctx: &WorkerLoopCtx<'_>,
    job: &jobs_repo::Job,
    run: &runs_repo::Run,
    started_at: OffsetDateTime,
    spec: job_spec::JobSpecV1,
    agent_id: &str,
) {
    if let Err(error) = dispatch_run_to_agent(DispatchRunToAgentArgs {
        db: ctx.db,
        secrets: ctx.secrets,
        agent_manager: ctx.agent_manager,
        run_events_bus: ctx.run_events_bus,
        job,
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
            ctx.db,
            ctx.run_events_bus,
            &run.id,
            "error",
            "dispatch_failed",
            &message,
            Some(serde_json::json!({ "agent_id": agent_id })),
        )
        .await;

        let _ = runs_repo::requeue_run(ctx.db, &run.id).await;
        let _ = agent_tasks_repo::delete_task(ctx.db, &run.id).await;
        ctx.run_queue_notify.notify_one();
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        return;
    }

    // Wait for Agent to complete the run (single-worker, no parallel runs).
    let deadline = OffsetDateTime::now_utc()
        .checked_add(time::Duration::hours(24))
        .unwrap_or_else(OffsetDateTime::now_utc);
    loop {
        let Some(current) = runs_repo::get_run(ctx.db, &run.id).await.unwrap_or(None) else {
            break;
        };
        if current.status != RunStatus::Running {
            notifications::enqueue_for_run_spec(ctx, &spec, &run.id).await;
            info!(run_id = %run.id, "run completed (agent)");
            break;
        }

        if OffsetDateTime::now_utc() >= deadline {
            warn!(run_id = %run.id, agent_id = %agent_id, "agent run timed out");
            let _ = run_events::append_and_broadcast(
                ctx.db,
                ctx.run_events_bus,
                &run.id,
                "error",
                "timeout",
                "timeout",
                Some(serde_json::json!({ "agent_id": agent_id })),
            )
            .await;

            let _ =
                runs_repo::complete_run(ctx.db, &run.id, RunStatus::Failed, None, Some("timeout"))
                    .await;
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
