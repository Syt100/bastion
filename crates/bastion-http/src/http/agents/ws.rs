use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ConnectInfo;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use sqlx::SqlitePool;
use tokio::sync::Notify;
use uuid::Uuid;

use bastion_core::agent_protocol::{
    AgentToHubMessageV1, ArtifactStreamOpenResultV1, HubToAgentMessageV1, PROTOCOL_VERSION,
};
use bastion_core::agent_stream::{
    ArtifactChunkFrameV1Flags, decode_artifact_chunk_frame_v1, encode_artifact_chunk_frame_v1,
};
use bastion_engine::agent_manager::AgentManager;
use bastion_engine::run_events;
use bastion_engine::run_events_bus::RunEventsBus;
use bastion_storage::agent_tasks_repo;
use bastion_storage::agents_repo;
use bastion_storage::artifact_delete_repo;
use bastion_storage::operations_repo;
use bastion_storage::run_artifacts_repo;
use bastion_storage::runs_repo;
use bastion_storage::secrets::SecretsCrypto;

use super::super::{AppError, AppState};
use super::agent_auth::authenticate_agent;
use super::snapshots::{send_node_config_snapshot, send_node_secrets_snapshot};
use super::stage_events;

mod artifact_stream;
mod artifact_stream_authz;

const ARTIFACT_STREAM_MAX_BYTES: usize = 1024 * 1024;
const ARTIFACT_STREAM_OPEN_TIMEOUT: Duration = Duration::from_secs(30);
const ARTIFACT_STREAM_PULL_TIMEOUT: Duration = Duration::from_secs(30);

const ARTIFACT_STREAM_AUTH_ERROR: &str = "artifact stream authorization failed";

pub(in crate::http) async fn agent_ws(
    state: axum::extract::State<AppState>,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let agent_id = authenticate_agent(&state.db, &headers).await?;

    let data_dir = state.config.data_dir.clone();
    let db = state.db.clone();
    let secrets = state.secrets.clone();
    let agent_manager = state.agent_manager.clone();
    let run_events_bus = state.run_events_bus.clone();
    let artifact_delete_notify = state.artifact_delete_notify.clone();
    Ok(ws.on_upgrade(move |socket| {
        handle_agent_socket(
            data_dir,
            db,
            agent_id,
            peer.ip(),
            secrets,
            agent_manager,
            run_events_bus,
            artifact_delete_notify,
            socket,
        )
    }))
}

#[allow(clippy::too_many_arguments)]
async fn handle_agent_socket(
    data_dir: PathBuf,
    db: SqlitePool,
    agent_id: String,
    peer_ip: std::net::IpAddr,
    secrets: Arc<SecretsCrypto>,
    agent_manager: AgentManager,
    run_events_bus: Arc<RunEventsBus>,
    artifact_delete_notify: Arc<Notify>,
    socket: WebSocket,
) {
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    if let Err(error) = sqlx::query("UPDATE agents SET last_seen_at = ? WHERE id = ?")
        .bind(now)
        .bind(&agent_id)
        .execute(&db)
        .await
    {
        tracing::warn!(agent_id = %agent_id, error = %error, "failed to update agent last_seen_at");
    }

    tracing::info!(agent_id = %agent_id, peer_ip = %peer_ip, "agent connected");

    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
    agent_manager.register(agent_id.clone(), tx).await;

    // Streams where *this agent* pulls bytes from the Hub (Hub acts as the stream server).
    // Keyed by stream_id (UUID).
    let mut hub_streams: HashMap<Uuid, artifact_stream::HubArtifactStream> = HashMap::new();

    // Best-effort stage tracking for progress snapshots (run_id -> last stage kind).
    let mut run_stage_cache: HashMap<String, String> = HashMap::new();

    // Send any pending tasks for this agent (reconnect-safe).
    match agent_tasks_repo::list_open_tasks_for_agent(&db, &agent_id, 100).await {
        Ok(tasks) => {
            for task in tasks {
                if let Ok(text) = serde_json::to_string(&task.payload) {
                    let _ = agent_manager
                        .send(&agent_id, Message::Text(text.into()))
                        .await;
                }
            }
        }
        Err(error) => {
            tracing::warn!(agent_id = %agent_id, error = %error, "failed to list pending tasks");
        }
    }

    let agent_id_send = agent_id.clone();
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                let text = text.to_string();
                let now = time::OffsetDateTime::now_utc().unix_timestamp();

                let _ = sqlx::query("UPDATE agents SET last_seen_at = ? WHERE id = ?")
                    .bind(now)
                    .bind(&agent_id)
                    .execute(&db)
                    .await;

                match serde_json::from_str::<AgentToHubMessageV1>(&text) {
                    Ok(AgentToHubMessageV1::Ping { v }) if v == PROTOCOL_VERSION => {
                        let _ = agent_manager
                            .send_json(&agent_id, &HubToAgentMessageV1::Pong { v })
                            .await;
                    }
                    Ok(AgentToHubMessageV1::Hello { v, .. }) if v == PROTOCOL_VERSION => {
                        // Store full hello payload for debugging/capabilities display.
                        let _ = sqlx::query(
                            "UPDATE agents SET capabilities_json = ?, last_seen_at = ? WHERE id = ?",
                        )
                        .bind(&text)
                        .bind(now)
                        .bind(&agent_id)
                        .execute(&db)
                        .await;

                        if let Err(error) =
                            send_node_secrets_snapshot(&db, &secrets, &agent_manager, &agent_id)
                                .await
                        {
                            tracing::warn!(
                                agent_id = %agent_id,
                                error = %error,
                                "failed to send secrets snapshot"
                            );
                        }

                        if let Err(error) =
                            send_node_config_snapshot(&db, &secrets, &agent_manager, &agent_id)
                                .await
                        {
                            tracing::warn!(
                                agent_id = %agent_id,
                                error = %error,
                                "failed to send config snapshot"
                            );
                        }
                    }
                    Ok(AgentToHubMessageV1::ConfigAck { v, snapshot_id })
                        if v == PROTOCOL_VERSION =>
                    {
                        tracing::info!(
                            agent_id = %agent_id,
                            snapshot_id = %snapshot_id,
                            "agent config snapshot ack"
                        );
                        if let Err(error) = agents_repo::record_applied_config_snapshot(
                            &db,
                            &agent_id,
                            &snapshot_id,
                        )
                        .await
                        {
                            tracing::warn!(
                                agent_id = %agent_id,
                                snapshot_id = %snapshot_id,
                                error = %error,
                                "failed to persist config snapshot ack"
                            );
                        }
                    }
                    Ok(AgentToHubMessageV1::Ack { v, task_id }) if v == PROTOCOL_VERSION => {
                        let _ = agent_tasks_repo::ack_task(&db, &task_id).await;
                    }
                    Ok(AgentToHubMessageV1::RunEvent {
                        v,
                        run_id,
                        level,
                        kind,
                        message,
                        fields,
                    }) if v == PROTOCOL_VERSION => {
                        if kind.trim() == bastion_core::progress::PROGRESS_SNAPSHOT_EVENT_KIND_V1 {
                            let _ = runs_repo::set_run_progress(&db, &run_id, fields).await;
                            stage_events::maybe_append_run_stage_event(
                                &db,
                                run_events_bus.as_ref(),
                                &mut run_stage_cache,
                                &run_id,
                                &message,
                            )
                            .await;
                            continue;
                        }

                        let _ = run_events::append_and_broadcast(
                            &db,
                            &run_events_bus,
                            &run_id,
                            &level,
                            &kind,
                            &message,
                            fields,
                        )
                        .await;
                    }
                    Ok(AgentToHubMessageV1::TaskResult {
                        v,
                        task_id,
                        run_id,
                        status,
                        summary,
                        error,
                    }) if v == PROTOCOL_VERSION => {
                        let run = runs_repo::get_run(&db, &run_id).await.ok().flatten();
                        if let Some(run) = run
                            && run.status == runs_repo::RunStatus::Running
                        {
                            let (run_status, err_code) = if status == "success" {
                                (runs_repo::RunStatus::Success, None)
                            } else {
                                let code = summary
                                    .as_ref()
                                    .and_then(|v| v.get("error_code"))
                                    .and_then(|v| v.as_str())
                                    .filter(|v| !v.trim().is_empty())
                                    .unwrap_or("agent_failed");
                                (runs_repo::RunStatus::Failed, Some(code))
                            };

                            let _ = runs_repo::complete_run(
                                &db,
                                &run_id,
                                run_status,
                                summary.clone(),
                                err_code,
                            )
                            .await;
                            if run_status == runs_repo::RunStatus::Success {
                                let _ =
                                    run_artifacts_repo::upsert_run_artifact_from_successful_run(
                                        &db, &run_id,
                                    )
                                    .await;
                            }
                            let _ = run_events::append_and_broadcast(
                                &db,
                                &run_events_bus,
                                &run_id,
                                if run_status == runs_repo::RunStatus::Success {
                                    "info"
                                } else {
                                    "error"
                                },
                                if run_status == runs_repo::RunStatus::Success {
                                    "complete"
                                } else {
                                    "failed"
                                },
                                if run_status == runs_repo::RunStatus::Success {
                                    "complete"
                                } else {
                                    "failed"
                                },
                                Some(serde_json::json!({ "agent_id": agent_id.clone() })),
                            )
                            .await;
                        }

                        let _ = agent_tasks_repo::complete_task(
                            &db,
                            &task_id,
                            summary.as_ref(),
                            error.as_deref(),
                        )
                        .await;
                    }
                    Ok(AgentToHubMessageV1::SnapshotDeleteEvent {
                        v,
                        run_id,
                        level,
                        kind,
                        message,
                        fields,
                    }) if v == PROTOCOL_VERSION => {
                        let _ = artifact_delete_repo::append_event(
                            &db, &run_id, &level, &kind, &message, fields, now,
                        )
                        .await;
                    }
                    Ok(AgentToHubMessageV1::SnapshotDeleteResult {
                        v,
                        run_id,
                        status,
                        error_kind,
                        error,
                    }) if v == PROTOCOL_VERSION => {
                        // Ensure this agent is the intended executor.
                        let task = artifact_delete_repo::get_task(&db, &run_id)
                            .await
                            .ok()
                            .flatten();
                        let Some(task) = task else {
                            continue;
                        };
                        if task.node_id != agent_id {
                            continue;
                        }

                        match status.as_str() {
                            "success" => {
                                let _ = artifact_delete_repo::mark_done(&db, &run_id, now).await;
                                let _ = run_artifacts_repo::mark_run_artifact_deleted(
                                    &db, &run_id, now,
                                )
                                .await;
                                let _ = artifact_delete_repo::append_event(
                                    &db,
                                    &run_id,
                                    "info",
                                    "done",
                                    "agent delete completed",
                                    Some(serde_json::json!({ "agent_id": agent_id.clone() })),
                                    now,
                                )
                                .await;
                                artifact_delete_notify.notify_one();
                            }
                            "not_found" => {
                                let _ = artifact_delete_repo::mark_done(&db, &run_id, now).await;
                                let _ = run_artifacts_repo::mark_run_artifact_missing(
                                    &db, &run_id, now,
                                )
                                .await;
                                let _ = artifact_delete_repo::append_event(
                                    &db,
                                    &run_id,
                                    "info",
                                    "done",
                                    "agent delete completed (not found)",
                                    Some(serde_json::json!({ "agent_id": agent_id.clone() })),
                                    now,
                                )
                                .await;
                                artifact_delete_notify.notify_one();
                            }
                            "failed" => {
                                let kind = normalize_delete_error_kind(error_kind.as_deref());
                                let msg = error.as_deref().unwrap_or("failed");
                                let last_error = sanitize_delete_error_string(msg);

                                if should_abandon_delete_task(task.attempts, task.created_at, now) {
                                    let _ = artifact_delete_repo::mark_abandoned(
                                        &db,
                                        &run_id,
                                        kind,
                                        &last_error,
                                        now,
                                    )
                                    .await;
                                    let _ = run_artifacts_repo::mark_run_artifact_error(
                                        &db,
                                        &run_id,
                                        kind,
                                        &last_error,
                                        now,
                                        now,
                                    )
                                    .await;
                                    let _ = artifact_delete_repo::append_event(
                                        &db,
                                        &run_id,
                                        "error",
                                        "abandoned",
                                        &format!("abandoned: {last_error}"),
                                        Some(serde_json::json!({ "agent_id": agent_id.clone(), "error_kind": kind })),
                                        now,
                                    )
                                    .await;
                                    continue;
                                }

                                let next_attempt_at = now.saturating_add(delete_backoff_seconds(
                                    &run_id,
                                    task.attempts,
                                    kind,
                                ));
                                if kind == "config" || kind == "auth" {
                                    let _ = artifact_delete_repo::mark_blocked(
                                        &db,
                                        &run_id,
                                        next_attempt_at,
                                        kind,
                                        &last_error,
                                        now,
                                    )
                                    .await;
                                    let _ = run_artifacts_repo::mark_run_artifact_error(
                                        &db,
                                        &run_id,
                                        kind,
                                        &last_error,
                                        now,
                                        now,
                                    )
                                    .await;
                                } else {
                                    let _ = artifact_delete_repo::mark_retrying(
                                        &db,
                                        &run_id,
                                        next_attempt_at,
                                        kind,
                                        &last_error,
                                        now,
                                    )
                                    .await;
                                    let _ =
                                        run_artifacts_repo::mark_run_artifact_deleting_with_error(
                                            &db,
                                            &run_id,
                                            kind,
                                            &last_error,
                                            now,
                                            now,
                                        )
                                        .await;
                                }

                                let _ = artifact_delete_repo::append_event(
                                    &db,
                                    &run_id,
                                    "warn",
                                    "failed",
                                    &format!("failed: {last_error}"),
                                    Some(serde_json::json!({ "agent_id": agent_id.clone(), "error_kind": kind, "next_attempt_at": next_attempt_at })),
                                    now,
                                )
                                .await;
                                artifact_delete_notify.notify_one();
                            }
                            _ => {}
                        }
                    }
                    Ok(AgentToHubMessageV1::FsListResult {
                        v,
                        request_id,
                        entries,
                        next_cursor,
                        total,
                        error_code,
                        error_details,
                        error,
                    }) if v == PROTOCOL_VERSION => {
                        let result = if let Some(message) = error {
                            Err(bastion_engine::agent_manager::FsListRemoteError {
                                code: error_code
                                    .unwrap_or_else(|| "error".to_string())
                                    .trim()
                                    .to_string(),
                                message: message.trim().to_string(),
                                details: error_details,
                            })
                        } else {
                            Ok(bastion_engine::agent_manager::FsListPage {
                                entries,
                                next_cursor,
                                total,
                            })
                        };
                        agent_manager
                            .complete_fs_list(&agent_id, &request_id, result)
                            .await;
                    }
                    Ok(AgentToHubMessageV1::WebdavListResult {
                        v,
                        request_id,
                        entries,
                        next_cursor,
                        total,
                        error_code,
                        error,
                    }) if v == PROTOCOL_VERSION => {
                        let result = if let Some(message) = error {
                            Err(bastion_engine::agent_manager::WebdavListRemoteError {
                                code: error_code
                                    .unwrap_or_else(|| "error".to_string())
                                    .trim()
                                    .to_string(),
                                message: message.trim().to_string(),
                            })
                        } else {
                            Ok(bastion_engine::agent_manager::WebdavListPage {
                                entries,
                                next_cursor,
                                total,
                            })
                        };
                        agent_manager
                            .complete_webdav_list(&agent_id, &request_id, result)
                            .await;
                    }
                    Ok(AgentToHubMessageV1::OperationEvent { v, event })
                        if v == PROTOCOL_VERSION =>
                    {
                        let bastion_core::agent_protocol::OperationEventV1 {
                            op_id,
                            level,
                            kind,
                            message,
                            fields,
                        } = event;

                        if kind.trim() == bastion_core::progress::PROGRESS_SNAPSHOT_EVENT_KIND_V1 {
                            let _ =
                                operations_repo::set_operation_progress(&db, &op_id, fields).await;
                            continue;
                        }

                        let _ = operations_repo::append_event(
                            &db, &op_id, &level, &kind, &message, fields,
                        )
                        .await;
                    }
                    Ok(AgentToHubMessageV1::OperationResult { v, result })
                        if v == PROTOCOL_VERSION =>
                    {
                        let status = if result.status.trim() == "success" {
                            operations_repo::OperationStatus::Success
                        } else {
                            operations_repo::OperationStatus::Failed
                        };

                        let _ = operations_repo::complete_operation(
                            &db,
                            &result.op_id,
                            status,
                            result.summary.clone(),
                            result.error.as_deref(),
                        )
                        .await;

                        let _ = agent_tasks_repo::complete_task(
                            &db,
                            &result.op_id,
                            result.summary.as_ref(),
                            result.error.as_deref(),
                        )
                        .await;
                    }
                    Ok(AgentToHubMessageV1::ArtifactStreamOpenResult { v, res })
                        if v == PROTOCOL_VERSION =>
                    {
                        agent_manager
                            .complete_artifact_stream_open(&agent_id, res)
                            .await;
                    }
                    Ok(AgentToHubMessageV1::ArtifactStreamOpen { v, req })
                        if v == PROTOCOL_VERSION =>
                    {
                        let Ok(stream_id) = Uuid::parse_str(req.stream_id.trim()) else {
                            continue;
                        };

                        if let Err(error) =
                            artifact_stream_authz::authorize_agent_artifact_stream_open(
                                &db, &agent_id, &req,
                            )
                            .await
                        {
                            tracing::warn!(
                                agent_id = %agent_id,
                                op_id = %req.op_id,
                                run_id = %req.run_id,
                                error = %error,
                                "artifact stream open denied"
                            );

                            let msg = HubToAgentMessageV1::ArtifactStreamOpenResult {
                                v: PROTOCOL_VERSION,
                                res: ArtifactStreamOpenResultV1 {
                                    stream_id: req.stream_id,
                                    size: None,
                                    error: Some(ARTIFACT_STREAM_AUTH_ERROR.to_string()),
                                },
                            };
                            let _ = agent_manager.send_json(&agent_id, &msg).await;
                            continue;
                        }

                        // Best-effort: close any previous stream with the same id.
                        if let Some(prev) = hub_streams.remove(&stream_id)
                            && let Some(dir) = prev.cleanup_dir
                        {
                            let _ = tokio::fs::remove_dir_all(dir).await;
                        }

                        let opened = artifact_stream::open_hub_artifact_stream(
                            &data_dir,
                            &db,
                            &secrets,
                            &agent_manager,
                            &req,
                            stream_id,
                        )
                        .await;

                        let (size, error) = match opened {
                            Ok((stream, size)) => {
                                hub_streams.insert(stream_id, stream);
                                (size, None)
                            }
                            Err(err) => (None, Some(err.to_string())),
                        };

                        let msg = HubToAgentMessageV1::ArtifactStreamOpenResult {
                            v: PROTOCOL_VERSION,
                            res: ArtifactStreamOpenResultV1 {
                                stream_id: req.stream_id,
                                size,
                                error,
                            },
                        };
                        let _ = agent_manager.send_json(&agent_id, &msg).await;
                    }
                    Ok(AgentToHubMessageV1::ArtifactStreamPull { v, req })
                        if v == PROTOCOL_VERSION =>
                    {
                        let Ok(stream_id) = Uuid::parse_str(req.stream_id.trim()) else {
                            continue;
                        };
                        let Some(stream) = hub_streams.get(&stream_id) else {
                            continue;
                        };

                        let max_bytes =
                            (req.max_bytes as usize).clamp(1, ARTIFACT_STREAM_MAX_BYTES);
                        let reader = stream.reader.clone();
                        let read_res = tokio::task::spawn_blocking(move || {
                            let mut buf = vec![0u8; max_bytes];
                            let mut guard = reader.lock().map_err(|_| {
                                std::io::Error::other("stream reader lock poisoned")
                            })?;
                            let n = guard.read(&mut buf)?;
                            buf.truncate(n);
                            Ok::<_, std::io::Error>(buf)
                        })
                        .await;

                        let bytes = match read_res {
                            Ok(Ok(bytes)) => bytes,
                            Ok(Err(error)) => {
                                tracing::warn!(
                                    agent_id = %agent_id,
                                    stream_id = %stream_id,
                                    error = %error,
                                    "artifact stream read failed"
                                );
                                if let Some(prev) = hub_streams.remove(&stream_id)
                                    && let Some(dir) = prev.cleanup_dir
                                {
                                    let _ = tokio::fs::remove_dir_all(dir).await;
                                }
                                continue;
                            }
                            Err(error) => {
                                tracing::warn!(
                                    agent_id = %agent_id,
                                    stream_id = %stream_id,
                                    error = %error,
                                    "artifact stream read task failed"
                                );
                                continue;
                            }
                        };

                        let eof = bytes.is_empty();
                        let frame = encode_artifact_chunk_frame_v1(
                            &stream_id,
                            ArtifactChunkFrameV1Flags { eof },
                            &bytes,
                        );
                        let _ = agent_manager
                            .send(&agent_id, Message::Binary(frame.into()))
                            .await;

                        if eof
                            && let Some(prev) = hub_streams.remove(&stream_id)
                            && let Some(dir) = prev.cleanup_dir
                        {
                            let _ = tokio::fs::remove_dir_all(dir).await;
                        }
                    }
                    Ok(AgentToHubMessageV1::ArtifactStreamClose { v, req })
                        if v == PROTOCOL_VERSION =>
                    {
                        if let Ok(stream_id) = Uuid::parse_str(req.stream_id.trim())
                            && let Some(prev) = hub_streams.remove(&stream_id)
                            && let Some(dir) = prev.cleanup_dir
                        {
                            let _ = tokio::fs::remove_dir_all(dir).await;
                        }
                    }
                    _ => {}
                }
            }
            Message::Binary(bytes) => {
                if let Ok(decoded) = decode_artifact_chunk_frame_v1(&bytes) {
                    agent_manager
                        .complete_artifact_stream_chunk(
                            &agent_id,
                            decoded.stream_id,
                            bastion_engine::agent_manager::ArtifactChunk {
                                eof: decoded.flags.eof,
                                bytes: decoded.payload.to_vec(),
                            },
                        )
                        .await;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    agent_manager.unregister(&agent_id_send).await;
    send_task.abort();

    tracing::info!(agent_id = %agent_id, "agent disconnected");
}

fn normalize_delete_error_kind(kind: Option<&str>) -> &'static str {
    match kind.unwrap_or("").trim() {
        "config" => "config",
        "auth" => "auth",
        "network" => "network",
        "http" => "http",
        _ => "unknown",
    }
}

fn sanitize_delete_error_string(s: &str) -> String {
    const MAX_LEN: usize = 500;

    let s = s.replace(['\n', '\r'], " ");
    let s = s.trim();
    if s.len() <= MAX_LEN {
        return s.to_string();
    }

    let mut out = s[..MAX_LEN].to_string();
    out.push('…');
    out
}

fn should_abandon_delete_task(attempts: i64, created_at: i64, now: i64) -> bool {
    const MAX_ATTEMPTS: i64 = 20;
    const MAX_AGE_SECS: i64 = 30 * 24 * 60 * 60;

    if attempts >= MAX_ATTEMPTS {
        return true;
    }

    let age = now.saturating_sub(created_at);
    age >= MAX_AGE_SECS
}

fn delete_backoff_seconds(run_id: &str, attempts: i64, kind: &str) -> i64 {
    let attempts = attempts.max(1);

    let (base, cap, max_jitter) = match kind {
        "network" | "http" => (60_i64, 6 * 60 * 60, 30_i64),
        "unknown" => (5 * 60, 6 * 60 * 60, 60_i64),
        "auth" | "config" => (6 * 60 * 60, 24 * 60 * 60, 10 * 60_i64),
        _ => (5 * 60, 6 * 60 * 60, 60_i64),
    };

    let exp = 1_i64 << (attempts.saturating_sub(1).min(30) as u32);
    let delay = base.saturating_mul(exp).min(cap);
    delay.saturating_add(delete_jitter_seconds(run_id, attempts, max_jitter))
}

fn delete_jitter_seconds(run_id: &str, attempts: i64, max_jitter: i64) -> i64 {
    if max_jitter <= 0 {
        return 0;
    }

    let mut hash = 0_u64;
    for byte in run_id.as_bytes() {
        hash = hash.wrapping_mul(131).wrapping_add(*byte as u64);
    }
    hash = hash.wrapping_add(attempts as u64 * 97);
    (hash % max_jitter as u64) as i64
}
