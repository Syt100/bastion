use std::path::{Path, PathBuf};

use url::Url;

mod cron;
mod scheduler;
mod storage;
mod sync;

pub(super) async fn offline_scheduler_loop(
    data_dir: PathBuf,
    agent_id: String,
    run_lock: std::sync::Arc<tokio::sync::Mutex<()>>,
    connected_rx: tokio::sync::watch::Receiver<bool>,
) {
    scheduler::offline_scheduler_loop(data_dir, agent_id, run_lock, connected_rx).await;
}

pub(super) async fn sync_offline_runs(
    base_url: &Url,
    agent_key: &str,
    data_dir: &Path,
) -> Result<(), anyhow::Error> {
    sync::sync_offline_runs(base_url, agent_key, data_dir).await
}
