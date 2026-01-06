use std::sync::Arc;

use axum::Json;
use axum::extract::ConnectInfo;
use axum::extract::Path;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use bastion_core::agent;
use bastion_core::agent_protocol::{
    AgentToHubMessageV1, HubToAgentMessageV1, JobConfigV1, OverlapPolicyV1, PROTOCOL_VERSION,
    WebdavSecretV1,
};
use bastion_core::job_spec;

use super::shared::{require_csrf, require_session};
use super::{AppError, AppState};
use bastion_engine::agent_job_resolver;
use bastion_engine::agent_manager::AgentManager;
use bastion_engine::notifications;
use bastion_engine::run_events;
use bastion_engine::run_events_bus::RunEventsBus;
use bastion_storage::agent_tasks_repo;
use bastion_storage::agents_repo;
use bastion_storage::jobs_repo;
use bastion_storage::runs_repo;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;

#[derive(Debug, Deserialize)]
pub(super) struct CreateEnrollmentTokenRequest {
    #[serde(default = "default_enroll_ttl_seconds")]
    ttl_seconds: i64,
    remaining_uses: Option<i64>,
}

fn default_enroll_ttl_seconds() -> i64 {
    60 * 60
}

#[derive(Debug, Serialize)]
pub(super) struct CreateEnrollmentTokenResponse {
    token: String,
    expires_at: i64,
    remaining_uses: Option<i64>,
}

pub(super) async fn create_enrollment_token(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Json(req): Json<CreateEnrollmentTokenRequest>,
) -> Result<Json<CreateEnrollmentTokenResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let token = agent::generate_token_b64_urlsafe(32);
    let token_hash = agent::sha256_urlsafe_token(&token)?;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let expires_at = now + req.ttl_seconds;

    sqlx::query(
        "INSERT INTO enrollment_tokens (token_hash, created_at, expires_at, remaining_uses) VALUES (?, ?, ?, ?)",
    )
    .bind(token_hash)
    .bind(now)
    .bind(expires_at)
    .bind(req.remaining_uses)
    .execute(&state.db)
    .await?;

    Ok(Json(CreateEnrollmentTokenResponse {
        token,
        expires_at,
        remaining_uses: req.remaining_uses,
    }))
}

#[derive(Debug, Serialize)]
pub(super) struct AgentListItem {
    id: String,
    name: Option<String>,
    revoked: bool,
    last_seen_at: Option<i64>,
    online: bool,
}

fn agent_online(revoked: bool, last_seen_at: Option<i64>, now: i64) -> bool {
    if revoked {
        return false;
    }

    let cutoff = now.saturating_sub(60);
    last_seen_at.is_some_and(|ts| ts >= cutoff)
}

pub(super) async fn list_agents(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<AgentListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();

    let rows = sqlx::query(
        "SELECT id, name, revoked_at, last_seen_at FROM agents ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    let agents = rows
        .into_iter()
        .map(|r| {
            let revoked = r.get::<Option<i64>, _>("revoked_at").is_some();
            let last_seen_at = r.get::<Option<i64>, _>("last_seen_at");
            let online = agent_online(revoked, last_seen_at, now);

            AgentListItem {
                id: r.get::<String, _>("id"),
                name: r.get::<Option<String>, _>("name"),
                revoked,
                last_seen_at,
                online,
            }
        })
        .collect();

    Ok(Json(agents))
}

pub(super) async fn revoke_agent(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    sqlx::query("UPDATE agents SET revoked_at = ? WHERE id = ? AND revoked_at IS NULL")
        .bind(now)
        .bind(agent_id)
        .execute(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
pub(super) struct RotateAgentKeyResponse {
    agent_id: String,
    agent_key: String,
}

pub(super) async fn rotate_agent_key(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
) -> Result<Json<RotateAgentKeyResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let agent_key = agents_repo::rotate_agent_key(&state.db, &agent_id)
        .await?
        .ok_or_else(|| AppError::not_found("agent_not_found", "Agent not found"))?;

    Ok(Json(RotateAgentKeyResponse {
        agent_id,
        agent_key,
    }))
}

#[derive(Debug, Deserialize)]
pub(super) struct AgentIngestRunRequest {
    run: AgentIngestRun,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AgentIngestRunStatus {
    Success,
    Failed,
    Rejected,
}

#[derive(Debug, Deserialize)]
struct AgentIngestRun {
    id: String,
    job_id: String,
    status: AgentIngestRunStatus,
    started_at: i64,
    ended_at: i64,
    #[serde(default)]
    summary: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    events: Vec<AgentIngestRunEvent>,
}

#[derive(Debug, Deserialize)]
struct AgentIngestRunEvent {
    seq: i64,
    ts: i64,
    level: String,
    kind: String,
    message: String,
    #[serde(default)]
    fields: Option<serde_json::Value>,
}

pub(super) async fn agent_ingest_runs(
    state: axum::extract::State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AgentIngestRunRequest>,
) -> Result<StatusCode, AppError> {
    const MAX_EVENTS_PER_RUN: usize = 2000;

    let agent_id = authenticate_agent(&state.db, &headers).await?;

    let run = req.run;
    if run.id.trim().is_empty() {
        return Err(AppError::bad_request(
            "invalid_run_id",
            "Run id is required",
        ));
    }
    if run.job_id.trim().is_empty() {
        return Err(AppError::bad_request(
            "invalid_job_id",
            "Job id is required",
        ));
    }
    if run.events.len() > MAX_EVENTS_PER_RUN {
        return Err(AppError::bad_request(
            "too_many_events",
            "Too many run events",
        ));
    }

    let row = sqlx::query("SELECT agent_id, spec_json FROM jobs WHERE id = ? LIMIT 1")
        .bind(&run.job_id)
        .fetch_optional(&state.db)
        .await?;

    let Some(row) = row else {
        return Err(AppError::bad_request("invalid_job_id", "Job not found"));
    };
    let job_agent_id = row.get::<Option<String>, _>("agent_id");
    if job_agent_id.as_deref() != Some(agent_id.as_str()) {
        return Err(AppError::bad_request(
            "invalid_job_id",
            "Job is not assigned to this Agent",
        ));
    }
    let spec_json = row.get::<String, _>("spec_json");

    let status = match run.status {
        AgentIngestRunStatus::Success => runs_repo::RunStatus::Success,
        AgentIngestRunStatus::Failed => runs_repo::RunStatus::Failed,
        AgentIngestRunStatus::Rejected => runs_repo::RunStatus::Rejected,
    };

    let summary_json = run
        .summary
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;

    let mut inserted_events = Vec::new();
    let mut tx = state.db.begin().await?;

    let _ = sqlx::query(
        "INSERT OR IGNORE INTO runs (id, job_id, status, started_at, ended_at, summary_json, error) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&run.id)
    .bind(&run.job_id)
    .bind(status.as_str())
    .bind(run.started_at)
    .bind(run.ended_at)
    .bind(summary_json)
    .bind(run.error.as_deref())
    .execute(&mut *tx)
    .await?;

    for ev in &run.events {
        if ev.seq <= 0 {
            continue;
        }
        let fields_json = ev.fields.as_ref().map(serde_json::to_string).transpose()?;
        let result = sqlx::query(
            "INSERT OR IGNORE INTO run_events (run_id, seq, ts, level, kind, message, fields_json) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&run.id)
        .bind(ev.seq)
        .bind(ev.ts)
        .bind(ev.level.trim())
        .bind(ev.kind.trim())
        .bind(ev.message.trim())
        .bind(fields_json)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() > 0 {
            inserted_events.push(runs_repo::RunEvent {
                run_id: run.id.clone(),
                seq: ev.seq,
                ts: ev.ts,
                level: ev.level.clone(),
                kind: ev.kind.clone(),
                message: ev.message.clone(),
                fields: ev.fields.clone(),
            });
        }
    }

    tx.commit().await?;

    for ev in &inserted_events {
        state.run_events_bus.publish(ev);
    }

    // Enqueue notifications after ingestion (may be delayed while offline).
    if let Ok(spec_value) = serde_json::from_str::<serde_json::Value>(&spec_json)
        && let Ok(spec) = job_spec::parse_value(&spec_value)
        && job_spec::validate(&spec).is_ok()
    {
        match notifications::enqueue_for_run_spec(&state.db, &spec, &run.id).await {
            Ok(true) => state.notifications_notify.notify_one(),
            Ok(false) => {}
            Err(error) => {
                tracing::warn!(run_id = %run.id, error = %error, "failed to enqueue notifications for ingested run");
            }
        }
    }

    tracing::info!(
        agent_id = %agent_id,
        run_id = %run.id,
        job_id = %run.job_id,
        status = ?status,
        events = run.events.len(),
        inserted_events = inserted_events.len(),
        "agent run ingested"
    );
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub(super) struct AgentEnrollRequest {
    token: String,
    name: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct AgentEnrollResponse {
    agent_id: String,
    agent_key: String,
}

pub(super) async fn agent_enroll(
    state: axum::extract::State<AppState>,
    Json(req): Json<AgentEnrollRequest>,
) -> Result<Json<AgentEnrollResponse>, AppError> {
    let token_hash = agent::sha256_urlsafe_token(&req.token).map_err(|_| {
        AppError::unauthorized("invalid_token", "Invalid enrollment token")
            .with_details(json!({ "field": "token" }))
    })?;
    let now = time::OffsetDateTime::now_utc().unix_timestamp();

    let mut tx = state.db.begin().await?;
    let row = sqlx::query(
        "SELECT expires_at, remaining_uses FROM enrollment_tokens WHERE token_hash = ? LIMIT 1",
    )
    .bind(&token_hash)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(row) = row else {
        return Err(AppError::unauthorized(
            "invalid_token",
            "Invalid enrollment token",
        ));
    };

    let expires_at = row.get::<i64, _>("expires_at");
    let remaining_uses = row.get::<Option<i64>, _>("remaining_uses");

    if expires_at <= now {
        sqlx::query("DELETE FROM enrollment_tokens WHERE token_hash = ?")
            .bind(&token_hash)
            .execute(&mut *tx)
            .await?;
        return Err(AppError::unauthorized(
            "expired_token",
            "Enrollment token expired",
        ));
    }

    if let Some(uses) = remaining_uses {
        if uses <= 0 {
            return Err(AppError::unauthorized(
                "invalid_token",
                "Invalid enrollment token",
            ));
        }
        let new_uses = uses - 1;
        if new_uses == 0 {
            sqlx::query("DELETE FROM enrollment_tokens WHERE token_hash = ?")
                .bind(&token_hash)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query("UPDATE enrollment_tokens SET remaining_uses = ? WHERE token_hash = ?")
                .bind(new_uses)
                .bind(&token_hash)
                .execute(&mut *tx)
                .await?;
        }
    }

    let agent_id = uuid::Uuid::new_v4().to_string();
    let agent_key = agent::generate_token_b64_urlsafe(32);
    let agent_key_hash = agent::sha256_urlsafe_token(&agent_key)?;

    sqlx::query("INSERT INTO agents (id, name, key_hash, created_at) VALUES (?, ?, ?, ?)")
        .bind(&agent_id)
        .bind(req.name)
        .bind(agent_key_hash)
        .bind(now)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(AgentEnrollResponse {
        agent_id,
        agent_key,
    }))
}

pub(super) async fn agent_ws(
    state: axum::extract::State<AppState>,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let agent_id = authenticate_agent(&state.db, &headers).await?;

    let db = state.db.clone();
    let secrets = state.secrets.clone();
    let agent_manager = state.agent_manager.clone();
    let run_events_bus = state.run_events_bus.clone();
    Ok(ws.on_upgrade(move |socket| {
        handle_agent_socket(
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

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let header = headers.get("authorization")?.to_str().ok()?;
    let token = header.strip_prefix("Bearer ")?;
    Some(token.trim().to_string())
}

async fn authenticate_agent(db: &SqlitePool, headers: &HeaderMap) -> Result<String, AppError> {
    let agent_key = bearer_token(headers)
        .ok_or_else(|| AppError::unauthorized("unauthorized", "Unauthorized"))?;
    let key_hash = agent::sha256_urlsafe_token(&agent_key)
        .map_err(|_| AppError::unauthorized("unauthorized", "Unauthorized"))?;

    let row = sqlx::query("SELECT id, revoked_at FROM agents WHERE key_hash = ? LIMIT 1")
        .bind(key_hash)
        .fetch_optional(db)
        .await?;

    let Some(row) = row else {
        return Err(AppError::unauthorized("unauthorized", "Unauthorized"));
    };
    if row.get::<Option<i64>, _>("revoked_at").is_some() {
        return Err(AppError::unauthorized("revoked", "Agent revoked"));
    }

    Ok(row.get::<String, _>("id"))
}

async fn handle_agent_socket(
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
                    _ => {}
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

#[derive(Debug, Deserialize)]
struct WebdavSecretPayload {
    username: String,
    password: String,
}

async fn send_node_secrets_snapshot(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    node_id: &str,
) -> Result<(), anyhow::Error> {
    let list = secrets_repo::list_secrets(db, node_id, "webdav").await?;

    let mut webdav = Vec::with_capacity(list.len());
    for entry in list {
        let Some(bytes) =
            secrets_repo::get_secret(db, secrets, node_id, "webdav", &entry.name).await?
        else {
            continue;
        };
        let payload: WebdavSecretPayload = serde_json::from_slice(&bytes)?;
        webdav.push(WebdavSecretV1 {
            name: entry.name,
            username: payload.username,
            password: payload.password,
            updated_at: entry.updated_at,
        });
    }

    let msg = HubToAgentMessageV1::SecretsSnapshot {
        v: PROTOCOL_VERSION,
        node_id: node_id.to_string(),
        issued_at: time::OffsetDateTime::now_utc().unix_timestamp(),
        webdav,
    };

    agent_manager.send_json(node_id, &msg).await?;
    Ok(())
}

pub(super) async fn send_node_config_snapshot(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    node_id: &str,
) -> Result<(), anyhow::Error> {
    let jobs = jobs_repo::list_jobs_for_agent(db, node_id).await?;

    let mut configs = Vec::with_capacity(jobs.len());
    for job in jobs {
        let spec = match job_spec::parse_value(&job.spec) {
            Ok(v) => v,
            Err(error) => {
                tracing::warn!(
                    node_id = %node_id,
                    job_id = %job.id,
                    error = %error,
                    "invalid job spec; skipping agent config snapshot job"
                );
                continue;
            }
        };
        if let Err(error) = job_spec::validate(&spec) {
            tracing::warn!(
                node_id = %node_id,
                job_id = %job.id,
                error = %error,
                "invalid job spec; skipping agent config snapshot job"
            );
            continue;
        }

        let resolved =
            agent_job_resolver::resolve_job_spec_for_agent(db, secrets, node_id, spec).await?;

        let overlap_policy = match job.overlap_policy {
            jobs_repo::OverlapPolicy::Reject => OverlapPolicyV1::Reject,
            jobs_repo::OverlapPolicy::Queue => OverlapPolicyV1::Queue,
        };

        configs.push(JobConfigV1 {
            job_id: job.id,
            name: job.name,
            schedule: job.schedule,
            overlap_policy,
            updated_at: job.updated_at,
            spec: resolved,
        });
    }

    let msg = HubToAgentMessageV1::ConfigSnapshot {
        v: PROTOCOL_VERSION,
        node_id: node_id.to_string(),
        snapshot_id: agent::generate_token_b64_urlsafe(16),
        issued_at: time::OffsetDateTime::now_utc().unix_timestamp(),
        jobs: configs,
    };

    agent_manager.send_json(node_id, &msg).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::agent_online;

    #[test]
    fn agent_online_false_when_revoked() {
        assert!(!agent_online(true, Some(1000), 1000));
    }

    #[test]
    fn agent_online_false_when_never_seen() {
        assert!(!agent_online(false, None, 1000));
    }

    #[test]
    fn agent_online_false_when_stale() {
        assert!(!agent_online(false, Some(900), 1000));
    }

    #[test]
    fn agent_online_true_when_recent() {
        assert!(agent_online(false, Some(950), 1000));
    }
}
