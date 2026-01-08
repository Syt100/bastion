use std::path::PathBuf;

mod cron_loop;
mod sink;
mod types;
mod worker_loop;

use types::{InFlightCounts, OfflineRunTask};

pub(super) async fn offline_scheduler_loop(
    data_dir: PathBuf,
    agent_id: String,
    run_lock: std::sync::Arc<tokio::sync::Mutex<()>>,
    connected_rx: tokio::sync::watch::Receiver<bool>,
) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<OfflineRunTask>();
    let inflight = std::sync::Arc::new(tokio::sync::Mutex::new(InFlightCounts::default()));

    tokio::spawn(cron_loop::offline_cron_loop(
        data_dir.clone(),
        agent_id.clone(),
        connected_rx,
        tx,
        inflight.clone(),
    ));

    worker_loop::offline_worker_loop(data_dir, agent_id, run_lock, rx, inflight).await;
}
