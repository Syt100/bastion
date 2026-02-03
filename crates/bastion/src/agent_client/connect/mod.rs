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
    AgentToHubMessageV1, ArtifactStreamOpenResultV1, ArtifactStreamOpenV1, ArtifactStreamPullV1,
    HubToAgentMessageV1, PROTOCOL_VERSION,
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
                            Ok(HubToAgentMessageV1::SnapshotDeleteTask { v, task }) if v == PROTOCOL_VERSION => {
                                let out_tx = out_tx.clone();
                                let run_lock = run_lock.clone();
                                let force_reconnect_tx = force_reconnect_tx.clone();
                                tokio::spawn(async move {
                                    let mut tx = OutboxSink { tx: out_tx };
                                    let flow = handlers::handle_snapshot_delete_task(&mut tx, run_lock, task).await;
                                    match flow {
                                        Ok(handlers::HandlerFlow::Continue) => {}
                                        Ok(handlers::HandlerFlow::Reconnect) => {
                                            let _ = force_reconnect_tx.send(());
                                        }
                                        Err(error) => {
                                            warn!(error = %error, "snapshot delete handler failed");
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
                                let Some(res) =
                                    build_artifact_stream_open_result(&mut hub_pull_streams, req)
                                        .await
                                else {
                                    continue;
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
                                let Some(frame) =
                                    build_artifact_stream_pull_frame(&mut hub_pull_streams, &req)
                                        .await
                                else {
                                    continue;
                                };

                                if tx.send(Message::Binary(frame.into())).await.is_err() {
                                    break 'main LoopAction::Reconnect;
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

async fn build_artifact_stream_open_result(
    hub_pull_streams: &mut HashMap<Uuid, tokio::fs::File>,
    req: ArtifactStreamOpenV1,
) -> Option<ArtifactStreamOpenResultV1> {
    let ArtifactStreamOpenV1 {
        stream_id,
        path,
        op_id: _,
        run_id: _,
        artifact: _,
    } = req;

    let stream_uuid = match Uuid::parse_str(stream_id.trim()) {
        Ok(id) => id,
        Err(_) => return None,
    };

    let (size, error) = match path.as_deref().map(str::trim) {
        Some(path) if !path.is_empty() => match tokio::fs::metadata(path).await {
            Ok(meta) => match tokio::fs::File::open(path).await {
                Ok(file) => {
                    hub_pull_streams.insert(stream_uuid, file);
                    (Some(meta.len()), None)
                }
                Err(e) => (None, Some(e.to_string())),
            },
            Err(e) => (None, Some(e.to_string())),
        },
        _ => (None, Some("missing path".to_string())),
    };

    Some(ArtifactStreamOpenResultV1 {
        stream_id,
        size,
        error,
    })
}

async fn build_artifact_stream_pull_frame(
    hub_pull_streams: &mut HashMap<Uuid, tokio::fs::File>,
    req: &ArtifactStreamPullV1,
) -> Option<Vec<u8>> {
    let stream_id = match Uuid::parse_str(req.stream_id.trim()) {
        Ok(id) => id,
        Err(_) => return None,
    };
    let file = hub_pull_streams.get_mut(&stream_id)?;

    let max_bytes = req.max_bytes.clamp(1, 1024 * 1024) as usize;
    let mut buf = vec![0u8; max_bytes];
    let n = match file.read(&mut buf).await {
        Ok(n) => n,
        Err(_) => {
            let _ = hub_pull_streams.remove(&stream_id);
            return None;
        }
    };
    buf.truncate(n);
    let eof = n == 0;

    let frame = encode_artifact_chunk_frame_v1(&stream_id, ArtifactChunkFrameV1Flags { eof }, &buf);
    if eof {
        let _ = hub_pull_streams.remove(&stream_id);
    }
    Some(frame)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn outbox_sink_forwards_messages_to_receiver() -> Result<(), anyhow::Error> {
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        let mut sink = OutboxSink { tx };

        sink.send(Message::Text("hello".into()))
            .await
            .map_err(anyhow::Error::from)?;

        let msg = rx.recv().await.expect("receive");
        assert!(matches!(msg, Message::Text(_)));
        Ok(())
    }

    #[tokio::test]
    async fn outbox_sink_returns_connection_closed_when_receiver_dropped() {
        let (tx, rx) = mpsc::unbounded_channel::<Message>();
        drop(rx);

        let mut sink = OutboxSink { tx };
        let err = sink
            .send(Message::Text("hello".into()))
            .await
            .expect_err("send should fail");

        assert!(matches!(err, tungstenite::Error::ConnectionClosed));
    }

    #[tokio::test]
    async fn artifact_stream_open_missing_path_reports_error_and_does_not_insert() {
        let mut streams = HashMap::<Uuid, tokio::fs::File>::new();
        let req = ArtifactStreamOpenV1 {
            stream_id: Uuid::new_v4().to_string(),
            op_id: "op".to_string(),
            run_id: "run".to_string(),
            artifact: "artifact".to_string(),
            path: None,
        };

        let res = build_artifact_stream_open_result(&mut streams, req)
            .await
            .expect("result");
        assert_eq!(res.size, None);
        assert_eq!(res.error.as_deref(), Some("missing path"));
        assert!(streams.is_empty());
    }

    #[tokio::test]
    async fn artifact_stream_open_existing_file_inserts_and_reports_size()
    -> Result<(), anyhow::Error> {
        let dir = tempdir()?;
        let file_path = dir.path().join("artifact.bin");
        let contents = b"hello world";
        std::fs::write(&file_path, contents)?;

        let stream_id = Uuid::new_v4().to_string();
        let mut streams = HashMap::<Uuid, tokio::fs::File>::new();
        let req = ArtifactStreamOpenV1 {
            stream_id: stream_id.clone(),
            op_id: "op".to_string(),
            run_id: "run".to_string(),
            artifact: "artifact".to_string(),
            path: Some(file_path.to_string_lossy().to_string()),
        };

        let res = build_artifact_stream_open_result(&mut streams, req)
            .await
            .expect("result");
        assert_eq!(res.stream_id, stream_id);
        assert_eq!(res.size, Some(contents.len() as u64));
        assert_eq!(res.error, None);

        let uuid = Uuid::parse_str(res.stream_id.trim())?;
        assert!(streams.contains_key(&uuid));
        Ok(())
    }

    #[tokio::test]
    async fn artifact_stream_pull_sends_chunks_and_eof_then_removes() -> Result<(), anyhow::Error> {
        let dir = tempdir()?;
        let file_path = dir.path().join("artifact.bin");
        let contents = b"abcdef";
        std::fs::write(&file_path, contents)?;

        let stream_id = Uuid::new_v4();
        let file = tokio::fs::File::open(&file_path).await?;

        let mut streams = HashMap::<Uuid, tokio::fs::File>::new();
        streams.insert(stream_id, file);

        // Pull 2 bytes at a time.
        let req = ArtifactStreamPullV1 {
            stream_id: stream_id.to_string(),
            max_bytes: 2,
        };

        let frame = build_artifact_stream_pull_frame(&mut streams, &req)
            .await
            .expect("first frame");
        let decoded = decode_artifact_chunk_frame_v1(&frame)?;
        assert_eq!(decoded.stream_id, stream_id);
        assert!(!decoded.flags.eof);
        assert_eq!(decoded.payload, b"ab");

        let frame = build_artifact_stream_pull_frame(&mut streams, &req)
            .await
            .expect("second frame");
        let decoded = decode_artifact_chunk_frame_v1(&frame)?;
        assert!(!decoded.flags.eof);
        assert_eq!(decoded.payload, b"cd");

        let frame = build_artifact_stream_pull_frame(&mut streams, &req)
            .await
            .expect("third frame");
        let decoded = decode_artifact_chunk_frame_v1(&frame)?;
        assert!(!decoded.flags.eof);
        assert_eq!(decoded.payload, b"ef");

        // EOF is sent as a final empty frame and stream is removed.
        let frame = build_artifact_stream_pull_frame(&mut streams, &req)
            .await
            .expect("eof frame");
        let decoded = decode_artifact_chunk_frame_v1(&frame)?;
        assert!(decoded.flags.eof);
        assert_eq!(decoded.payload, b"");
        assert!(!streams.contains_key(&stream_id));
        Ok(())
    }
}
