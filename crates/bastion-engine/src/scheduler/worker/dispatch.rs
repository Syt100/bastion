use sqlx::SqlitePool;
use time::OffsetDateTime;

use bastion_core::agent_protocol::{BackupRunTaskV1, HubToAgentMessageV1, PROTOCOL_VERSION};
use bastion_core::job_spec;
use bastion_storage::agent_tasks_repo;
use bastion_storage::jobs_repo;
use bastion_storage::secrets::SecretsCrypto;

use crate::agent_manager::AgentManager;
use crate::run_events;
use crate::run_events_bus::RunEventsBus;

pub(super) struct DispatchRunToAgentArgs<'a> {
    pub(super) db: &'a SqlitePool,
    pub(super) secrets: &'a SecretsCrypto,
    pub(super) agent_manager: &'a AgentManager,
    pub(super) run_events_bus: &'a RunEventsBus,
    pub(super) job: &'a jobs_repo::Job,
    pub(super) run_id: &'a str,
    pub(super) started_at: OffsetDateTime,
    pub(super) spec: job_spec::JobSpecV1,
    pub(super) agent_id: &'a str,
}

pub(super) async fn dispatch_run_to_agent(
    args: DispatchRunToAgentArgs<'_>,
) -> Result<(), anyhow::Error> {
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
