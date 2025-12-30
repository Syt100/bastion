use std::sync::Arc;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::extract::Path;
use axum::extract::Request;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::SqlitePool;
use tower_cookies::CookieManagerLayer;
use tower_cookies::Cookies;
use tower_cookies::cookie::{Cookie, SameSite};
use tower_http::trace::TraceLayer;

use crate::agent;
use crate::auth;
use crate::config::Config;
use crate::jobs_repo;
use crate::runs_repo;
use crate::scheduler;
use crate::secrets::SecretsCrypto;
use crate::secrets_repo;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: SqlitePool,
    pub secrets: Arc<SecretsCrypto>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    ok: bool,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { ok: true })
}

#[derive(Debug, Serialize)]
struct SystemStatusResponse {
    version: &'static str,
    insecure_http: bool,
}

async fn system_status(state: axum::extract::State<AppState>) -> Json<SystemStatusResponse> {
    Json(SystemStatusResponse {
        version: env!("CARGO_PKG_VERSION"),
        insecure_http: state.config.insecure_http,
    })
}

#[derive(Debug, Serialize)]
struct SecretListItem {
    name: String,
    updated_at: i64,
}

async fn list_webdav_secrets(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<SecretListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let secrets = secrets_repo::list_secrets(&state.db, "webdav").await?;
    Ok(Json(
        secrets
            .into_iter()
            .map(|s| SecretListItem {
                name: s.name,
                updated_at: s.updated_at,
            })
            .collect(),
    ))
}

#[derive(Debug, Deserialize)]
struct UpsertWebdavSecretRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct WebdavSecretResponse {
    name: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebdavSecretPayload {
    username: String,
    password: String,
}

async fn upsert_webdav_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(req): Json<UpsertWebdavSecretRequest>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    if name.trim().is_empty() {
        return Err(AppError::bad_request(
            "invalid_name",
            "Secret name is required",
        ));
    }
    if req.username.trim().is_empty() {
        return Err(AppError::bad_request(
            "invalid_username",
            "Username is required",
        ));
    }

    let payload = WebdavSecretPayload {
        username: req.username.trim().to_string(),
        password: req.password,
    };
    let bytes = serde_json::to_vec(&payload)?;

    secrets_repo::upsert_secret(&state.db, &state.secrets, "webdav", name.trim(), &bytes).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_webdav_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(name): Path<String>,
) -> Result<Json<WebdavSecretResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let bytes = secrets_repo::get_secret(&state.db, &state.secrets, "webdav", &name)
        .await?
        .ok_or_else(|| AppError::not_found("secret_not_found", "Secret not found"))?;

    let payload: WebdavSecretPayload = serde_json::from_slice(&bytes)?;
    Ok(Json(WebdavSecretResponse {
        name,
        username: payload.username,
        password: payload.password,
    }))
}

async fn delete_webdav_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let deleted = secrets_repo::delete_secret(&state.db, "webdav", &name).await?;
    if !deleted {
        return Err(AppError::not_found("secret_not_found", "Secret not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn require_secure_middleware(
    state: axum::extract::State<AppState>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();
    let should_enforce = path.starts_with("/api/") || path.starts_with("/agent/");
    let allow_insecure = matches!(path, "/api/health" | "/api/system" | "/api/setup/status");

    if !should_enforce || allow_insecure {
        return next.run(req).await;
    }

    let peer_ip = peer.ip();
    let https = request_is_https(&state, req.headers(), peer_ip);
    let allow = https || state.config.insecure_http || peer_ip.is_loopback();
    if allow {
        return next.run(req).await;
    }

    AppError::bad_request(
        "insecure_not_allowed",
        "HTTPS required (reverse proxy) or start with --insecure-http",
    )
    .into_response()
}

#[derive(Debug, Serialize)]
struct SetupStatusResponse {
    needs_setup: bool,
}

async fn setup_status(
    state: axum::extract::State<AppState>,
) -> Result<Json<SetupStatusResponse>, AppError> {
    let count = auth::users_count(&state.db).await?;
    Ok(Json(SetupStatusResponse {
        needs_setup: count == 0,
    }))
}

#[derive(Debug, serde::Deserialize)]
struct SetupInitializeRequest {
    username: String,
    password: String,
}

async fn setup_initialize(
    state: axum::extract::State<AppState>,
    Json(req): Json<SetupInitializeRequest>,
) -> Result<StatusCode, AppError> {
    let count = auth::users_count(&state.db).await?;
    if count != 0 {
        return Err(AppError::conflict(
            "already_initialized",
            "Setup is already complete",
        ));
    }

    auth::create_user(&state.db, &req.username, &req.password).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, serde::Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    csrf_token: String,
}

async fn login(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let Some(user) = auth::find_user_by_username(&state.db, &req.username).await? else {
        return Err(AppError::unauthorized(
            "invalid_credentials",
            "Invalid credentials",
        ));
    };

    if !auth::verify_password(&user.password_hash, &req.password)? {
        return Err(AppError::unauthorized(
            "invalid_credentials",
            "Invalid credentials",
        ));
    }

    let session = auth::create_session(&state.db, user.id).await?;
    set_session_cookie(&state, &headers, peer.ip(), &cookies, &session.id)?;

    Ok(Json(LoginResponse {
        csrf_token: session.csrf_token,
    }))
}

async fn logout(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let session_id = cookies
        .get(SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string());

    let Some(session_id) = session_id else {
        return Ok(StatusCode::NO_CONTENT);
    };

    let session = auth::get_session(&state.db, &session_id).await?;
    let Some(session) = session else {
        let mut cookie = Cookie::new(SESSION_COOKIE_NAME, "");
        cookie.set_path("/");
        cookies.remove(cookie);
        return Ok(StatusCode::NO_CONTENT);
    };

    let csrf = headers
        .get(CSRF_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if csrf != session.csrf_token {
        return Err(AppError::unauthorized("invalid_csrf", "Invalid CSRF token"));
    }

    auth::delete_session(&state.db, &session_id).await?;
    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, "");
    cookie.set_path("/");
    cookies.remove(cookie);
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
struct SessionResponse {
    authenticated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    csrf_token: Option<String>,
}

async fn session(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<SessionResponse>, AppError> {
    let session_id = cookies
        .get(SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string());

    let Some(session_id) = session_id else {
        return Ok(Json(SessionResponse {
            authenticated: false,
            csrf_token: None,
        }));
    };

    let session = auth::get_session(&state.db, &session_id).await?;
    let Some(session) = session else {
        return Ok(Json(SessionResponse {
            authenticated: false,
            csrf_token: None,
        }));
    };

    Ok(Json(SessionResponse {
        authenticated: true,
        csrf_token: Some(session.csrf_token),
    }))
}

const SESSION_COOKIE_NAME: &str = "bastion_session";
const CSRF_HEADER: &str = "x-csrf-token";

async fn require_session(
    state: &AppState,
    cookies: &Cookies,
) -> Result<auth::SessionRow, AppError> {
    let session_id = cookies
        .get(SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::unauthorized("unauthorized", "Unauthorized"))?;

    let session = auth::get_session(&state.db, &session_id).await?;
    let Some(session) = session else {
        return Err(AppError::unauthorized("unauthorized", "Unauthorized"));
    };
    Ok(session)
}

fn require_csrf(headers: &HeaderMap, session: &auth::SessionRow) -> Result<(), AppError> {
    let csrf = headers
        .get(CSRF_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if csrf != session.csrf_token {
        return Err(AppError::unauthorized("invalid_csrf", "Invalid CSRF token"));
    }
    Ok(())
}

fn set_session_cookie(
    state: &AppState,
    headers: &HeaderMap,
    peer_ip: std::net::IpAddr,
    cookies: &Cookies,
    session_id: &str,
) -> Result<(), anyhow::Error> {
    let is_secure = request_is_https(state, headers, peer_ip);

    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, session_id.to_string());
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    cookie.set_secure(is_secure);

    cookies.add(cookie);
    Ok(())
}

fn request_is_https(state: &AppState, headers: &HeaderMap, peer_ip: std::net::IpAddr) -> bool {
    if state.config.insecure_http {
        return false;
    }

    let trusted = state
        .config
        .trusted_proxies
        .iter()
        .any(|net| net.contains(&peer_ip));
    if !trusted {
        return false;
    }

    let Some(proto) = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };

    proto.eq_ignore_ascii_case("https")
}

#[derive(Debug, Deserialize)]
struct CreateEnrollmentTokenRequest {
    #[serde(default = "default_enroll_ttl_seconds")]
    ttl_seconds: i64,
    remaining_uses: Option<i64>,
}

fn default_enroll_ttl_seconds() -> i64 {
    60 * 60
}

#[derive(Debug, Serialize)]
struct CreateEnrollmentTokenResponse {
    token: String,
    expires_at: i64,
    remaining_uses: Option<i64>,
}

async fn create_enrollment_token(
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
struct AgentListItem {
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

async fn list_agents(
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

async fn revoke_agent(
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

fn validate_job_spec(spec: &serde_json::Value) -> Result<(), AppError> {
    let Some(obj) = spec.as_object() else {
        return Err(AppError::bad_request(
            "invalid_spec",
            "Job spec must be an object",
        ));
    };

    let v = obj
        .get("v")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| AppError::bad_request("invalid_spec", "Job spec must include integer v"))?;
    if v != 1 {
        return Err(AppError::bad_request(
            "invalid_spec",
            "Unsupported job spec version",
        ));
    }

    let ty = obj.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
        AppError::bad_request("invalid_spec", "Job spec must include string type")
    })?;
    if !matches!(ty, "filesystem" | "sqlite" | "vaultwarden") {
        return Err(AppError::bad_request(
            "invalid_spec",
            "Unsupported job spec type",
        ));
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct CreateJobRequest {
    name: String,
    schedule: Option<String>,
    overlap_policy: jobs_repo::OverlapPolicy,
    spec: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct UpdateJobRequest {
    name: String,
    schedule: Option<String>,
    overlap_policy: jobs_repo::OverlapPolicy,
    spec: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct JobListItem {
    id: String,
    name: String,
    schedule: Option<String>,
    overlap_policy: jobs_repo::OverlapPolicy,
    created_at: i64,
    updated_at: i64,
}

async fn list_jobs(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<JobListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let jobs = jobs_repo::list_jobs(&state.db).await?;

    Ok(Json(
        jobs.into_iter()
            .map(|j| JobListItem {
                id: j.id,
                name: j.name,
                schedule: j.schedule,
                overlap_policy: j.overlap_policy,
                created_at: j.created_at,
                updated_at: j.updated_at,
            })
            .collect(),
    ))
}

async fn create_job(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Json(req): Json<CreateJobRequest>,
) -> Result<Json<jobs_repo::Job>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    if req.name.trim().is_empty() {
        return Err(AppError::bad_request(
            "invalid_name",
            "Job name is required",
        ));
    }

    let schedule = req
        .schedule
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string());

    validate_job_spec(&req.spec)?;

    if let Some(schedule) = schedule.as_deref() {
        scheduler::validate_cron(schedule)
            .map_err(|_| AppError::bad_request("invalid_schedule", "Invalid cron schedule"))?;
    }

    let job = jobs_repo::create_job(
        &state.db,
        req.name.trim(),
        schedule.as_deref(),
        req.overlap_policy,
        req.spec,
    )
    .await?;

    Ok(Json(job))
}

async fn get_job(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(job_id): Path<String>,
) -> Result<Json<jobs_repo::Job>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let job = jobs_repo::get_job(&state.db, &job_id)
        .await?
        .ok_or_else(|| AppError::not_found("job_not_found", "Job not found"))?;
    Ok(Json(job))
}

async fn update_job(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Json(req): Json<UpdateJobRequest>,
) -> Result<Json<jobs_repo::Job>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    if req.name.trim().is_empty() {
        return Err(AppError::bad_request(
            "invalid_name",
            "Job name is required",
        ));
    }

    let schedule = req
        .schedule
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string());

    validate_job_spec(&req.spec)?;

    if let Some(schedule) = schedule.as_deref() {
        scheduler::validate_cron(schedule)
            .map_err(|_| AppError::bad_request("invalid_schedule", "Invalid cron schedule"))?;
    }

    let updated = jobs_repo::update_job(
        &state.db,
        &job_id,
        req.name.trim(),
        schedule.as_deref(),
        req.overlap_policy,
        req.spec,
    )
    .await?;
    if !updated {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }

    let job = jobs_repo::get_job(&state.db, &job_id)
        .await?
        .ok_or_else(|| AppError::not_found("job_not_found", "Job not found"))?;
    Ok(Json(job))
}

async fn delete_job(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let deleted = jobs_repo::delete_job(&state.db, &job_id).await?;
    if !deleted {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
struct TriggerRunResponse {
    run_id: String,
    status: runs_repo::RunStatus,
}

async fn trigger_job_run(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> Result<Json<TriggerRunResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let job = jobs_repo::get_job(&state.db, &job_id)
        .await?
        .ok_or_else(|| AppError::not_found("job_not_found", "Job not found"))?;

    let running_count = sqlx::query(
        "SELECT COUNT(1) AS n FROM runs WHERE job_id = ? AND status IN ('running', 'queued')",
    )
    .bind(&job.id)
    .fetch_one(&state.db)
    .await?
    .get::<i64, _>("n");

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let (status, ended_at, error) =
        if job.overlap_policy == jobs_repo::OverlapPolicy::Reject && running_count > 0 {
            (
                runs_repo::RunStatus::Rejected,
                Some(now),
                Some("overlap_rejected"),
            )
        } else {
            (runs_repo::RunStatus::Queued, None, None)
        };

    let run = runs_repo::create_run(&state.db, &job.id, status, now, ended_at, None, error).await?;

    let event_kind = match status {
        runs_repo::RunStatus::Rejected => "rejected",
        runs_repo::RunStatus::Queued => "queued",
        _ => "unknown",
    };
    runs_repo::append_run_event(
        &state.db,
        &run.id,
        "info",
        event_kind,
        event_kind,
        Some(serde_json::json!({ "source": "manual" })),
    )
    .await?;

    Ok(Json(TriggerRunResponse {
        run_id: run.id,
        status: run.status,
    }))
}

#[derive(Debug, Serialize)]
struct RunListItem {
    id: String,
    status: runs_repo::RunStatus,
    started_at: i64,
    ended_at: Option<i64>,
    error: Option<String>,
}

async fn list_job_runs(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(job_id): Path<String>,
) -> Result<Json<Vec<RunListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let job_exists = jobs_repo::get_job(&state.db, &job_id).await?.is_some();
    if !job_exists {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }

    let runs = runs_repo::list_runs_for_job(&state.db, &job_id, 50).await?;
    Ok(Json(
        runs.into_iter()
            .map(|r| RunListItem {
                id: r.id,
                status: r.status,
                started_at: r.started_at,
                ended_at: r.ended_at,
                error: r.error,
            })
            .collect(),
    ))
}

async fn list_run_events(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(run_id): Path<String>,
) -> Result<Json<Vec<runs_repo::RunEvent>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let events = runs_repo::list_run_events(&state.db, &run_id, 500).await?;
    Ok(Json(events))
}

#[derive(Debug, Deserialize)]
struct AgentEnrollRequest {
    token: String,
    name: Option<String>,
}

#[derive(Debug, Serialize)]
struct AgentEnrollResponse {
    agent_id: String,
    agent_key: String,
}

async fn agent_enroll(
    state: axum::extract::State<AppState>,
    Json(req): Json<AgentEnrollRequest>,
) -> Result<Json<AgentEnrollResponse>, AppError> {
    let token_hash = agent::sha256_urlsafe_token(&req.token)?;
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

async fn agent_ws(
    state: axum::extract::State<AppState>,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let agent_key = bearer_token(&headers)
        .ok_or_else(|| AppError::unauthorized("unauthorized", "Unauthorized"))?;
    let key_hash = agent::sha256_urlsafe_token(&agent_key)?;

    let row = sqlx::query("SELECT id, revoked_at FROM agents WHERE key_hash = ? LIMIT 1")
        .bind(key_hash)
        .fetch_optional(&state.db)
        .await?;

    let Some(row) = row else {
        return Err(AppError::unauthorized("unauthorized", "Unauthorized"));
    };
    if row.get::<Option<i64>, _>("revoked_at").is_some() {
        return Err(AppError::unauthorized("revoked", "Agent revoked"));
    }

    let agent_id = row.get::<String, _>("id");

    let db = state.db.clone();
    Ok(ws.on_upgrade(move |socket| handle_agent_socket(db, agent_id, peer.ip(), socket)))
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let header = headers.get("authorization")?.to_str().ok()?;
    let token = header.strip_prefix("Bearer ")?;
    Some(token.trim().to_string())
}

async fn handle_agent_socket(
    db: SqlitePool,
    agent_id: String,
    peer_ip: std::net::IpAddr,
    mut socket: WebSocket,
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

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                let text = text.to_string();
                if let Err(error) = sqlx::query(
                    "UPDATE agents SET capabilities_json = ?, last_seen_at = ? WHERE id = ?",
                )
                .bind(text.clone())
                .bind(time::OffsetDateTime::now_utc().unix_timestamp())
                .bind(&agent_id)
                .execute(&db)
                .await
                {
                    tracing::warn!(agent_id = %agent_id, error = %error, "failed to update agent capabilities");
                }

                let _ = socket
                    .send(Message::Text(r#"{"v":1,"type":"ack"}"#.into()))
                    .await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    tracing::info!(agent_id = %agent_id, "agent disconnected");
}

#[derive(Debug)]
struct AppError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl AppError {
    fn bad_request(code: &'static str, message: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: message.to_string(),
        }
    }

    fn unauthorized(code: &'static str, message: &'static str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code,
            message: message.to_string(),
        }
    }

    fn conflict(code: &'static str, message: &'static str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code,
            message: message.to_string(),
        }
    }

    fn not_found(code: &'static str, message: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code,
            message: message.to_string(),
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        let error: anyhow::Error = error.into();
        tracing::error!(error = %error, "request failed");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: "Internal server error".to_string(),
        }
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct Body<'a> {
            error: &'a str,
            message: &'a str,
        }

        let body = Json(Body {
            error: self.code,
            message: &self.message,
        });
        (self.status, body).into_response()
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/system", get(system_status))
        .route("/api/setup/status", get(setup_status))
        .route("/api/setup/initialize", post(setup_initialize))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/session", get(session))
        .route("/api/secrets/webdav", get(list_webdav_secrets))
        .route(
            "/api/secrets/webdav/{name}",
            get(get_webdav_secret)
                .put(upsert_webdav_secret)
                .delete(delete_webdav_secret),
        )
        .route("/api/agents", get(list_agents))
        .route("/api/agents/{id}/revoke", post(revoke_agent))
        .route(
            "/api/agents/enrollment-tokens",
            post(create_enrollment_token),
        )
        .route("/api/jobs", get(list_jobs).post(create_job))
        .route(
            "/api/jobs/{id}",
            get(get_job).put(update_job).delete(delete_job),
        )
        .route("/api/jobs/{id}/run", post(trigger_job_run))
        .route("/api/jobs/{id}/runs", get(list_job_runs))
        .route("/api/runs/{id}/events", get(list_run_events))
        .route("/agent/enroll", post(agent_enroll))
        .route("/agent/ws", get(agent_ws))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            require_secure_middleware,
        ))
        .layer(CookieManagerLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        .fallback(ui_fallback)
}

async fn ui_fallback(method: Method, uri: Uri) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return StatusCode::NOT_FOUND.into_response();
    }

    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    let bytes = match load_ui_asset(path).or_else(|| load_ui_asset("index.html")) {
        Some(bytes) => bytes,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, mime.as_ref())
        .body(Body::from(bytes))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

#[cfg(feature = "embed-ui")]
fn load_ui_asset(path: &str) -> Option<Vec<u8>> {
    static UI_DIST: include_dir::Dir<'static> =
        include_dir::include_dir!("$CARGO_MANIFEST_DIR/../../ui/dist");
    let file = UI_DIST.get_file(path)?;
    Some(file.contents().to_vec())
}

#[cfg(not(feature = "embed-ui"))]
fn load_ui_asset(path: &str) -> Option<Vec<u8>> {
    use std::fs;
    use std::path::PathBuf;

    let base = std::env::var("BASTION_UI_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("ui/dist"));
    fs::read(base.join(path)).ok()
}
