use std::path::Path;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tracing::{debug, warn};
use url::Url;

use bastion_core::agent_protocol::{AgentToHubMessageV1, HubToAgentMessageV1, PROTOCOL_VERSION};
use bastion_core::run_failure::RunFailedWithSummary;

use super::fs_list::fs_list_dir_entries;
use super::identity::AgentIdentityV1;
use super::managed::{
    load_cached_task_result, save_managed_config_snapshot, save_managed_secrets_snapshot,
    save_task_result,
};
use super::offline;
use super::util::{is_ws_error, normalize_base_url};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LoopAction {
    Reconnect,
    Exit,
}

pub(super) async fn connect_and_run(
    ws_url: &Url,
    identity: &AgentIdentityV1,
    data_dir: &Path,
    heartbeat: Duration,
    pong_timeout: Duration,
    run_lock: std::sync::Arc<tokio::sync::Mutex<()>>,
    connected_tx: &tokio::sync::watch::Sender<bool>,
) -> Result<LoopAction, anyhow::Error> {
    let mut req = ws_url.as_str().into_client_request()?;
    req.headers_mut().insert(
        AUTHORIZATION,
        format!("Bearer {}", identity.agent_key).parse()?,
    );

    let (socket, _) = tokio_tungstenite::connect_async(req).await?;
    let (mut tx, mut rx) = socket.split();

    let _ = connected_tx.send(true);
    struct ConnectedGuard(tokio::sync::watch::Sender<bool>);
    impl Drop for ConnectedGuard {
        fn drop(&mut self) {
            let _ = self.0.send(false);
        }
    }
    let _connected_guard = ConnectedGuard(connected_tx.clone());

    let hello = AgentToHubMessageV1::Hello {
        v: PROTOCOL_VERSION,
        agent_id: identity.agent_id.clone(),
        name: identity.name.clone(),
        info: serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        }),
        capabilities: serde_json::json!({
            "backup": ["filesystem", "sqlite", "vaultwarden"],
            "control": ["fs_list"],
        }),
    };
    tx.send(Message::Text(serde_json::to_string(&hello)?.into()))
        .await?;

    if let Ok(base_url) = normalize_base_url(&identity.hub_url)
        && let Err(error) =
            offline::sync_offline_runs(&base_url, &identity.agent_key, data_dir).await
    {
        warn!(
            agent_id = %identity.agent_id,
            error = %error,
            "failed to sync offline runs"
        );
    }

    let mut tick = tokio::time::interval(heartbeat);
    let mut last_pong = tokio::time::Instant::now();
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            _ = &mut shutdown => {
                let _ = tx.send(Message::Close(None)).await;
                return Ok(LoopAction::Exit);
            }
            _ = tick.tick() => {
                if last_pong.elapsed() > pong_timeout {
                    warn!(
                        agent_id = %identity.agent_id,
                        timeout_seconds = pong_timeout.as_secs(),
                        "pong timeout; reconnecting"
                    );
                    return Ok(LoopAction::Reconnect);
                }
                let ping = AgentToHubMessageV1::Ping { v: PROTOCOL_VERSION };
                if tx.send(Message::Text(serde_json::to_string(&ping)?.into())).await.is_err() {
                    return Ok(LoopAction::Reconnect);
                }
            }
            msg = rx.next() => {
                let Some(msg) = msg else {
                    return Ok(LoopAction::Reconnect);
                };
                match msg {
                    Ok(Message::Text(text)) => {
                        let text = text.to_string();
                        match serde_json::from_str::<HubToAgentMessageV1>(&text) {
                            Ok(HubToAgentMessageV1::Pong { .. }) => {
                                last_pong = tokio::time::Instant::now();
                            }
                            Ok(HubToAgentMessageV1::SecretsSnapshot {
                                v,
                                node_id,
                                issued_at,
                                webdav,
                            }) if v == PROTOCOL_VERSION => {
                                if node_id != identity.agent_id {
                                    warn!(
                                        agent_id = %identity.agent_id,
                                        node_id = %node_id,
                                        "received secrets snapshot for unexpected node_id; ignoring"
                                    );
                                    continue;
                                }

                                if let Err(error) =
                                    save_managed_secrets_snapshot(data_dir, &node_id, issued_at, &webdav)
                                {
                                    warn!(agent_id = %identity.agent_id, error = %error, "failed to persist secrets snapshot");
                                } else {
                                    debug!(
                                        agent_id = %identity.agent_id,
                                        webdav = webdav.len(),
                                        "persisted secrets snapshot"
                                    );
                                }
                            }
                            Ok(HubToAgentMessageV1::ConfigSnapshot {
                                v,
                                node_id,
                                snapshot_id,
                                issued_at,
                                jobs,
                            }) if v == PROTOCOL_VERSION => {
                                if node_id != identity.agent_id {
                                    warn!(
                                        agent_id = %identity.agent_id,
                                        node_id = %node_id,
                                        "received config snapshot for unexpected node_id; ignoring"
                                    );
                                    continue;
                                }

                                if let Err(error) = save_managed_config_snapshot(
                                    data_dir,
                                    &node_id,
                                    &snapshot_id,
                                    issued_at,
                                    &jobs,
                                ) {
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
                                if tx.send(Message::Text(serde_json::to_string(&ack)?.into())).await.is_err() {
                                    return Ok(LoopAction::Reconnect);
                                }
                            }
                            Ok(HubToAgentMessageV1::Task { v, task_id, task }) if v == PROTOCOL_VERSION => {
                                let run_id = task.run_id.clone();
                                debug!(task_id = %task_id, run_id = %run_id, "received task");

                                if let Some(cached) = load_cached_task_result(data_dir, &task_id, &run_id) {
                                    debug!(task_id = %task_id, run_id = %run_id, "replaying cached task result");
                                    let ack = AgentToHubMessageV1::Ack { v: PROTOCOL_VERSION, task_id: task_id.clone() };
                                    if tx.send(Message::Text(serde_json::to_string(&ack)?.into())).await.is_err() {
                                        return Ok(LoopAction::Reconnect);
                                    }

                                    if tx.send(Message::Text(serde_json::to_string(&cached)?.into())).await.is_err() {
                                        return Ok(LoopAction::Reconnect);
                                    }
                                    continue;
                                }

                                let ack = AgentToHubMessageV1::Ack { v: PROTOCOL_VERSION, task_id: task_id.clone() };
                                if tx.send(Message::Text(serde_json::to_string(&ack)?.into())).await.is_err() {
                                    return Ok(LoopAction::Reconnect);
                                }

                                let _guard = run_lock.lock().await;
                                match super::handle_backup_task(data_dir, &mut tx, &task_id, *task).await {
                                    Ok(()) => {}
                                    Err(error) => {
                                        if is_ws_error(&error) {
                                            warn!(
                                                task_id = %task_id,
                                                run_id = %run_id,
                                                error = %error,
                                                "task aborted due to websocket error; reconnecting"
                                            );
                                            return Ok(LoopAction::Reconnect);
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

                                        if tx.send(Message::Text(serde_json::to_string(&result)?.into())).await.is_err() {
                                            return Ok(LoopAction::Reconnect);
                                        }
                                    }
                                };
                            }
                            Ok(HubToAgentMessageV1::FsList { v, request_id, path }) if v == PROTOCOL_VERSION => {
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
                                if tx.send(Message::Text(serde_json::to_string(&msg)?.into())).await.is_err() {
                                    return Ok(LoopAction::Reconnect);
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(Message::Close(_)) => return Ok(LoopAction::Reconnect),
                    Ok(_) => {}
                    Err(_) => return Ok(LoopAction::Reconnect),
                }
            }
        }
    }
}
