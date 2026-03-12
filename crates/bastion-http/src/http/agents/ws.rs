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
use bastion_core::error_envelope::{
    ErrorEnvelopeV1, ErrorOriginV1, ErrorRetriableV1, ErrorTransportV1, LocalizedTextRefV1,
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
const AGENT_WS_OUTBOX_CAPACITY: usize = 512;
const AGENT_LAST_SEEN_MIN_UPDATE_SECS: i64 = 10;

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
            AgentSocketContext {
                data_dir,
                db,
                agent_id,
                peer_ip: peer.ip(),
                secrets,
                agent_manager,
                run_events_bus,
                artifact_delete_notify,
            },
            socket,
        )
    }))
}

struct AgentSocketContext {
    data_dir: PathBuf,
    db: SqlitePool,
    agent_id: String,
    peer_ip: std::net::IpAddr,
    secrets: Arc<SecretsCrypto>,
    agent_manager: AgentManager,
    run_events_bus: Arc<RunEventsBus>,
    artifact_delete_notify: Arc<Notify>,
}

async fn handle_agent_socket(ctx: AgentSocketContext, socket: WebSocket) {
    let AgentSocketContext {
        data_dir,
        db,
        agent_id,
        peer_ip,
        secrets,
        agent_manager,
        run_events_bus,
        artifact_delete_notify,
    } = ctx;
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    if let Err(error) = sqlx::query("UPDATE agents SET last_seen_at = ? WHERE id = ?")
        .bind(now)
        .bind(&agent_id)
        .execute(&db)
        .await
    {
        tracing::warn!(agent_id = %agent_id, error = %error, "failed to update agent last_seen_at");
    }
    let mut last_seen_persisted_at = now;

    tracing::info!(agent_id = %agent_id, peer_ip = %peer_ip, "agent connected");

    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(AGENT_WS_OUTBOX_CAPACITY);
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
                let payload = task.payload.clone();
                if let Ok(text) = serde_json::to_string(&payload) {
                    let _ = agent_manager
                        .send(&agent_id, Message::Text(text.into()))
                        .await;
                }

                if let Ok(msg) = serde_json::from_value::<HubToAgentMessageV1>(payload) {
                    match msg {
                        HubToAgentMessageV1::Task { task, .. } => {
                            if let Ok(Some(run)) = runs_repo::get_run(&db, &task.run_id).await
                                && run.cancel_requested_at.is_some()
                                && run.status == runs_repo::RunStatus::Running
                            {
                                let _ = agent_manager
                                    .send_json(
                                        &agent_id,
                                        &HubToAgentMessageV1::CancelRunTask {
                                            v: PROTOCOL_VERSION,
                                            run_id: task.run_id,
                                        },
                                    )
                                    .await;
                            }
                        }
                        HubToAgentMessageV1::RestoreTask { task, .. } => {
                            if let Ok(Some(op)) =
                                operations_repo::get_operation(&db, &task.op_id).await
                                && op.cancel_requested_at.is_some()
                                && op.status == operations_repo::OperationStatus::Running
                            {
                                let _ = agent_manager
                                    .send_json(
                                        &agent_id,
                                        &HubToAgentMessageV1::CancelOperationTask {
                                            v: PROTOCOL_VERSION,
                                            op_id: task.op_id,
                                        },
                                    )
                                    .await;
                            }
                        }
                        _ => {}
                    }
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

                if should_persist_agent_last_seen(last_seen_persisted_at, now) {
                    if let Err(error) =
                        sqlx::query("UPDATE agents SET last_seen_at = ? WHERE id = ?")
                            .bind(now)
                            .bind(&agent_id)
                            .execute(&db)
                            .await
                    {
                        tracing::warn!(
                            agent_id = %agent_id,
                            error = %error,
                            "failed to update agent last_seen_at"
                        );
                    } else {
                        last_seen_persisted_at = now;
                    }
                }

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
                            let (run_status, err_code) = match status.trim() {
                                "success" => (runs_repo::RunStatus::Success, None),
                                "canceled" => (runs_repo::RunStatus::Canceled, Some("canceled")),
                                _ => {
                                    let code = summary
                                        .as_ref()
                                        .and_then(|v| v.get("error_code"))
                                        .and_then(|v| v.as_str())
                                        .filter(|v| !v.trim().is_empty())
                                        .unwrap_or("agent_failed");
                                    (runs_repo::RunStatus::Failed, Some(code))
                                }
                            };

                            let _ = runs_repo::complete_run(
                                &db,
                                &run_id,
                                run_status,
                                summary.clone(),
                                err_code,
                            )
                            .await;
                            let final_status = runs_repo::get_run(&db, &run_id)
                                .await
                                .ok()
                                .flatten()
                                .map(|r| r.status)
                                .unwrap_or(run_status);
                            if final_status == runs_repo::RunStatus::Success {
                                let _ =
                                    run_artifacts_repo::upsert_run_artifact_from_successful_run(
                                        &db, &run_id,
                                    )
                                    .await;
                            }
                            let event_level = if matches!(
                                final_status,
                                runs_repo::RunStatus::Success | runs_repo::RunStatus::Canceled
                            ) {
                                "info"
                            } else {
                                "error"
                            };
                            let event_kind = if final_status == runs_repo::RunStatus::Success {
                                "complete"
                            } else if final_status == runs_repo::RunStatus::Canceled {
                                "canceled"
                            } else {
                                "failed"
                            };
                            let event_fields = if final_status == runs_repo::RunStatus::Failed {
                                Some(agent_task_result_failure_fields(
                                    &agent_id,
                                    &task_id,
                                    &run_id,
                                    summary.as_ref(),
                                    error.as_deref(),
                                ))
                            } else {
                                Some(serde_json::json!({ "agent_id": agent_id.clone() }))
                            };
                            let _ = run_events::append_and_broadcast(
                                &db,
                                &run_events_bus,
                                &run_id,
                                event_level,
                                event_kind,
                                event_kind,
                                event_fields,
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
                                        Some(snapshot_delete_failure_fields(
                                            &agent_id,
                                            kind,
                                            &last_error,
                                            None,
                                            "abandoned",
                                        )),
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
                                    Some(snapshot_delete_failure_fields(
                                        &agent_id,
                                        kind,
                                        &last_error,
                                        Some(next_attempt_at.saturating_sub(now).max(0) as u64),
                                        "failed",
                                    )),
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
                        let requested_status = match result.status.trim() {
                            "success" => operations_repo::OperationStatus::Success,
                            "canceled" => operations_repo::OperationStatus::Canceled,
                            _ => operations_repo::OperationStatus::Failed,
                        };

                        let completed = operations_repo::complete_operation(
                            &db,
                            &result.op_id,
                            requested_status,
                            result.summary.clone(),
                            result.error.as_deref(),
                        )
                        .await
                        .unwrap_or(false);

                        let final_status = operations_repo::get_operation(&db, &result.op_id)
                            .await
                            .ok()
                            .flatten()
                            .map(|op| op.status)
                            .unwrap_or(requested_status);

                        if completed && final_status == operations_repo::OperationStatus::Canceled {
                            let _ = operations_repo::append_event(
                                &db,
                                &result.op_id,
                                "info",
                                "canceled",
                                "canceled",
                                Some(serde_json::json!({ "agent_id": agent_id.clone() })),
                            )
                            .await;
                        }

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

fn should_persist_agent_last_seen(last_persisted_at: i64, now: i64) -> bool {
    now.saturating_sub(last_persisted_at) >= AGENT_LAST_SEEN_MIN_UPDATE_SECS
}

fn localized_text(key: &'static str) -> LocalizedTextRefV1 {
    LocalizedTextRefV1::new(key)
}

fn insert_error_envelope(
    fields: &mut serde_json::Map<String, serde_json::Value>,
    envelope: ErrorEnvelopeV1,
) {
    let Ok(value) = serde_json::to_value(envelope) else {
        return;
    };
    fields.insert("error_envelope".to_string(), value);
}

fn normalize_error_code(value: Option<&str>) -> String {
    let raw = value.unwrap_or("unknown").trim().to_lowercase();
    let mut out = String::new();
    let mut last_sep = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_sep = false;
        } else if !last_sep {
            out.push('_');
            last_sep = true;
        }
    }
    let normalized = out.trim_matches('_').to_string();
    if normalized.is_empty() {
        "unknown".to_string()
    } else {
        normalized
    }
}

fn task_result_hint_text(code: &str) -> &'static str {
    match code {
        "source_consistency" => {
            "source changed during backup; review the consistency policy and inspect the run summary"
        }
        "snapshot_unavailable" => {
            "snapshot provider was unavailable; review snapshot mode/provider support and source path configuration"
        }
        "integrity_check" => {
            "integrity checks failed; inspect the reported lines and repair the database before retrying"
        }
        "network" | "timeout" | "rate_limited" => {
            "transient execution failure detected; review agent connectivity and retry once the environment stabilizes"
        }
        _ => "inspect the related agent logs and run summary for the root cause",
    }
}

fn task_result_is_retriable(code: &str) -> bool {
    matches!(
        code,
        "network" | "timeout" | "rate_limited" | "upstream_unavailable"
    )
}

fn agent_task_result_failure_fields(
    agent_id: &str,
    task_id: &str,
    run_id: &str,
    summary: Option<&serde_json::Value>,
    error: Option<&str>,
) -> serde_json::Value {
    let normalized_code = normalize_error_code(
        summary
            .and_then(|value| value.get("error_code"))
            .and_then(|value| value.as_str())
            .or(error),
    );
    let mut fields = serde_json::Map::new();
    fields.insert("agent_id".to_string(), serde_json::json!(agent_id));
    fields.insert("error_kind".to_string(), serde_json::json!(normalized_code));
    fields.insert(
        "hint".to_string(),
        serde_json::json!(task_result_hint_text(&normalized_code)),
    );
    if let Some(error) = error.filter(|value| !value.trim().is_empty()) {
        fields.insert("error".to_string(), serde_json::json!(error));
    }
    if let Some(code) = summary
        .and_then(|value| value.get("error_code"))
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
    {
        fields.insert("error_code".to_string(), serde_json::json!(code));
    }

    let mut envelope = ErrorEnvelopeV1::new(
        format!("agent.task_result.{normalized_code}"),
        normalized_code.clone(),
        if task_result_is_retriable(&normalized_code) {
            ErrorRetriableV1::new(true).with_reason(normalized_code.clone())
        } else {
            ErrorRetriableV1::new(false)
        },
        localized_text("diagnostics.hint.agent.task_result_failed"),
        localized_text("diagnostics.message.agent.task_result_failed"),
        ErrorTransportV1::new("agent_ws"),
    )
    .with_origin(ErrorOriginV1::new("agent", "ws", "task_result"))
    .with_stage("finalize")
    .with_context(serde_json::json!({
        "agent_id": agent_id,
        "task_id": task_id,
        "run_id": run_id,
        "error_code": summary.and_then(|value| value.get("error_code")).cloned(),
    }));
    if let Some(error) = error.filter(|value| !value.trim().is_empty()) {
        envelope = envelope.with_debug(serde_json::json!({ "error": error }));
    }
    insert_error_envelope(&mut fields, envelope);
    serde_json::Value::Object(fields)
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

fn snapshot_delete_hint_text(kind: &str) -> &'static str {
    match kind {
        "config" => "review snapshot target settings and delete task metadata",
        "auth" => "verify credentials and permissions for the delete operation",
        "network" => "check network connectivity and retry artifact deletion",
        "http" => "inspect the HTTP response/body and upstream availability",
        _ => "inspect artifact delete task logs for the root cause",
    }
}

fn snapshot_delete_failure_fields(
    agent_id: &str,
    kind: &str,
    error: &str,
    retry_after_sec: Option<u64>,
    status: &str,
) -> serde_json::Value {
    let mut fields = serde_json::Map::new();
    fields.insert("agent_id".to_string(), serde_json::json!(agent_id));
    fields.insert("error_kind".to_string(), serde_json::json!(kind));
    fields.insert(
        "hint".to_string(),
        serde_json::json!(snapshot_delete_hint_text(kind)),
    );
    if let Some(retry_after_sec) = retry_after_sec {
        fields.insert(
            "retry_after_secs".to_string(),
            serde_json::json!(retry_after_sec),
        );
    }

    let envelope = ErrorEnvelopeV1::new(
        format!("agent.snapshot_delete.{kind}"),
        kind.to_string(),
        if matches!(kind, "network" | "http") {
            let mut out = ErrorRetriableV1::new(true).with_reason(kind.to_string());
            if let Some(retry_after_sec) = retry_after_sec {
                out = out.with_retry_after_sec(retry_after_sec);
            }
            out
        } else {
            ErrorRetriableV1::new(false)
        },
        localized_text(match kind {
            "config" => "diagnostics.hint.artifact_delete.config",
            "auth" => "diagnostics.hint.artifact_delete.auth",
            "network" => "diagnostics.hint.artifact_delete.network",
            "http" => "diagnostics.hint.artifact_delete.http",
            _ => "diagnostics.hint.artifact_delete.unknown",
        }),
        localized_text(match kind {
            "config" => "diagnostics.message.artifact_delete.config",
            "auth" => "diagnostics.message.artifact_delete.auth",
            "network" => "diagnostics.message.artifact_delete.network",
            "http" => "diagnostics.message.artifact_delete.http",
            _ => "diagnostics.message.artifact_delete.unknown",
        }),
        ErrorTransportV1::new("agent_ws"),
    )
    .with_origin(ErrorOriginV1::new("agent", "ws", "snapshot_delete_result"))
    .with_stage("cleanup")
    .with_context(serde_json::json!({
        "agent_id": agent_id,
        "status": status,
    }))
    .with_debug(serde_json::json!({ "error": error }));
    insert_error_envelope(&mut fields, envelope);
    serde_json::Value::Object(fields)
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

#[cfg(test)]
mod tests {
    use super::{
        AGENT_LAST_SEEN_MIN_UPDATE_SECS, agent_task_result_failure_fields,
        should_persist_agent_last_seen, snapshot_delete_failure_fields,
    };

    #[test]
    fn should_persist_last_seen_only_after_min_interval() {
        let base = 1_700_000_000_i64;
        assert!(!should_persist_agent_last_seen(base, base));
        assert!(!should_persist_agent_last_seen(
            base,
            base + AGENT_LAST_SEEN_MIN_UPDATE_SECS - 1
        ));
        assert!(should_persist_agent_last_seen(
            base,
            base + AGENT_LAST_SEEN_MIN_UPDATE_SECS
        ));
        assert!(should_persist_agent_last_seen(
            base,
            base + AGENT_LAST_SEEN_MIN_UPDATE_SECS + 5
        ));
    }

    #[test]
    fn task_result_failure_fields_include_error_envelope_context() {
        let fields = agent_task_result_failure_fields(
            "agent-1",
            "task-1",
            "run-1",
            Some(&serde_json::json!({ "error_code": "source_consistency" })),
            Some("source changed during backup"),
        );
        let obj = fields.as_object().expect("object");
        assert_eq!(
            obj.get("error_kind").and_then(|value| value.as_str()),
            Some("source_consistency")
        );
        assert_eq!(
            obj.get("error_envelope")
                .and_then(|value| value.get("transport"))
                .and_then(|value| value.get("protocol"))
                .and_then(|value| value.as_str()),
            Some("agent_ws")
        );
        assert_eq!(
            obj.get("error_envelope")
                .and_then(|value| value.get("context"))
                .and_then(|value| value.get("task_id"))
                .and_then(|value| value.as_str()),
            Some("task-1")
        );
    }

    #[test]
    fn snapshot_delete_failure_fields_include_retry_metadata() {
        let fields = snapshot_delete_failure_fields(
            "agent-1",
            "network",
            "dial tcp timeout",
            Some(90),
            "failed",
        );
        let obj = fields.as_object().expect("object");
        assert_eq!(
            obj.get("error_envelope")
                .and_then(|value| value.get("retriable"))
                .and_then(|value| value.get("retry_after_sec"))
                .and_then(|value| value.as_u64()),
            Some(90)
        );
        assert_eq!(
            obj.get("error_envelope")
                .and_then(|value| value.get("message"))
                .and_then(|value| value.get("key"))
                .and_then(|value| value.as_str()),
            Some("diagnostics.message.artifact_delete.network")
        );
    }
}
