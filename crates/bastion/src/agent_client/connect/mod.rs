mod handlers;
mod handshake;
mod heartbeat;

use std::path::Path;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tracing::warn;
use url::Url;

use bastion_core::agent_protocol::{HubToAgentMessageV1, PROTOCOL_VERSION};

use super::identity::AgentIdentityV1;
use super::offline;
use super::util::normalize_base_url;

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

    let _connected_guard = handshake::ConnectedGuard::new(connected_tx.clone());
    handshake::send_hello(&mut tx, identity).await?;

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
                let _ = tx.send(heartbeat::close_message()).await;
                return Ok(LoopAction::Exit);
            }
            _ = tick.tick() => {
                if heartbeat::pong_timed_out(&last_pong, pong_timeout) {
                    warn!(
                        agent_id = %identity.agent_id,
                        timeout_seconds = pong_timeout.as_secs(),
                        "pong timeout; reconnecting"
                    );
                    return Ok(LoopAction::Reconnect);
                }

                let ping = heartbeat::ping_message()?;
                if tx.send(ping).await.is_err() {
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
                            Ok(HubToAgentMessageV1::SecretsSnapshot { v, node_id, issued_at, webdav })
                                if v == PROTOCOL_VERSION =>
                            {
                                if handlers::handle_secrets_snapshot(identity, data_dir, node_id, issued_at, webdav).await
                                    == handlers::HandlerFlow::Reconnect
                                {
                                    return Ok(LoopAction::Reconnect);
                                }
                            }
                            Ok(HubToAgentMessageV1::ConfigSnapshot {
                                v,
                                node_id,
                                snapshot_id,
                                issued_at,
                                jobs,
                            }) if v == PROTOCOL_VERSION => {
                                if handlers::handle_config_snapshot(
                                    &mut tx,
                                    identity,
                                    data_dir,
                                    node_id,
                                    snapshot_id,
                                    issued_at,
                                    jobs,
                                )
                                .await?
                                    == handlers::HandlerFlow::Reconnect
                                {
                                    return Ok(LoopAction::Reconnect);
                                }
                            }
                            Ok(HubToAgentMessageV1::Task { v, task_id, task }) if v == PROTOCOL_VERSION => {
                                if handlers::handle_task(
                                    &mut tx,
                                    data_dir,
                                    run_lock.clone(),
                                    task_id,
                                    task,
                                )
                                .await?
                                    == handlers::HandlerFlow::Reconnect
                                {
                                    return Ok(LoopAction::Reconnect);
                                }
                            }
                            Ok(HubToAgentMessageV1::FsList {
                                v,
                                request_id,
                                path,
                                cursor,
                                limit,
                                q,
                                kind,
                                hide_dotfiles,
                                type_sort,
                                size_min_bytes,
                                size_max_bytes,
                            }) if v == PROTOCOL_VERSION => {
                                if handlers::handle_fs_list(
                                    &mut tx,
                                    request_id,
                                    path,
                                    cursor,
                                    limit,
                                    q,
                                    kind,
                                    hide_dotfiles,
                                    type_sort,
                                    size_min_bytes,
                                    size_max_bytes,
                                )
                                .await?
                                    == handlers::HandlerFlow::Reconnect
                                {
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
