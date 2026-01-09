use std::path::Path;

use sqlx::SqlitePool;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use bastion_storage::secrets::SecretsCrypto;

use crate::agent_manager::AgentManager;
use crate::run_events_bus::RunEventsBus;

use super::WorkerLoopArgs;

mod agent;
mod claim;
mod local;
mod notifications;
mod process;

struct WorkerLoopCtx<'a> {
    db: &'a SqlitePool,
    data_dir: &'a Path,
    secrets: &'a SecretsCrypto,
    agent_manager: &'a AgentManager,
    run_events_bus: &'a RunEventsBus,
    run_queue_notify: &'a Notify,
    notifications_notify: &'a Notify,
    shutdown: &'a CancellationToken,
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

    let ctx = WorkerLoopCtx {
        db: &db,
        data_dir: data_dir.as_path(),
        secrets: secrets.as_ref(),
        agent_manager: &agent_manager,
        run_events_bus: run_events_bus.as_ref(),
        run_queue_notify: run_queue_notify.as_ref(),
        notifications_notify: notifications_notify.as_ref(),
        shutdown: &shutdown,
    };

    loop {
        if shutdown.is_cancelled() {
            break;
        }

        let Some(run) = claim::claim_next_queued_run_or_wait(&ctx).await else {
            continue;
        };

        process::process_run(&ctx, run).await;
    }
}
