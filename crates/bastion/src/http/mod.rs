use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use sqlx::SqlitePool;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;

use crate::agent_manager::AgentManager;
use crate::config::Config;
use crate::run_events_bus::RunEventsBus;
use crate::secrets::SecretsCrypto;

mod agents;
mod auth;
mod error;
mod jobs;
mod middleware;
mod operations;
mod secrets;
mod shared;

use error::AppError;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: SqlitePool,
    pub secrets: Arc<SecretsCrypto>,
    pub agent_manager: AgentManager,
    pub run_events_bus: Arc<RunEventsBus>,
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

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/system", get(system_status))
        .route("/api/setup/status", get(auth::setup_status))
        .route("/api/setup/initialize", post(auth::setup_initialize))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/session", get(auth::session))
        .route("/api/secrets/webdav", get(secrets::list_webdav_secrets))
        .route(
            "/api/secrets/webdav/{name}",
            get(secrets::get_webdav_secret)
                .put(secrets::upsert_webdav_secret)
                .delete(secrets::delete_webdav_secret),
        )
        .route(
            "/api/secrets/wecom-bot",
            get(secrets::list_wecom_bot_secrets),
        )
        .route(
            "/api/secrets/wecom-bot/{name}",
            get(secrets::get_wecom_bot_secret)
                .put(secrets::upsert_wecom_bot_secret)
                .delete(secrets::delete_wecom_bot_secret),
        )
        .route("/api/secrets/smtp", get(secrets::list_smtp_secrets))
        .route(
            "/api/secrets/smtp/{name}",
            get(secrets::get_smtp_secret)
                .put(secrets::upsert_smtp_secret)
                .delete(secrets::delete_smtp_secret),
        )
        .route("/api/agents", get(agents::list_agents))
        .route("/api/agents/{id}/revoke", post(agents::revoke_agent))
        .route(
            "/api/agents/{id}/rotate-key",
            post(agents::rotate_agent_key),
        )
        .route(
            "/api/agents/enrollment-tokens",
            post(agents::create_enrollment_token),
        )
        .route("/api/jobs", get(jobs::list_jobs).post(jobs::create_job))
        .route(
            "/api/jobs/{id}",
            get(jobs::get_job)
                .put(jobs::update_job)
                .delete(jobs::delete_job),
        )
        .route("/api/jobs/{id}/run", post(jobs::trigger_job_run))
        .route("/api/jobs/{id}/runs", get(jobs::list_job_runs))
        .route("/api/runs/{id}/events", get(jobs::list_run_events))
        .route("/api/runs/{id}/events/ws", get(jobs::run_events_ws))
        .route("/api/runs/{id}/restore", post(operations::start_restore))
        .route("/api/runs/{id}/verify", post(operations::start_verify))
        .route("/api/operations/{id}", get(operations::get_operation))
        .route(
            "/api/operations/{id}/events",
            get(operations::list_operation_events),
        )
        .route("/agent/enroll", post(agents::agent_enroll))
        .route("/agent/ws", get(agents::agent_ws))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::require_secure_middleware,
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

#[cfg(test)]
mod ws_tests;
