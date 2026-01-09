use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use bastion_storage::secrets::SecretsCrypto;

use crate::agent_manager::AgentManager;
use crate::run_events_bus::RunEventsBus;

mod dispatch;
mod execute;
mod r#loop;
mod target_store;

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
    r#loop::run_worker_loop(args).await;
}
