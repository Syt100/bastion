use time::OffsetDateTime;
use tracing::{info, warn};

use bastion_core::job_spec;
use bastion_storage::agent_tasks_repo;
use bastion_storage::jobs_repo;
use bastion_storage::runs_repo::{self, RunStatus};

use crate::error_envelope::{
    envelope, insert_error_envelope, origin, retriable, retriable_with_reason, transport,
    with_context_param,
};
use crate::run_events;

use super::super::dispatch::{DispatchRunToAgentArgs, dispatch_run_to_agent};
use super::WorkerLoopCtx;
use super::notifications;

fn classify_dispatch_error(
    error: &anyhow::Error,
) -> (&'static str, bool, &'static str, &'static str) {
    let chain = error
        .chain()
        .map(|cause| cause.to_string().to_lowercase())
        .collect::<Vec<_>>()
        .join(" | ");

    if chain.contains("agent_driver_capability_mismatch") {
        return (
            "config",
            false,
            "diagnostics.hint.dispatch.capability_mismatch",
            "diagnostics.message.dispatch.capability_mismatch",
        );
    }

    if chain.contains("agent not connected")
        || chain.contains("timed out")
        || chain.contains("connection")
        || chain.contains("network")
    {
        return (
            "network",
            true,
            "diagnostics.hint.dispatch.agent_unreachable",
            "diagnostics.message.dispatch.agent_unreachable",
        );
    }

    (
        "unknown",
        true,
        "diagnostics.hint.dispatch.unknown",
        "diagnostics.message.dispatch.unknown",
    )
}

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
        let (error_kind, is_retriable, hint_key, message_key) = classify_dispatch_error(&error);
        let mut fields = serde_json::Map::new();
        fields.insert(
            "agent_id".to_string(),
            serde_json::Value::String(agent_id.to_string()),
        );
        fields.insert(
            "error_kind".to_string(),
            serde_json::Value::String(error_kind.to_string()),
        );
        let mut env = envelope(
            format!("scheduler.dispatch.{error_kind}"),
            error_kind,
            if is_retriable {
                retriable_with_reason(true, error_kind)
            } else {
                retriable(false)
            },
            hint_key,
            message_key,
            transport("internal"),
        )
        .with_origin(origin("scheduler", "dispatch", "agent_task"))
        .with_stage("dispatch");
        env = with_context_param(env, "agent_id", agent_id);
        env = with_context_param(env, "error", error.to_string());
        insert_error_envelope(&mut fields, env);
        let _ = run_events::append_and_broadcast(
            ctx.db,
            ctx.run_events_bus,
            &run.id,
            "error",
            "dispatch_failed",
            &message,
            Some(serde_json::Value::Object(fields)),
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
            let mut fields = serde_json::Map::new();
            fields.insert(
                "agent_id".to_string(),
                serde_json::Value::String(agent_id.to_string()),
            );
            fields.insert(
                "error_kind".to_string(),
                serde_json::Value::String("timeout".to_string()),
            );
            let mut env = envelope(
                "scheduler.agent.timeout",
                "timeout",
                retriable(false),
                "diagnostics.hint.dispatch.timeout",
                "diagnostics.message.dispatch.timeout",
                transport("internal"),
            )
            .with_origin(origin("scheduler", "agent", "wait_completion"))
            .with_stage("running");
            env = with_context_param(env, "agent_id", agent_id);
            insert_error_envelope(&mut fields, env);
            let _ = run_events::append_and_broadcast(
                ctx.db,
                ctx.run_events_bus,
                &run.id,
                "error",
                "timeout",
                "timeout",
                Some(serde_json::Value::Object(fields)),
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

#[cfg(test)]
mod tests {
    use super::classify_dispatch_error;

    #[test]
    fn classify_dispatch_error_covers_capability_and_network_failures() {
        let mismatch = anyhow::anyhow!("agent_driver_capability_mismatch: source unsupported");
        assert_eq!(classify_dispatch_error(&mismatch).0, "config");
        assert!(!classify_dispatch_error(&mismatch).1);

        let network = anyhow::anyhow!("agent not connected");
        assert_eq!(classify_dispatch_error(&network).0, "network");
        assert!(classify_dispatch_error(&network).1);

        let unknown = anyhow::anyhow!("boom");
        assert_eq!(classify_dispatch_error(&unknown).0, "unknown");
    }
}
