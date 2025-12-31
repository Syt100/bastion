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
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::SqlitePool;
use tower_cookies::CookieManagerLayer;
use tower_cookies::Cookies;
use tower_cookies::cookie::{Cookie, SameSite};
use tower_http::trace::TraceLayer;

use crate::agent;
use crate::agent_manager::AgentManager;
use crate::agent_protocol::{AgentToHubMessageV1, HubToAgentMessageV1, PROTOCOL_VERSION};
use crate::agent_tasks_repo;
use crate::auth;
use crate::config::Config;
use crate::job_spec;
use crate::jobs_repo;
use crate::operations_repo;
use crate::restore;
use crate::runs_repo;
use crate::scheduler;
use crate::secrets::SecretsCrypto;
use crate::secrets_repo;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: SqlitePool,
    pub secrets: Arc<SecretsCrypto>,
    pub agent_manager: AgentManager,
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

async fn list_wecom_bot_secrets(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<SecretListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let secrets = secrets_repo::list_secrets(&state.db, "wecom_bot").await?;
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

#[derive(Debug, Deserialize)]
struct UpsertWecomBotSecretRequest {
    webhook_url: String,
}

#[derive(Debug, Serialize)]
struct WecomBotSecretResponse {
    name: String,
    webhook_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WecomBotSecretPayload {
    webhook_url: String,
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

async fn upsert_wecom_bot_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(req): Json<UpsertWecomBotSecretRequest>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    if name.trim().is_empty() {
        return Err(AppError::bad_request(
            "invalid_name",
            "Secret name is required",
        ));
    }

    let webhook_url = req.webhook_url.trim();
    if webhook_url.is_empty() {
        return Err(AppError::bad_request(
            "invalid_webhook_url",
            "Webhook URL is required",
        ));
    }
    let url = url::Url::parse(webhook_url)?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AppError::bad_request(
            "invalid_webhook_url",
            "Webhook URL must be http(s)",
        ));
    }

    let payload = WecomBotSecretPayload {
        webhook_url: webhook_url.to_string(),
    };
    let bytes = serde_json::to_vec(&payload)?;

    secrets_repo::upsert_secret(&state.db, &state.secrets, "wecom_bot", name.trim(), &bytes)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_wecom_bot_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(name): Path<String>,
) -> Result<Json<WecomBotSecretResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let bytes = secrets_repo::get_secret(&state.db, &state.secrets, "wecom_bot", &name)
        .await?
        .ok_or_else(|| AppError::not_found("secret_not_found", "Secret not found"))?;

    let payload: WecomBotSecretPayload = serde_json::from_slice(&bytes)?;
    Ok(Json(WecomBotSecretResponse {
        name,
        webhook_url: payload.webhook_url,
    }))
}

async fn delete_wecom_bot_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let deleted = secrets_repo::delete_secret(&state.db, "wecom_bot", &name).await?;
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
    let allow_insecure = matches!(path, "/api/health" | "/api/system" | "/api/setup/status");

    if allow_insecure {
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
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let client_ip = effective_client_ip(&state, &headers, peer.ip());
    let client_ip_str = client_ip.to_string();

    if let Some(retry_after) =
        auth::login_throttle_retry_after_seconds(&state.db, &client_ip_str, now).await?
    {
        return Err(AppError::too_many_requests(
            "rate_limited",
            format!("Too many login attempts. Retry after {retry_after}s."),
        ));
    }

    let Some(user) = auth::find_user_by_username(&state.db, &req.username).await? else {
        let _ = auth::record_login_failure(&state.db, &client_ip_str, now).await;
        return Err(AppError::unauthorized(
            "invalid_credentials",
            "Invalid credentials",
        ));
    };

    if !auth::verify_password(&user.password_hash, &req.password)? {
        let _ = auth::record_login_failure(&state.db, &client_ip_str, now).await;
        return Err(AppError::unauthorized(
            "invalid_credentials",
            "Invalid credentials",
        ));
    }

    let _ = auth::clear_login_throttle(&state.db, &client_ip_str).await;

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

    if !is_trusted_proxy(state, peer_ip) {
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

fn is_trusted_proxy(state: &AppState, peer_ip: std::net::IpAddr) -> bool {
    state
        .config
        .trusted_proxies
        .iter()
        .any(|net| net.contains(&peer_ip))
}

fn effective_client_ip(
    state: &AppState,
    headers: &HeaderMap,
    peer_ip: std::net::IpAddr,
) -> std::net::IpAddr {
    if !is_trusted_proxy(state, peer_ip) {
        return peer_ip;
    }

    let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) else {
        return peer_ip;
    };

    let first = xff.split(',').next().unwrap_or("").trim();
    first.parse().unwrap_or(peer_ip)
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
    job_spec::validate_value(spec).map_err(|error| {
        AppError::bad_request("invalid_spec", format!("Invalid job spec: {error}"))
    })
}

#[derive(Debug, Deserialize)]
struct CreateJobRequest {
    name: String,
    agent_id: Option<String>,
    schedule: Option<String>,
    overlap_policy: jobs_repo::OverlapPolicy,
    spec: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct UpdateJobRequest {
    name: String,
    agent_id: Option<String>,
    schedule: Option<String>,
    overlap_policy: jobs_repo::OverlapPolicy,
    spec: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct JobListItem {
    id: String,
    name: String,
    agent_id: Option<String>,
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
                agent_id: j.agent_id,
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

    let agent_id = req
        .agent_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    if let Some(agent_id) = agent_id {
        let row = sqlx::query("SELECT revoked_at FROM agents WHERE id = ? LIMIT 1")
            .bind(agent_id)
            .fetch_optional(&state.db)
            .await?;

        let Some(row) = row else {
            return Err(AppError::bad_request("invalid_agent_id", "Agent not found"));
        };
        if row.get::<Option<i64>, _>("revoked_at").is_some() {
            return Err(AppError::bad_request(
                "invalid_agent_id",
                "Agent is revoked",
            ));
        }
    }

    validate_job_spec(&req.spec)?;

    if let Some(schedule) = schedule.as_deref() {
        scheduler::validate_cron(schedule)
            .map_err(|_| AppError::bad_request("invalid_schedule", "Invalid cron schedule"))?;
    }

    let job = jobs_repo::create_job(
        &state.db,
        req.name.trim(),
        agent_id,
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

    let agent_id = req
        .agent_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    if let Some(agent_id) = agent_id {
        let row = sqlx::query("SELECT revoked_at FROM agents WHERE id = ? LIMIT 1")
            .bind(agent_id)
            .fetch_optional(&state.db)
            .await?;

        let Some(row) = row else {
            return Err(AppError::bad_request("invalid_agent_id", "Agent not found"));
        };
        if row.get::<Option<i64>, _>("revoked_at").is_some() {
            return Err(AppError::bad_request(
                "invalid_agent_id",
                "Agent is revoked",
            ));
        }
    }

    validate_job_spec(&req.spec)?;

    if let Some(schedule) = schedule.as_deref() {
        scheduler::validate_cron(schedule)
            .map_err(|_| AppError::bad_request("invalid_schedule", "Invalid cron schedule"))?;
    }

    let updated = jobs_repo::update_job(
        &state.db,
        &job_id,
        req.name.trim(),
        agent_id,
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

async fn run_events_ws(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    Path(run_id): Path<String>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let _session = require_session(&state, &cookies).await?;
    require_ws_same_origin(&state, &headers, peer.ip())?;

    let run_exists = runs_repo::get_run(&state.db, &run_id).await?.is_some();
    if !run_exists {
        return Err(AppError::not_found("run_not_found", "Run not found"));
    }

    let db = state.db.clone();
    Ok(ws.on_upgrade(move |socket| handle_run_events_socket(db, run_id, socket)))
}

fn require_ws_same_origin(
    state: &AppState,
    headers: &HeaderMap,
    peer_ip: std::net::IpAddr,
) -> Result<(), AppError> {
    let origin = headers
        .get(axum::http::header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::unauthorized("invalid_origin", "Invalid origin"))?;

    let expected_host = if is_trusted_proxy(state, peer_ip) {
        headers
            .get("x-forwarded-host")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(|s| s.trim().to_string())
            .or_else(|| {
                headers
                    .get(axum::http::header::HOST)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
            })
    } else {
        headers
            .get(axum::http::header::HOST)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }
    .ok_or_else(|| AppError::unauthorized("invalid_origin", "Invalid origin"))?;

    let expected_host = expected_host
        .split(':')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();

    let origin_host = match url::Url::parse(origin) {
        Ok(url) => url.host_str().unwrap_or("").to_ascii_lowercase(),
        Err(_) => return Err(AppError::unauthorized("invalid_origin", "Invalid origin")),
    };

    if origin_host != expected_host {
        return Err(AppError::unauthorized("invalid_origin", "Invalid origin"));
    }

    Ok(())
}

async fn handle_run_events_socket(db: SqlitePool, run_id: String, mut socket: WebSocket) {
    let mut last_seq = 0i64;
    let mut idle_after_end = 0u32;

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {
                let events = match runs_repo::list_run_events_after_seq(&db, &run_id, last_seq, 200).await {
                    Ok(v) => v,
                    Err(_) => break,
                };

                if events.is_empty() {
                    match runs_repo::get_run(&db, &run_id).await {
                        Ok(Some(run)) => {
                            let ended = !matches!(run.status, runs_repo::RunStatus::Queued | runs_repo::RunStatus::Running);
                            if ended {
                                idle_after_end += 1;
                                if idle_after_end >= 10 {
                                    break;
                                }
                            } else {
                                idle_after_end = 0;
                            }
                        }
                        Ok(None) | Err(_) => break,
                    }
                    continue;
                }

                idle_after_end = 0;
                for event in events {
                    last_seq = event.seq;
                    let payload = match serde_json::to_string(&event) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    if socket.send(Message::Text(payload.into())).await.is_err() {
                        return;
                    }
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct StartRestoreRequest {
    destination_dir: String,
    conflict_policy: String,
}

#[derive(Debug, Serialize)]
struct StartOperationResponse {
    op_id: String,
}

async fn start_restore(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(run_id): Path<String>,
    Json(req): Json<StartRestoreRequest>,
) -> Result<Json<StartOperationResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let destination_dir = req.destination_dir.trim();
    if destination_dir.is_empty() {
        return Err(AppError::bad_request(
            "invalid_destination",
            "destination_dir is required",
        ));
    }

    let conflict = req
        .conflict_policy
        .parse::<restore::ConflictPolicy>()
        .map_err(|_| AppError::bad_request("invalid_conflict_policy", "Invalid conflict policy"))?;

    let run = runs_repo::get_run(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;
    if run.status != runs_repo::RunStatus::Success {
        return Err(AppError::bad_request(
            "invalid_run",
            "Run is not successful",
        ));
    }

    let op = operations_repo::create_operation(&state.db, operations_repo::OperationKind::Restore)
        .await?;
    let _ = operations_repo::append_event(
        &state.db,
        &op.id,
        "info",
        "requested",
        "requested",
        Some(serde_json::json!({
            "run_id": run_id,
            "destination_dir": destination_dir,
            "conflict_policy": conflict.as_str(),
        })),
    )
    .await;

    restore::spawn_restore_operation(
        state.db.clone(),
        state.secrets.clone(),
        state.config.data_dir.clone(),
        op.id.clone(),
        run_id,
        std::path::PathBuf::from(destination_dir),
        conflict,
    )
    .await;

    Ok(Json(StartOperationResponse { op_id: op.id }))
}

async fn start_verify(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(run_id): Path<String>,
) -> Result<Json<StartOperationResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let run = runs_repo::get_run(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;
    if run.status != runs_repo::RunStatus::Success {
        return Err(AppError::bad_request(
            "invalid_run",
            "Run is not successful",
        ));
    }

    let op = operations_repo::create_operation(&state.db, operations_repo::OperationKind::Verify)
        .await?;
    let _ = operations_repo::append_event(
        &state.db,
        &op.id,
        "info",
        "requested",
        "requested",
        Some(serde_json::json!({ "run_id": run_id })),
    )
    .await;

    restore::spawn_verify_operation(
        state.db.clone(),
        state.secrets.clone(),
        state.config.data_dir.clone(),
        op.id.clone(),
        run_id,
    )
    .await;

    Ok(Json(StartOperationResponse { op_id: op.id }))
}

#[derive(Debug, Serialize)]
struct OperationResponse {
    id: String,
    kind: operations_repo::OperationKind,
    status: operations_repo::OperationStatus,
    created_at: i64,
    started_at: i64,
    ended_at: Option<i64>,
    summary: Option<serde_json::Value>,
    error: Option<String>,
}

async fn get_operation(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(op_id): Path<String>,
) -> Result<Json<OperationResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let op = operations_repo::get_operation(&state.db, &op_id)
        .await?
        .ok_or_else(|| AppError::not_found("operation_not_found", "Operation not found"))?;
    Ok(Json(OperationResponse {
        id: op.id,
        kind: op.kind,
        status: op.status,
        created_at: op.created_at,
        started_at: op.started_at,
        ended_at: op.ended_at,
        summary: op.summary,
        error: op.error,
    }))
}

async fn list_operation_events(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(op_id): Path<String>,
) -> Result<Json<Vec<operations_repo::OperationEvent>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let events = operations_repo::list_events(&state.db, &op_id, 500).await?;
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
    let agent_manager = state.agent_manager.clone();
    Ok(ws.on_upgrade(move |socket| {
        handle_agent_socket(db, agent_id, peer.ip(), agent_manager, socket)
    }))
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
    agent_manager: AgentManager,
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
                        let _ = runs_repo::append_run_event(
                            &db, &run_id, &level, &kind, &message, fields,
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
                        if let Some(run) = run {
                            if run.status == runs_repo::RunStatus::Running {
                                let (run_status, err_code) = if status == "success" {
                                    (runs_repo::RunStatus::Success, None)
                                } else {
                                    (runs_repo::RunStatus::Failed, Some("agent_failed"))
                                };

                                let _ = runs_repo::complete_run(
                                    &db,
                                    &run_id,
                                    run_status,
                                    summary.clone(),
                                    err_code,
                                )
                                .await;
                                let _ = runs_repo::append_run_event(
                                    &db,
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

#[derive(Debug)]
struct AppError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl AppError {
    fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: message.into(),
        }
    }

    fn too_many_requests(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            code,
            message: message.into(),
        }
    }

    fn unauthorized(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code,
            message: message.into(),
        }
    }

    fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code,
            message: message.into(),
        }
    }

    fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code,
            message: message.into(),
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
        .route("/api/secrets/wecom-bot", get(list_wecom_bot_secrets))
        .route(
            "/api/secrets/wecom-bot/{name}",
            get(get_wecom_bot_secret)
                .put(upsert_wecom_bot_secret)
                .delete(delete_wecom_bot_secret),
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
        .route("/api/runs/{id}/events/ws", get(run_events_ws))
        .route("/api/runs/{id}/restore", post(start_restore))
        .route("/api/runs/{id}/verify", post(start_verify))
        .route("/api/operations/{id}", get(get_operation))
        .route("/api/operations/{id}/events", get(list_operation_events))
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

    let (bytes, served_path) = if let Some(bytes) = load_ui_asset(path) {
        (bytes, path)
    } else if let Some(bytes) = load_ui_asset("index.html") {
        (bytes, "index.html")
    } else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let mime = mime_guess::from_path(served_path).first_or_octet_stream();
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
