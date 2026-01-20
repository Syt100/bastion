mod handlers;
mod handshake;
mod heartbeat;

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use futures_util::{Sink, SinkExt, StreamExt};
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tracing::warn;
use url::Url;
use uuid::Uuid;

use bastion_core::agent_protocol::{
    AgentToHubMessageV1, ArtifactStreamOpenResultV1, HubToAgentMessageV1, PROTOCOL_VERSION,
};
use bastion_core::agent_stream::{
    ArtifactChunkFrameV1Flags, decode_artifact_chunk_frame_v1, encode_artifact_chunk_frame_v1,
};

use super::hub_stream::{HubStreamChunk, HubStreamManager};
use super::identity::AgentIdentityV1;
use super::offline;
use super::util::normalize_base_url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LoopAction {
    Reconnect,
    Exit,
}

#[derive(Clone)]
struct OutboxSink {
    tx: mpsc::UnboundedSender<Message>,
}

impl Sink<Message> for OutboxSink {
    type Error = tungstenite::Error;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        self.tx
            .send(item)
            .map_err(|_| tungstenite::Error::ConnectionClosed)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
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
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Outbox so long-running tasks can keep the main receive loop responsive (heartbeats + streams).
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Message>();
    let send_task = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut tx = OutboxSink { tx: out_tx.clone() };
    let data_dir = data_dir.to_path_buf();
    let hub_streams = HubStreamManager::new(out_tx.clone());
    let (force_reconnect_tx, mut force_reconnect_rx) = mpsc::unbounded_channel::<()>();

    let _connected_guard = handshake::ConnectedGuard::new(connected_tx.clone());
    handshake::send_hello(&mut tx, identity).await?;

    if let Ok(base_url) = normalize_base_url(&identity.hub_url)
        && let Err(error) =
            offline::sync_offline_runs(&base_url, &identity.agent_key, &data_dir).await
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

    // Streams where the Hub pulls bytes from this Agent (the Agent serves file bytes).
    let mut hub_pull_streams: HashMap<Uuid, tokio::fs::File> = HashMap::new();

    let action = 'main: loop {
        tokio::select! {
            _ = &mut shutdown => {
                let _ = tx.send(heartbeat::close_message()).await;
                break 'main LoopAction::Exit;
            }
            _ = tick.tick() => {
                if heartbeat::pong_timed_out(&last_pong, pong_timeout) {
                    warn!(
                        agent_id = %identity.agent_id,
                        timeout_seconds = pong_timeout.as_secs(),
                        "pong timeout; reconnecting"
                    );
                    break 'main LoopAction::Reconnect;
                }

                let ping = heartbeat::ping_message()?;
                if tx.send(ping).await.is_err() {
                    break 'main LoopAction::Reconnect;
                }
            }
            Some(_) = force_reconnect_rx.recv() => {
                break 'main LoopAction::Reconnect;
            }
            msg = ws_rx.next() => {
                let Some(msg) = msg else {
                    break 'main LoopAction::Reconnect;
                };

                match msg {
                    Ok(Message::Text(text)) => {
                        let text = text.to_string();
                        match serde_json::from_str::<HubToAgentMessageV1>(&text) {
                            Ok(HubToAgentMessageV1::Pong { .. }) => {
                                last_pong = tokio::time::Instant::now();
                            }
                            Ok(HubToAgentMessageV1::SecretsSnapshot { v, node_id, issued_at, webdav, backup_age_identities })
                                if v == PROTOCOL_VERSION =>
                            {
                                if handlers::handle_secrets_snapshot(identity, &data_dir, node_id, issued_at, webdav, backup_age_identities).await
                                    == handlers::HandlerFlow::Reconnect
                                {
                                    break 'main LoopAction::Reconnect;
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
                                    &data_dir,
                                    node_id,
                                    snapshot_id,
                                    issued_at,
                                    jobs,
                                )
                                .await?
                                    == handlers::HandlerFlow::Reconnect
                                {
                                    break 'main LoopAction::Reconnect;
                                }
                            }
                            Ok(HubToAgentMessageV1::Task { v, task_id, task }) if v == PROTOCOL_VERSION => {
                                let out_tx = out_tx.clone();
                                let data_dir = data_dir.clone();
                                let run_lock = run_lock.clone();
                                let force_reconnect_tx = force_reconnect_tx.clone();
                                tokio::spawn(async move {
                                    let mut tx = OutboxSink { tx: out_tx };
                                    let flow = handlers::handle_task(
                                        &mut tx,
                                        &data_dir,
                                        run_lock,
                                        task_id,
                                        task,
                                    )
                                    .await;
                                    match flow {
                                        Ok(handlers::HandlerFlow::Continue) => {}
                                        Ok(handlers::HandlerFlow::Reconnect) => {
                                            let _ = force_reconnect_tx.send(());
                                        }
                                        Err(error) => {
                                            warn!(error = %error, "task handler failed");
                                            let _ = force_reconnect_tx.send(());
                                        }
                                    }
                                });
                            }
                            Ok(HubToAgentMessageV1::RestoreTask { v, task_id, task }) if v == PROTOCOL_VERSION => {
                                let out_tx = out_tx.clone();
                                let data_dir = data_dir.clone();
                                let run_lock = run_lock.clone();
                                let hub_streams = hub_streams.clone();
                                let force_reconnect_tx = force_reconnect_tx.clone();
                                tokio::spawn(async move {
                                    let mut tx = OutboxSink { tx: out_tx };
                                    let flow = handlers::handle_restore_task(
                                        &mut tx,
                                        &data_dir,
                                        run_lock,
                                        &hub_streams,
                                        task_id,
                                        task,
                                    )
                                    .await;
                                    match flow {
                                        Ok(handlers::HandlerFlow::Continue) => {}
                                        Ok(handlers::HandlerFlow::Reconnect) => {
                                            let _ = force_reconnect_tx.send(());
                                        }
                                        Err(error) => {
                                            warn!(error = %error, "restore task handler failed");
                                            let _ = force_reconnect_tx.send(());
                                        }
                                    }
                                });
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
                                sort_by,
                                sort_dir,
                                size_min_bytes,
                                size_max_bytes,
                            }) if v == PROTOCOL_VERSION => {
                                let out_tx = out_tx.clone();
                                let force_reconnect_tx = force_reconnect_tx.clone();
                                tokio::spawn(async move {
                                    let mut tx = OutboxSink { tx: out_tx };
                                    let flow = handlers::handle_fs_list(
                                        &mut tx,
                                        handlers::FsListRequest {
                                            request_id,
                                            path,
                                            cursor,
                                            limit,
                                            q,
                                            kind,
                                            hide_dotfiles,
                                            type_sort,
                                            sort_by,
                                            sort_dir,
                                            size_min_bytes,
                                            size_max_bytes,
                                        },
                                    )
                                    .await;
                                    match flow {
                                        Ok(handlers::HandlerFlow::Continue) => {}
                                        Ok(handlers::HandlerFlow::Reconnect) => {
                                            let _ = force_reconnect_tx.send(());
                                        }
                                        Err(error) => {
                                            warn!(error = %error, "fs list handler failed");
                                            let _ = force_reconnect_tx.send(());
                                        }
                                    }
                                });
                            }
                            Ok(HubToAgentMessageV1::WebdavList {
                                v,
                                request_id,
                                base_url,
                                secret_name,
                                path,
                                cursor,
                                limit,
                                q,
                                kind,
                                hide_dotfiles,
                                type_sort,
                                sort_by,
                                sort_dir,
                                size_min_bytes,
                                size_max_bytes,
                            }) if v == PROTOCOL_VERSION => {
                                let out_tx = out_tx.clone();
                                let data_dir = data_dir.clone();
                                let force_reconnect_tx = force_reconnect_tx.clone();
                                tokio::spawn(async move {
                                    let mut tx = OutboxSink { tx: out_tx };
                                    let flow = handlers::handle_webdav_list(
                                        &mut tx,
                                        &data_dir,
                                        handlers::WebdavListRequest {
                                            request_id,
                                            base_url,
                                            secret_name,
                                            path,
                                            cursor,
                                            limit,
                                            q,
                                            kind,
                                            hide_dotfiles,
                                            type_sort,
                                            sort_by,
                                            sort_dir,
                                            size_min_bytes,
                                            size_max_bytes,
                                        },
                                    )
                                    .await;
                                    match flow {
                                        Ok(handlers::HandlerFlow::Continue) => {}
                                        Ok(handlers::HandlerFlow::Reconnect) => {
                                            let _ = force_reconnect_tx.send(());
                                        }
                                        Err(error) => {
                                            warn!(error = %error, "webdav list handler failed");
                                            let _ = force_reconnect_tx.send(());
                                        }
                                    }
                                });
                            }
                            Ok(HubToAgentMessageV1::ArtifactStreamOpenResult { v, res }) if v == PROTOCOL_VERSION => {
                                hub_streams.complete_open(res).await;
                            }
                            Ok(HubToAgentMessageV1::ArtifactStreamOpen { v, req })
                                if v == PROTOCOL_VERSION =>
                            {
                                let stream_id = match Uuid::parse_str(req.stream_id.trim()) {
                                    Ok(id) => id,
                                    Err(_) => continue,
                                };

                                let (size, error) = match req.path.as_deref().map(str::trim) {
                                    Some(path) if !path.is_empty() => {
                                        match tokio::fs::metadata(path).await {
                                            Ok(meta) => match tokio::fs::File::open(path).await {
                                                Ok(file) => {
                                                    hub_pull_streams.insert(stream_id, file);
                                                    (Some(meta.len()), None)
                                                }
                                                Err(e) => (None, Some(e.to_string())),
                                            },
                                            Err(e) => (None, Some(e.to_string())),
                                        }
                                    }
                                    _ => (None, Some("missing path".to_string())),
                                };

                                let res = ArtifactStreamOpenResultV1 {
                                    stream_id: req.stream_id,
                                    size,
                                    error,
                                };
                                let msg = AgentToHubMessageV1::ArtifactStreamOpenResult {
                                    v: PROTOCOL_VERSION,
                                    res,
                                };
                                if tx
                                    .send(Message::Text(serde_json::to_string(&msg)?.into()))
                                    .await
                                    .is_err()
                                {
                                    break 'main LoopAction::Reconnect;
                                }
                            }
                            Ok(HubToAgentMessageV1::ArtifactStreamPull { v, req })
                                if v == PROTOCOL_VERSION =>
                            {
                                let stream_id = match Uuid::parse_str(req.stream_id.trim()) {
                                    Ok(id) => id,
                                    Err(_) => continue,
                                };
                                let Some(file) = hub_pull_streams.get_mut(&stream_id) else {
                                    continue;
                                };

                                let max_bytes = req.max_bytes.clamp(1, 1024 * 1024) as usize;
                                let mut buf = vec![0u8; max_bytes];
                                let n = match file.read(&mut buf).await {
                                    Ok(n) => n,
                                    Err(_) => {
                                        let _ = hub_pull_streams.remove(&stream_id);
                                        continue;
                                    }
                                };
                                buf.truncate(n);
                                let eof = n == 0;

                                let frame = encode_artifact_chunk_frame_v1(
                                    &stream_id,
                                    ArtifactChunkFrameV1Flags { eof },
                                    &buf,
                                );
                                if tx.send(Message::Binary(frame.into())).await.is_err() {
                                    break 'main LoopAction::Reconnect;
                                }
                                if eof {
                                    let _ = hub_pull_streams.remove(&stream_id);
                                }
                            }
                            Ok(HubToAgentMessageV1::ArtifactStreamClose { v, req })
                                if v == PROTOCOL_VERSION =>
                            {
                                if let Ok(stream_id) = Uuid::parse_str(req.stream_id.trim()) {
                                    let _ = hub_pull_streams.remove(&stream_id);
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(Message::Binary(bytes)) => {
                        if let Ok(decoded) = decode_artifact_chunk_frame_v1(&bytes) {
                            hub_streams
                                .complete_chunk(
                                HubStreamChunk {
                                    eof: decoded.flags.eof,
                                    bytes: decoded.payload.to_vec(),
                                },
                                decoded.stream_id,
                            )
                            .await;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        break 'main LoopAction::Reconnect;
                    }
                    Ok(_) => {}
                    Err(_) => {
                        break 'main LoopAction::Reconnect;
                    }
                }
            }
        }
    };

    send_task.abort();
    Ok(action)
}
