use std::path::Path;
use std::sync::Arc;

use futures_util::{Sink, SinkExt};
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, warn};

use bastion_core::agent_protocol::{
    AgentToHubMessageV1, BackupRunTaskV1, JobConfigV1, PROTOCOL_VERSION, WebdavSecretV1,
};
use bastion_core::run_failure::RunFailedWithSummary;

use super::super::fs_list::fs_list_dir_entries;
use super::super::identity::AgentIdentityV1;
use super::super::managed::{
    load_cached_task_result, save_managed_config_snapshot, save_managed_secrets_snapshot,
    save_task_result,
};
use super::super::util::is_ws_error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HandlerFlow {
    Continue,
    Reconnect,
}

async fn send_json<S>(tx: &mut S, msg: &impl serde::Serialize) -> Result<HandlerFlow, anyhow::Error>
where
    S: Sink<Message, Error = tungstenite::Error> + Unpin,
{
    let text = serde_json::to_string(msg)?;
    if tx.send(Message::Text(text.into())).await.is_err() {
        return Ok(HandlerFlow::Reconnect);
    }
    Ok(HandlerFlow::Continue)
}

pub(super) async fn handle_secrets_snapshot(
    identity: &AgentIdentityV1,
    data_dir: &Path,
    node_id: String,
    issued_at: i64,
    webdav: Vec<WebdavSecretV1>,
) -> HandlerFlow {
    if node_id != identity.agent_id {
        warn!(
            agent_id = %identity.agent_id,
            node_id = %node_id,
            "received secrets snapshot for unexpected node_id; ignoring"
        );
        return HandlerFlow::Continue;
    }

    if let Err(error) = save_managed_secrets_snapshot(data_dir, &node_id, issued_at, &webdav) {
        warn!(
            agent_id = %identity.agent_id,
            error = %error,
            "failed to persist secrets snapshot"
        );
    } else {
        debug!(
            agent_id = %identity.agent_id,
            webdav = webdav.len(),
            "persisted secrets snapshot"
        );
    }

    HandlerFlow::Continue
}

pub(super) async fn handle_config_snapshot<S>(
    tx: &mut S,
    identity: &AgentIdentityV1,
    data_dir: &Path,
    node_id: String,
    snapshot_id: String,
    issued_at: i64,
    jobs: Vec<JobConfigV1>,
) -> Result<HandlerFlow, anyhow::Error>
where
    S: Sink<Message, Error = tungstenite::Error> + Unpin,
{
    if node_id != identity.agent_id {
        warn!(
            agent_id = %identity.agent_id,
            node_id = %node_id,
            "received config snapshot for unexpected node_id; ignoring"
        );
        return Ok(HandlerFlow::Continue);
    }

    if let Err(error) =
        save_managed_config_snapshot(data_dir, &node_id, &snapshot_id, issued_at, &jobs)
    {
        warn!(
            agent_id = %identity.agent_id,
            snapshot_id = %snapshot_id,
            error = %error,
            "failed to persist config snapshot"
        );
    } else {
        debug!(
            agent_id = %identity.agent_id,
            snapshot_id = %snapshot_id,
            jobs = jobs.len(),
            "persisted config snapshot"
        );
    }

    let ack = AgentToHubMessageV1::ConfigAck {
        v: PROTOCOL_VERSION,
        snapshot_id,
    };
    send_json(tx, &ack).await
}

pub(super) async fn handle_task<S>(
    tx: &mut S,
    data_dir: &Path,
    run_lock: Arc<tokio::sync::Mutex<()>>,
    task_id: String,
    task: Box<BackupRunTaskV1>,
) -> Result<HandlerFlow, anyhow::Error>
where
    S: Sink<Message, Error = tungstenite::Error> + Unpin,
{
    let run_id = task.run_id.clone();
    debug!(task_id = %task_id, run_id = %run_id, "received task");

    if let Some(cached) = load_cached_task_result(data_dir, &task_id, &run_id) {
        debug!(
            task_id = %task_id,
            run_id = %run_id,
            "replaying cached task result"
        );
        let ack = AgentToHubMessageV1::Ack {
            v: PROTOCOL_VERSION,
            task_id: task_id.clone(),
        };
        if send_json(tx, &ack).await? == HandlerFlow::Reconnect {
            return Ok(HandlerFlow::Reconnect);
        }

        if send_json(tx, &cached).await? == HandlerFlow::Reconnect {
            return Ok(HandlerFlow::Reconnect);
        }
        return Ok(HandlerFlow::Continue);
    }

    let ack = AgentToHubMessageV1::Ack {
        v: PROTOCOL_VERSION,
        task_id: task_id.clone(),
    };
    if send_json(tx, &ack).await? == HandlerFlow::Reconnect {
        return Ok(HandlerFlow::Reconnect);
    }

    let _guard = run_lock.lock().await;
    match super::super::handle_backup_task(data_dir, tx, &task_id, *task).await {
        Ok(()) => {}
        Err(error) => {
            if is_ws_error(&error) {
                warn!(
                    task_id = %task_id,
                    run_id = %run_id,
                    error = %error,
                    "task aborted due to websocket error; reconnecting"
                );
                return Ok(HandlerFlow::Reconnect);
            }

            warn!(task_id = %task_id, run_id = %run_id, error = %error, "task failed");
            let summary = error
                .downcast_ref::<RunFailedWithSummary>()
                .map(|e| e.summary.clone());
            let result = AgentToHubMessageV1::TaskResult {
                v: PROTOCOL_VERSION,
                task_id: task_id.clone(),
                run_id,
                status: "failed".to_string(),
                summary,
                error: Some(format!("{error:#}")),
            };

            if let Err(error) = save_task_result(data_dir, &result) {
                warn!(task_id = %task_id, error = %error, "failed to persist task result");
            }

            if send_json(tx, &result).await? == HandlerFlow::Reconnect {
                return Ok(HandlerFlow::Reconnect);
            }
        }
    }

    Ok(HandlerFlow::Continue)
}

pub(super) async fn handle_fs_list<S>(
    tx: &mut S,
    request_id: String,
    path: String,
) -> Result<HandlerFlow, anyhow::Error>
where
    S: Sink<Message, Error = tungstenite::Error> + Unpin,
{
    let path = path.trim().to_string();
    let result = tokio::task::spawn_blocking(move || fs_list_dir_entries(&path)).await;
    let (entries, error) = match result {
        Ok(Ok(entries)) => (entries, None),
        Ok(Err(msg)) => (Vec::new(), Some(msg)),
        Err(error) => (Vec::new(), Some(format!("fs list task failed: {error}"))),
    };

    let msg = AgentToHubMessageV1::FsListResult {
        v: PROTOCOL_VERSION,
        request_id,
        entries,
        error,
    };
    send_json(tx, &msg).await
}
