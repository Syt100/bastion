use std::path::{Path, PathBuf};
use std::time::Duration;

use futures_util::{Sink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tracing::{debug, info, warn};
use url::Url;

use bastion_core::agent_protocol::{
    AgentToHubMessageV1, EncryptionResolvedV1, HubToAgentMessageV1, JobSpecResolvedV1,
    PROTOCOL_VERSION, TargetResolvedV1,
};
use bastion_core::run_failure::RunFailedWithSummary;
use bastion_targets::WebdavCredentials;

use crate::config::AgentArgs;
use bastion_backup as backup;
use bastion_targets as targets;

const IDENTITY_FILE_NAME: &str = "agent.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AgentIdentityV1 {
    v: u32,
    hub_url: String,
    agent_id: String,
    agent_key: String,
    name: Option<String>,
    enrolled_at: i64,
}

#[derive(Debug, Serialize)]
struct EnrollRequest<'a> {
    token: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct EnrollResponse {
    agent_id: String,
    agent_key: String,
}

pub async fn run(args: AgentArgs) -> Result<(), anyhow::Error> {
    if args.heartbeat_seconds == 0 {
        anyhow::bail!("heartbeat_seconds must be > 0");
    }

    let data_dir = bastion_config::data_dir::resolve_data_dir(args.data_dir)?;
    let base_url = normalize_base_url(&args.hub_url)?;

    let identity_path = identity_path(&data_dir);
    let identity = match load_identity(&identity_path)? {
        Some(v) => {
            let stored_url = normalize_base_url(&v.hub_url)?;
            if stored_url != base_url {
                anyhow::bail!(
                    "agent is already enrolled for hub_url={}, delete {} to re-enroll",
                    stored_url,
                    identity_path.display()
                );
            }
            v
        }
        None => {
            let Some(token) = args.enroll_token.as_deref() else {
                anyhow::bail!(
                    "agent is not enrolled yet; provide --enroll-token or set BASTION_AGENT_ENROLL_TOKEN"
                );
            };

            info!(hub_url = %base_url, "enrolling agent");
            let resp = enroll(&base_url, token, args.name.as_deref()).await?;
            let now = time::OffsetDateTime::now_utc().unix_timestamp();
            let identity = AgentIdentityV1 {
                v: 1,
                hub_url: base_url.to_string(),
                agent_id: resp.agent_id,
                agent_key: resp.agent_key,
                name: args.name.clone(),
                enrolled_at: now,
            };
            save_identity(&identity_path, &identity)?;
            identity
        }
    };

    let ws_url = agent_ws_url(&base_url)?;
    let heartbeat = Duration::from_secs(args.heartbeat_seconds);
    let pong_timeout = Duration::from_secs(args.heartbeat_seconds.saturating_mul(3));
    let mut backoff = Duration::from_secs(1);
    let mut attempt = 0u32;

    loop {
        let action = connect_and_run(&ws_url, &identity, &data_dir, heartbeat, pong_timeout).await;
        match action {
            Ok(LoopAction::Exit) => return Ok(()),
            Ok(LoopAction::Reconnect) => {
                attempt = attempt.saturating_add(1);
                tokio::time::sleep(jittered_backoff(backoff, &identity.agent_id, attempt)).await;
                backoff = std::cmp::min(backoff * 2, Duration::from_secs(30));
            }
            Err(error) => {
                warn!(error = %error, "agent connection failed; retrying");
                attempt = attempt.saturating_add(1);
                tokio::time::sleep(jittered_backoff(backoff, &identity.agent_id, attempt)).await;
                backoff = std::cmp::min(backoff * 2, Duration::from_secs(30));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoopAction {
    Reconnect,
    Exit,
}

async fn connect_and_run(
    ws_url: &Url,
    identity: &AgentIdentityV1,
    data_dir: &Path,
    heartbeat: Duration,
    pong_timeout: Duration,
) -> Result<LoopAction, anyhow::Error> {
    let mut req = ws_url.as_str().into_client_request()?;
    req.headers_mut().insert(
        AUTHORIZATION,
        format!("Bearer {}", identity.agent_key).parse()?,
    );

    let (socket, _) = tokio_tungstenite::connect_async(req).await?;
    let (mut tx, mut rx) = socket.split();

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
        }),
    };
    tx.send(Message::Text(serde_json::to_string(&hello)?.into()))
        .await?;

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

                                match handle_backup_task(data_dir, &mut tx, &task_id, *task).await {
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

async fn handle_backup_task(
    data_dir: &Path,
    tx: &mut (impl Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    task_id: &str,
    task: bastion_core::agent_protocol::BackupRunTaskV1,
) -> Result<(), anyhow::Error> {
    let run_id = task.run_id.clone();
    let job_id = task.job_id.clone();
    let started_at = time::OffsetDateTime::from_unix_timestamp(task.started_at)
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc());

    send_run_event(tx, &run_id, "info", "start", "start", None).await?;

    let summary = match task.spec {
        JobSpecResolvedV1::Filesystem {
            pipeline,
            source,
            target,
            ..
        } => {
            send_run_event(tx, &run_id, "info", "packaging", "packaging", None).await?;
            let part_size = target_part_size_bytes(&target);
            let error_policy = source.error_policy;
            let encryption = match pipeline.encryption {
                EncryptionResolvedV1::None => backup::PayloadEncryption::None,
                EncryptionResolvedV1::AgeX25519 {
                    recipient,
                    key_name,
                } => backup::PayloadEncryption::AgeX25519 {
                    recipient,
                    key_name,
                },
            };
            let data_dir_buf = data_dir.to_path_buf();
            let job_id_clone = job_id.clone();
            let run_id_clone = run_id.clone();
            let build = tokio::task::spawn_blocking(move || {
                backup::filesystem::build_filesystem_run(
                    &data_dir_buf,
                    &job_id_clone,
                    &run_id_clone,
                    started_at,
                    &source,
                    &encryption,
                    part_size,
                )
            })
            .await??;

            if build.issues.warnings_total > 0 || build.issues.errors_total > 0 {
                let level = if build.issues.errors_total > 0 {
                    "error"
                } else {
                    "warn"
                };
                let fields = serde_json::json!({
                    "warnings_total": build.issues.warnings_total,
                    "errors_total": build.issues.errors_total,
                    "sample_warnings": &build.issues.sample_warnings,
                    "sample_errors": &build.issues.sample_errors,
                });
                send_run_event(
                    tx,
                    &run_id,
                    level,
                    "fs_issues",
                    "filesystem issues",
                    Some(fields),
                )
                .await?;
            }

            let issues = build.issues;
            let artifacts = build.artifacts;

            send_run_event(tx, &run_id, "info", "upload", "upload", None).await?;
            let target_summary =
                store_artifacts_to_resolved_target(&job_id, &run_id, &target, &artifacts).await?;

            let _ = tokio::fs::remove_dir_all(&artifacts.run_dir).await;

            let mut summary = serde_json::json!({
                "target": target_summary,
                "entries_count": artifacts.entries_count,
                "parts": artifacts.parts.len(),
                "filesystem": {
                    "warnings_total": issues.warnings_total,
                    "errors_total": issues.errors_total,
                }
            });

            if error_policy == bastion_core::job_spec::FsErrorPolicy::SkipFail
                && issues.errors_total > 0
            {
                if let Some(obj) = summary.as_object_mut() {
                    obj.insert(
                        "error_code".to_string(),
                        serde_json::Value::String("fs_issues".to_string()),
                    );
                }
                return Err(anyhow::Error::new(RunFailedWithSummary::new(
                    "fs_issues",
                    format!(
                        "filesystem backup completed with {} errors",
                        issues.errors_total
                    ),
                    summary,
                )));
            }

            summary
        }
        JobSpecResolvedV1::Sqlite {
            pipeline,
            source,
            target,
            ..
        } => {
            send_run_event(tx, &run_id, "info", "snapshot", "snapshot", None).await?;
            let sqlite_path = source.path.clone();
            let part_size = target_part_size_bytes(&target);

            let encryption = match pipeline.encryption {
                EncryptionResolvedV1::None => backup::PayloadEncryption::None,
                EncryptionResolvedV1::AgeX25519 {
                    recipient,
                    key_name,
                } => backup::PayloadEncryption::AgeX25519 {
                    recipient,
                    key_name,
                },
            };
            let data_dir_buf = data_dir.to_path_buf();
            let job_id_clone = job_id.clone();
            let run_id_clone = run_id.clone();
            let build = tokio::task::spawn_blocking(move || {
                backup::sqlite::build_sqlite_run(
                    &data_dir_buf,
                    &job_id_clone,
                    &run_id_clone,
                    started_at,
                    &source,
                    &encryption,
                    part_size,
                )
            })
            .await??;

            if let Some(check) = build.integrity_check.as_ref() {
                let data = serde_json::json!({
                    "ok": check.ok,
                    "truncated": check.truncated,
                    "lines": check.lines,
                });
                send_run_event(
                    tx,
                    &run_id,
                    if check.ok { "info" } else { "error" },
                    "integrity_check",
                    "integrity_check",
                    Some(data),
                )
                .await?;
                if !check.ok {
                    let first = check.lines.first().cloned().unwrap_or_default();
                    anyhow::bail!("sqlite integrity_check failed: {}", first);
                }
            }

            send_run_event(tx, &run_id, "info", "upload", "upload", None).await?;
            let target_summary =
                store_artifacts_to_resolved_target(&job_id, &run_id, &target, &build.artifacts)
                    .await?;
            let _ = tokio::fs::remove_dir_all(&build.artifacts.run_dir).await;

            serde_json::json!({
                "target": target_summary,
                "entries_count": build.artifacts.entries_count,
                "parts": build.artifacts.parts.len(),
                "sqlite": {
                    "path": sqlite_path,
                    "snapshot_name": build.snapshot_name,
                    "snapshot_size": build.snapshot_size,
                    "integrity_check": build.integrity_check.map(|check| serde_json::json!({
                        "ok": check.ok,
                        "truncated": check.truncated,
                        "lines": check.lines,
                    })),
                },
            })
        }
        JobSpecResolvedV1::Vaultwarden {
            pipeline,
            source,
            target,
            ..
        } => {
            send_run_event(tx, &run_id, "info", "snapshot", "snapshot", None).await?;
            let vw_data_dir = source.data_dir.clone();
            let part_size = target_part_size_bytes(&target);

            let encryption = match pipeline.encryption {
                EncryptionResolvedV1::None => backup::PayloadEncryption::None,
                EncryptionResolvedV1::AgeX25519 {
                    recipient,
                    key_name,
                } => backup::PayloadEncryption::AgeX25519 {
                    recipient,
                    key_name,
                },
            };
            let data_dir_buf = data_dir.to_path_buf();
            let job_id_clone = job_id.clone();
            let run_id_clone = run_id.clone();
            let artifacts = tokio::task::spawn_blocking(move || {
                backup::vaultwarden::build_vaultwarden_run(
                    &data_dir_buf,
                    &job_id_clone,
                    &run_id_clone,
                    started_at,
                    &source,
                    &encryption,
                    part_size,
                )
            })
            .await??;

            send_run_event(tx, &run_id, "info", "upload", "upload", None).await?;
            let target_summary =
                store_artifacts_to_resolved_target(&job_id, &run_id, &target, &artifacts).await?;
            let _ = tokio::fs::remove_dir_all(&artifacts.run_dir).await;

            serde_json::json!({
                "target": target_summary,
                "entries_count": artifacts.entries_count,
                "parts": artifacts.parts.len(),
                "vaultwarden": {
                    "data_dir": vw_data_dir,
                    "db": "db.sqlite3",
                }
            })
        }
    };

    send_run_event(tx, &run_id, "info", "complete", "complete", None).await?;

    let result = AgentToHubMessageV1::TaskResult {
        v: PROTOCOL_VERSION,
        task_id: task_id.to_string(),
        run_id: run_id.clone(),
        status: "success".to_string(),
        summary: Some(summary),
        error: None,
    };
    if let Err(error) = save_task_result(data_dir, &result) {
        warn!(task_id = %task_id, error = %error, "failed to persist task result");
    }
    tx.send(Message::Text(serde_json::to_string(&result)?.into()))
        .await?;
    Ok(())
}

async fn send_run_event(
    tx: &mut (impl Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    run_id: &str,
    level: &str,
    kind: &str,
    message: &str,
    fields: Option<serde_json::Value>,
) -> Result<(), anyhow::Error> {
    let msg = AgentToHubMessageV1::RunEvent {
        v: PROTOCOL_VERSION,
        run_id: run_id.to_string(),
        level: level.to_string(),
        kind: kind.to_string(),
        message: message.to_string(),
        fields,
    };
    tx.send(Message::Text(serde_json::to_string(&msg)?.into()))
        .await?;
    Ok(())
}

fn target_part_size_bytes(target: &TargetResolvedV1) -> u64 {
    match target {
        TargetResolvedV1::Webdav {
            part_size_bytes, ..
        } => *part_size_bytes,
        TargetResolvedV1::LocalDir {
            part_size_bytes, ..
        } => *part_size_bytes,
    }
}

async fn store_artifacts_to_resolved_target(
    job_id: &str,
    run_id: &str,
    target: &TargetResolvedV1,
    artifacts: &backup::LocalRunArtifacts,
) -> Result<serde_json::Value, anyhow::Error> {
    match target {
        TargetResolvedV1::Webdav {
            base_url,
            username,
            password,
            ..
        } => {
            let creds = WebdavCredentials {
                username: username.clone(),
                password: password.clone(),
            };
            let run_url =
                targets::webdav::store_run(base_url, creds, job_id, run_id, artifacts).await?;
            Ok(serde_json::json!({ "type": "webdav", "run_url": run_url.as_str() }))
        }
        TargetResolvedV1::LocalDir { base_dir, .. } => {
            let base_dir = base_dir.to_string();
            let job_id = job_id.to_string();
            let run_id = run_id.to_string();
            let artifacts = artifacts.clone();
            let run_dir = tokio::task::spawn_blocking(move || {
                targets::local_dir::store_run(
                    std::path::Path::new(&base_dir),
                    &job_id,
                    &run_id,
                    &artifacts,
                )
            })
            .await??;
            Ok(serde_json::json!({
                "type": "local_dir",
                "run_dir": run_dir.to_string_lossy().to_string()
            }))
        }
    }
}

fn identity_path(data_dir: &Path) -> PathBuf {
    data_dir.join(IDENTITY_FILE_NAME)
}

fn normalize_base_url(raw: &str) -> Result<Url, anyhow::Error> {
    let mut url = Url::parse(raw)?;
    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/", url.path()));
    }
    Ok(url)
}

fn agent_ws_url(base_url: &Url) -> Result<Url, anyhow::Error> {
    let mut url = base_url.clone();
    match url.scheme() {
        "http" => url.set_scheme("ws").ok(),
        "https" => url.set_scheme("wss").ok(),
        other => anyhow::bail!("unsupported hub_url scheme: {other}"),
    };
    Ok(url.join("agent/ws")?)
}

async fn enroll(
    base_url: &Url,
    token: &str,
    name: Option<&str>,
) -> Result<EnrollResponse, anyhow::Error> {
    let enroll_url = base_url.join("agent/enroll")?;
    let res = reqwest::Client::new()
        .post(enroll_url)
        .json(&EnrollRequest { token, name })
        .send()
        .await?;

    if res.status() != reqwest::StatusCode::OK {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        anyhow::bail!("enroll failed: HTTP {status}: {text}");
    }

    Ok(res.json::<EnrollResponse>().await?)
}

fn load_identity(path: &Path) -> Result<Option<AgentIdentityV1>, anyhow::Error> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(serde_json::from_slice::<AgentIdentityV1>(&bytes)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn save_identity(path: &Path, identity: &AgentIdentityV1) -> Result<(), anyhow::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let bytes = serde_json::to_vec_pretty(identity)?;
    let tmp = path.with_extension("json.partial");
    let _ = std::fs::remove_file(&tmp);
    std::fs::write(&tmp, bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }

    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn jittered_backoff(base: Duration, agent_id: &str, attempt: u32) -> Duration {
    if base.is_zero() {
        return base;
    }

    // Equal-jitter backoff: [base/2, base], deterministic per agent+attempt.
    let half = base / 2;
    let half_ms = half.as_millis().min(u128::from(u64::MAX)) as u64;
    if half_ms == 0 {
        return base;
    }

    let seed = fnv1a64(agent_id.as_bytes())
        .wrapping_add(u64::from(attempt).wrapping_mul(0x9e3779b97f4a7c15));
    let jitter_ms = seed % (half_ms + 1);
    half + Duration::from_millis(jitter_ms)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn is_ws_error(error: &anyhow::Error) -> bool {
    error
        .chain()
        .any(|e| e.is::<tokio_tungstenite::tungstenite::Error>())
}

fn is_safe_task_id(task_id: &str) -> bool {
    !task_id.is_empty()
        && task_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn task_results_dir(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("agent").join("task_results")
}

fn task_result_path(data_dir: &Path, task_id: &str) -> Option<std::path::PathBuf> {
    if !is_safe_task_id(task_id) {
        return None;
    }
    Some(task_results_dir(data_dir).join(format!("{task_id}.json")))
}

fn load_cached_task_result(
    data_dir: &Path,
    task_id: &str,
    run_id: &str,
) -> Option<AgentToHubMessageV1> {
    let path = task_result_path(data_dir, task_id)?;
    let bytes = std::fs::read(path).ok()?;
    let msg = serde_json::from_slice::<AgentToHubMessageV1>(&bytes).ok()?;
    match &msg {
        AgentToHubMessageV1::TaskResult {
            v,
            task_id: saved_task_id,
            run_id: saved_run_id,
            ..
        } if *v == PROTOCOL_VERSION && saved_task_id == task_id && saved_run_id == run_id => {
            Some(msg)
        }
        _ => None,
    }
}

fn save_task_result(data_dir: &Path, msg: &AgentToHubMessageV1) -> Result<(), anyhow::Error> {
    let AgentToHubMessageV1::TaskResult { task_id, .. } = msg else {
        return Ok(());
    };

    let Some(path) = task_result_path(data_dir, task_id) else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let bytes = serde_json::to_vec_pretty(msg)?;
    let tmp = path.with_extension("json.partial");
    let _ = std::fs::remove_file(&tmp);
    std::fs::write(&tmp, bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }

    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{agent_ws_url, normalize_base_url, save_identity};

    #[test]
    fn normalize_base_url_appends_slash() {
        let url = normalize_base_url("http://localhost:9876").unwrap();
        assert_eq!(url.as_str(), "http://localhost:9876/");
    }

    #[test]
    fn agent_ws_url_converts_scheme() {
        let base = normalize_base_url("https://hub.example.com/bastion").unwrap();
        let ws = agent_ws_url(&base).unwrap();
        assert_eq!(ws.as_str(), "wss://hub.example.com/bastion/agent/ws");
    }

    #[test]
    fn identity_is_written_atomically() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("agent.json");
        let id = super::AgentIdentityV1 {
            v: 1,
            hub_url: "http://localhost:9876/".to_string(),
            agent_id: "a".to_string(),
            agent_key: "k".to_string(),
            name: Some("n".to_string()),
            enrolled_at: 1,
        };

        save_identity(&path, &id).unwrap();
        let saved = std::fs::read_to_string(&path).unwrap();
        assert!(saved.contains("\"agent_id\""));
    }
}
