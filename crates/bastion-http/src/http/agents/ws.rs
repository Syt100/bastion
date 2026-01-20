use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::ConnectInfo;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use sqlx::SqlitePool;
use url::Url;
use uuid::Uuid;

use bastion_backup::restore::sources::{
    ArtifactSource, LocalDirSource, RunArtifactSource, WebdavSource,
};
use bastion_core::HUB_NODE_ID;
use bastion_core::agent_protocol::{
    AgentToHubMessageV1, ArtifactStreamOpenResultV1, HubToAgentMessageV1, PROTOCOL_VERSION,
};
use bastion_core::agent_stream::{
    ArtifactChunkFrameV1Flags, decode_artifact_chunk_frame_v1, encode_artifact_chunk_frame_v1,
};
use bastion_core::backup_format::{COMPLETE_NAME, ENTRIES_INDEX_NAME, MANIFEST_NAME};
use bastion_core::job_spec;
use bastion_core::manifest::{HashAlgorithm, ManifestV1};
use bastion_engine::agent_manager::AgentManager;
use bastion_engine::run_events;
use bastion_engine::run_events_bus::RunEventsBus;
use bastion_storage::agent_tasks_repo;
use bastion_storage::agents_repo;
use bastion_storage::jobs_repo;
use bastion_storage::operations_repo;
use bastion_storage::runs_repo;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;
use bastion_targets::{WebdavClient, WebdavCredentials};

use super::super::{AppError, AppState};
use super::agent_auth::authenticate_agent;
use super::snapshots::{send_node_config_snapshot, send_node_secrets_snapshot};

const ARTIFACT_STREAM_MAX_BYTES: usize = 1024 * 1024;
const ARTIFACT_STREAM_OPEN_TIMEOUT: Duration = Duration::from_secs(30);
const ARTIFACT_STREAM_PULL_TIMEOUT: Duration = Duration::from_secs(30);

struct HubArtifactStream {
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    cleanup_dir: Option<PathBuf>,
}

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
    Ok(ws.on_upgrade(move |socket| {
        handle_agent_socket(
            data_dir,
            db,
            agent_id,
            peer.ip(),
            secrets,
            agent_manager,
            run_events_bus,
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
    let mut hub_streams: HashMap<Uuid, HubArtifactStream> = HashMap::new();

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
                    Ok(AgentToHubMessageV1::FsListResult {
                        v,
                        request_id,
                        entries,
                        next_cursor,
                        total,
                        error,
                    }) if v == PROTOCOL_VERSION => {
                        let result = if let Some(error) = error {
                            Err(error)
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

                        // Best-effort: close any previous stream with the same id.
                        if let Some(prev) = hub_streams.remove(&stream_id)
                            && let Some(dir) = prev.cleanup_dir
                        {
                            let _ = tokio::fs::remove_dir_all(dir).await;
                        }

                        let opened = open_hub_artifact_stream(
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

#[derive(Debug, Clone)]
enum RunArtifactsLocation {
    Webdav { client: WebdavClient, run_url: Url },
    LocalDir { node_id: String, run_dir: PathBuf },
}

fn target_ref(spec: &job_spec::JobSpecV1) -> &job_spec::TargetV1 {
    match spec {
        job_spec::JobSpecV1::Filesystem { target, .. } => target,
        job_spec::JobSpecV1::Sqlite { target, .. } => target,
        job_spec::JobSpecV1::Vaultwarden { target, .. } => target,
    }
}

async fn resolve_run_artifacts_location(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    run_id: &str,
) -> Result<RunArtifactsLocation, anyhow::Error> {
    let run = runs_repo::get_run(db, run_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("run not found"))?;
    if run.status != runs_repo::RunStatus::Success {
        anyhow::bail!("run is not successful");
    }

    let job = jobs_repo::get_job(db, &run.job_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("job not found"))?;
    let node_id = job
        .agent_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(HUB_NODE_ID)
        .to_string();

    let spec = job_spec::parse_value(&job.spec)?;
    job_spec::validate(&spec)?;

    match target_ref(&spec) {
        job_spec::TargetV1::Webdav {
            base_url,
            secret_name,
            ..
        } => {
            let secret_name = secret_name.trim();
            if secret_name.is_empty() {
                anyhow::bail!("webdav.secret_name is required");
            }

            let cred_bytes = secrets_repo::get_secret(db, secrets, &node_id, "webdav", secret_name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
            let credentials = WebdavCredentials::from_json(&cred_bytes)?;

            let mut base_url = Url::parse(base_url.trim())?;
            if !base_url.path().ends_with('/') {
                base_url.set_path(&format!("{}/", base_url.path()));
            }
            let client = WebdavClient::new(base_url.clone(), credentials)?;

            let job_url = base_url.join(&format!("{}/", run.job_id))?;
            let run_url = job_url.join(&format!("{run_id}/"))?;
            Ok(RunArtifactsLocation::Webdav { client, run_url })
        }
        job_spec::TargetV1::LocalDir { base_dir, .. } => {
            let base_dir = base_dir.trim();
            if base_dir.is_empty() {
                anyhow::bail!("local_dir.base_dir is required");
            }
            let run_dir = PathBuf::from(base_dir).join(&run.job_id).join(run_id);
            Ok(RunArtifactsLocation::LocalDir { node_id, run_dir })
        }
    }
}

fn artifact_stream_staging_dir(data_dir: &Path, op_id: &str, stream_id: Uuid) -> PathBuf {
    data_dir
        .join("hub")
        .join("artifact_streams")
        .join(op_id)
        .join(stream_id.to_string())
}

async fn open_hub_artifact_stream(
    data_dir: &Path,
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    req: &bastion_core::agent_protocol::ArtifactStreamOpenV1,
    stream_id: Uuid,
) -> Result<(HubArtifactStream, Option<u64>), anyhow::Error> {
    let op_id = req.op_id.trim();
    if op_id.is_empty() {
        anyhow::bail!("op_id is required");
    }
    let run_id = req.run_id.trim();
    if run_id.is_empty() {
        anyhow::bail!("run_id is required");
    }
    let artifact = req.artifact.trim();
    if artifact.is_empty() {
        anyhow::bail!("artifact is required");
    }

    let location = resolve_run_artifacts_location(db, secrets, run_id).await?;

    match artifact {
        "payload" => match location {
            RunArtifactsLocation::LocalDir { node_id, run_dir } if node_id == HUB_NODE_ID => {
                let source = RunArtifactSource::Local(LocalDirSource::new(run_dir));
                let manifest = source.read_manifest().await?;
                let size = Some(manifest.artifacts.iter().map(|p| p.size).sum::<u64>());

                let reader = source.open_payload_reader(&manifest, data_dir)?;
                Ok((
                    HubArtifactStream {
                        reader: Arc::new(Mutex::new(reader)),
                        cleanup_dir: None,
                    },
                    size,
                ))
            }
            RunArtifactsLocation::LocalDir { node_id, run_dir } => {
                let manifest =
                    read_agent_manifest(agent_manager, &node_id, op_id, run_id, &run_dir).await?;
                let size = Some(manifest.artifacts.iter().map(|p| p.size).sum::<u64>());

                let reader = RemoteAgentPartsReader::new(
                    tokio::runtime::Handle::current(),
                    agent_manager.clone(),
                    node_id,
                    op_id.to_string(),
                    run_id.to_string(),
                    run_dir,
                    manifest,
                );

                Ok((
                    HubArtifactStream {
                        reader: Arc::new(Mutex::new(Box::new(reader))),
                        cleanup_dir: None,
                    },
                    size,
                ))
            }
            RunArtifactsLocation::Webdav { client, run_url } => {
                let handle = tokio::runtime::Handle::current();
                let source = RunArtifactSource::Webdav(Box::new(WebdavSource::new(
                    handle.clone(),
                    client,
                    run_url,
                )));
                let manifest = source.read_manifest().await?;
                let size = Some(manifest.artifacts.iter().map(|p| p.size).sum::<u64>());

                let staging_dir = artifact_stream_staging_dir(data_dir, op_id, stream_id);
                tokio::fs::create_dir_all(&staging_dir).await?;

                let reader = source.open_payload_reader(&manifest, &staging_dir)?;
                Ok((
                    HubArtifactStream {
                        reader: Arc::new(Mutex::new(reader)),
                        cleanup_dir: Some(staging_dir),
                    },
                    size,
                ))
            }
        },
        MANIFEST_NAME | COMPLETE_NAME => match location {
            RunArtifactsLocation::LocalDir { node_id, run_dir } if node_id == HUB_NODE_ID => {
                let path = run_dir.join(artifact);
                let size = std::fs::metadata(&path)?.len();
                let file = std::fs::File::open(&path)?;
                Ok((
                    HubArtifactStream {
                        reader: Arc::new(Mutex::new(Box::new(file))),
                        cleanup_dir: None,
                    },
                    Some(size),
                ))
            }
            RunArtifactsLocation::LocalDir { node_id, run_dir } => {
                let (reader, size) = {
                    let path = run_dir.join(artifact);
                    open_agent_file_reader(agent_manager, &node_id, op_id, run_id, artifact, &path)
                        .await?
                };
                Ok((
                    HubArtifactStream {
                        reader: Arc::new(Mutex::new(reader)),
                        cleanup_dir: None,
                    },
                    size,
                ))
            }
            RunArtifactsLocation::Webdav { client, run_url } => {
                let url = run_url.join(artifact)?;
                let bytes = client.get_bytes(&url).await?;
                let size = Some(bytes.len() as u64);
                let reader = std::io::Cursor::new(bytes);
                Ok((
                    HubArtifactStream {
                        reader: Arc::new(Mutex::new(Box::new(reader))),
                        cleanup_dir: None,
                    },
                    size,
                ))
            }
        },
        ENTRIES_INDEX_NAME => match location {
            RunArtifactsLocation::LocalDir { node_id, run_dir } if node_id == HUB_NODE_ID => {
                let path = run_dir.join(ENTRIES_INDEX_NAME);
                let size = std::fs::metadata(&path)?.len();
                let file = std::fs::File::open(&path)?;
                Ok((
                    HubArtifactStream {
                        reader: Arc::new(Mutex::new(Box::new(file))),
                        cleanup_dir: None,
                    },
                    Some(size),
                ))
            }
            RunArtifactsLocation::LocalDir { node_id, run_dir } => {
                let path = run_dir.join(ENTRIES_INDEX_NAME);
                let (reader, size) = open_agent_file_reader(
                    agent_manager,
                    &node_id,
                    op_id,
                    run_id,
                    ENTRIES_INDEX_NAME,
                    &path,
                )
                .await?;
                Ok((
                    HubArtifactStream {
                        reader: Arc::new(Mutex::new(reader)),
                        cleanup_dir: None,
                    },
                    size,
                ))
            }
            RunArtifactsLocation::Webdav { client, run_url } => {
                let url = run_url.join(ENTRIES_INDEX_NAME)?;
                let staging_dir = artifact_stream_staging_dir(data_dir, op_id, stream_id);
                tokio::fs::create_dir_all(&staging_dir).await?;
                let dest = staging_dir.join(ENTRIES_INDEX_NAME);
                let size = client.get_to_file(&url, &dest, None, 3).await?;
                let file = std::fs::File::open(&dest)?;
                Ok((
                    HubArtifactStream {
                        reader: Arc::new(Mutex::new(Box::new(file))),
                        cleanup_dir: Some(staging_dir),
                    },
                    Some(size),
                ))
            }
        },
        other => anyhow::bail!("unsupported artifact: {}", other),
    }
}

async fn read_agent_manifest(
    agent_manager: &AgentManager,
    agent_id: &str,
    op_id: &str,
    run_id: &str,
    run_dir: &Path,
) -> Result<ManifestV1, anyhow::Error> {
    let path = run_dir.join(MANIFEST_NAME);
    let stream_id = Uuid::new_v4();
    let open = bastion_core::agent_protocol::ArtifactStreamOpenV1 {
        stream_id: stream_id.to_string(),
        op_id: op_id.to_string(),
        run_id: run_id.to_string(),
        artifact: MANIFEST_NAME.to_string(),
        path: Some(path.to_string_lossy().to_string()),
    };

    let res = agent_manager
        .artifact_stream_open(agent_id, open, ARTIFACT_STREAM_OPEN_TIMEOUT)
        .await?;
    if let Some(error) = res.error.as_deref()
        && !error.trim().is_empty()
    {
        anyhow::bail!("agent open manifest failed: {error}");
    }

    let mut bytes = Vec::new();
    loop {
        let chunk = agent_manager
            .artifact_stream_pull(
                agent_id,
                bastion_core::agent_protocol::ArtifactStreamPullV1 {
                    stream_id: stream_id.to_string(),
                    max_bytes: ARTIFACT_STREAM_MAX_BYTES as u32,
                },
                ARTIFACT_STREAM_PULL_TIMEOUT,
            )
            .await?;
        bytes.extend_from_slice(&chunk.bytes);
        if chunk.eof {
            break;
        }
    }

    let manifest = serde_json::from_slice::<ManifestV1>(&bytes)?;
    Ok(manifest)
}

async fn open_agent_file_reader(
    agent_manager: &AgentManager,
    agent_id: &str,
    op_id: &str,
    run_id: &str,
    artifact: &str,
    path: &Path,
) -> Result<(Box<dyn Read + Send>, Option<u64>), anyhow::Error> {
    let stream_id = Uuid::new_v4();
    let open = bastion_core::agent_protocol::ArtifactStreamOpenV1 {
        stream_id: stream_id.to_string(),
        op_id: op_id.to_string(),
        run_id: run_id.to_string(),
        artifact: artifact.to_string(),
        path: Some(path.to_string_lossy().to_string()),
    };

    let res = agent_manager
        .artifact_stream_open(agent_id, open, ARTIFACT_STREAM_OPEN_TIMEOUT)
        .await?;
    if let Some(error) = res.error.as_deref()
        && !error.trim().is_empty()
    {
        anyhow::bail!("agent open {artifact} failed: {error}");
    }

    let reader = RemoteAgentFileReader {
        handle: tokio::runtime::Handle::current(),
        agent_manager: agent_manager.clone(),
        agent_id: agent_id.to_string(),
        stream_id,
        eof: false,
        buf: Vec::new(),
        pos: 0,
    };
    Ok((Box::new(reader), res.size))
}

struct RemoteAgentFileReader {
    handle: tokio::runtime::Handle,
    agent_manager: AgentManager,
    agent_id: String,
    stream_id: Uuid,
    eof: bool,
    buf: Vec<u8>,
    pos: usize,
}

impl Read for RemoteAgentFileReader {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }

        loop {
            if self.pos < self.buf.len() {
                let n = std::cmp::min(out.len(), self.buf.len() - self.pos);
                out[..n].copy_from_slice(&self.buf[self.pos..self.pos + n]);
                self.pos += n;
                if self.pos >= self.buf.len() {
                    self.buf.clear();
                    self.pos = 0;
                }
                return Ok(n);
            }

            if self.eof {
                return Ok(0);
            }

            let want = out.len().clamp(1, ARTIFACT_STREAM_MAX_BYTES);
            let chunk = self
                .handle
                .block_on(self.agent_manager.artifact_stream_pull(
                    &self.agent_id,
                    bastion_core::agent_protocol::ArtifactStreamPullV1 {
                        stream_id: self.stream_id.to_string(),
                        max_bytes: want as u32,
                    },
                    ARTIFACT_STREAM_PULL_TIMEOUT,
                ))
                .map_err(|e| std::io::Error::other(e.to_string()))?;

            self.eof = chunk.eof;
            self.buf = chunk.bytes;
            self.pos = 0;
        }
    }
}

struct RemoteAgentPartsReader {
    handle: tokio::runtime::Handle,
    agent_manager: AgentManager,
    agent_id: String,
    op_id: String,
    run_id: String,
    run_dir: PathBuf,
    parts: Vec<bastion_core::manifest::ArtifactPart>,
    next_index: usize,
    current: Option<RemoteActivePart>,
}

struct RemoteActivePart {
    name: String,
    reader: RemoteAgentFileReader,
    hasher: blake3::Hasher,
    read_bytes: u64,
    expected_size: u64,
    expected_hash_alg: HashAlgorithm,
    expected_hash: String,
}

impl RemoteAgentPartsReader {
    fn new(
        handle: tokio::runtime::Handle,
        agent_manager: AgentManager,
        agent_id: String,
        op_id: String,
        run_id: String,
        run_dir: PathBuf,
        manifest: ManifestV1,
    ) -> Self {
        Self {
            handle,
            agent_manager,
            agent_id,
            op_id,
            run_id,
            run_dir,
            parts: manifest.artifacts,
            next_index: 0,
            current: None,
        }
    }

    fn open_next(&mut self) -> std::io::Result<()> {
        let idx = self.next_index;
        let spec = self
            .parts
            .get(idx)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "no more parts"))?
            .clone();

        let path = self.run_dir.join(&spec.name);
        let stream_id = Uuid::new_v4();
        let open = bastion_core::agent_protocol::ArtifactStreamOpenV1 {
            stream_id: stream_id.to_string(),
            op_id: self.op_id.clone(),
            run_id: self.run_id.clone(),
            artifact: spec.name.clone(),
            path: Some(path.to_string_lossy().to_string()),
        };

        let res = self
            .handle
            .block_on(self.agent_manager.artifact_stream_open(
                &self.agent_id,
                open,
                ARTIFACT_STREAM_OPEN_TIMEOUT,
            ))
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        if let Some(error) = res.error.as_deref()
            && !error.trim().is_empty()
        {
            return Err(std::io::Error::other(format!(
                "agent open part {} failed: {error}",
                spec.name
            )));
        }
        if let Some(size) = res.size
            && size != spec.size
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "part size mismatch for {}: expected {}, got {}",
                    spec.name, spec.size, size
                ),
            ));
        }

        self.current = Some(RemoteActivePart {
            name: spec.name.clone(),
            reader: RemoteAgentFileReader {
                handle: self.handle.clone(),
                agent_manager: self.agent_manager.clone(),
                agent_id: self.agent_id.clone(),
                stream_id,
                eof: false,
                buf: Vec::new(),
                pos: 0,
            },
            hasher: blake3::Hasher::new(),
            read_bytes: 0,
            expected_size: spec.size,
            expected_hash_alg: spec.hash_alg,
            expected_hash: spec.hash,
        });
        self.next_index += 1;
        Ok(())
    }

    fn finish_current(&mut self) -> std::io::Result<()> {
        let Some(active) = self.current.take() else {
            return Ok(());
        };

        if active.read_bytes != active.expected_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "part read size mismatch for {}: expected {}, got {}",
                    active.name, active.expected_size, active.read_bytes
                ),
            ));
        }

        match active.expected_hash_alg {
            HashAlgorithm::Blake3 => {
                let computed = active.hasher.finalize().to_hex().to_string();
                if computed != active.expected_hash {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "part hash mismatch for {}: expected {}, got {}",
                            active.name, active.expected_hash, computed
                        ),
                    ));
                }
            }
            other => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("unsupported part hash algorithm: {other:?}"),
                ));
            }
        }

        Ok(())
    }
}

impl Read for RemoteAgentPartsReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if self.current.is_none() {
                if self.next_index >= self.parts.len() {
                    return Ok(0);
                }
                self.open_next()?;
            }

            let active = self.current.as_mut().expect("current part exists");
            let n = active.reader.read(buf)?;
            if n == 0 {
                self.finish_current()?;
                continue;
            }

            active.hasher.update(&buf[..n]);
            active.read_bytes = active.read_bytes.saturating_add(n as u64);
            if active.read_bytes > active.expected_size {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "part size overflow for {}: expected {}, got >{}",
                        active.name, active.expected_size, active.expected_size
                    ),
                ));
            }
            return Ok(n);
        }
    }
}
