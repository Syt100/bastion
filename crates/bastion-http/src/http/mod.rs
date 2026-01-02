use std::sync::Arc;

use axum::body::Body;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use sqlx::SqlitePool;
use tokio::sync::Notify;
use tower_cookies::CookieManagerLayer;
use tower_http::request_id::{
    MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use tower_http::trace::TraceLayer;

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;
use bastion_engine::run_events_bus::RunEventsBus;
use bastion_storage::secrets::SecretsCrypto;

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
    pub run_queue_notify: Arc<Notify>,
    pub jobs_notify: Arc<Notify>,
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
    let request_id_header = axum::http::HeaderName::from_static("x-request-id");
    let trace_layer =
        TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
            let request_id = request
                .extensions()
                .get::<RequestId>()
                .and_then(|v| v.header_value().to_str().ok())
                .unwrap_or("-");
            tracing::info_span!(
                "http.request",
                request_id = %request_id,
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
            )
        });

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
        .layer(trace_layer)
        .layer(PropagateRequestIdLayer::new(request_id_header.clone()))
        .layer(SetRequestIdLayer::new(request_id_header, MakeRequestUuid))
        .with_state(state)
        .fallback(ui_fallback)
}

async fn ui_fallback(method: Method, uri: Uri, headers: HeaderMap) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return StatusCode::NOT_FOUND.into_response();
    }

    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    if !is_safe_ui_path(path) {
        return StatusCode::NOT_FOUND.into_response();
    }

    #[cfg(feature = "embed-ui")]
    {
        ui_fallback_embed(method, path, &headers)
    }

    #[cfg(not(feature = "embed-ui"))]
    {
        ui_fallback_fs(method, path, &headers).await
    }
}

#[cfg(feature = "embed-ui")]
fn ui_fallback_embed(method: Method, path: &str, headers: &HeaderMap) -> Response {
    let (served_path, bytes) = if let Some(bytes) = load_ui_asset_bytes(path) {
        (path, bytes)
    } else if let Some(bytes) = load_ui_asset_bytes("index.html") {
        ("index.html", bytes)
    } else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let etag = embed_etag(bytes);
    if etag_matches(headers, &etag) {
        return ui_not_modified(&etag, served_path);
    }

    let body = if method == Method::HEAD {
        Body::empty()
    } else {
        Body::from(bytes.to_vec())
    };

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::ETAG, &etag)
        .body(body)
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    with_ui_headers(resp, served_path, &etag)
}

#[cfg(not(feature = "embed-ui"))]
async fn ui_fallback_fs(method: Method, path: &str, headers: &HeaderMap) -> Response {
    use tokio_util::io::ReaderStream;

    let (served_path, file, meta) = if let Some((file, meta)) = try_open_ui_file(path).await {
        (path, file, meta)
    } else if let Some((file, meta)) = try_open_ui_file("index.html").await {
        ("index.html", file, meta)
    } else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let etag = file_etag(&meta);
    if etag_matches(headers, &etag) {
        return ui_not_modified(&etag, served_path);
    }

    let len = meta.len();
    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::ETAG, &etag);
    if len > 0 {
        resp = resp.header(axum::http::header::CONTENT_LENGTH, len.to_string());
    }

    let body = if method == Method::HEAD {
        Body::empty()
    } else {
        Body::from_stream(ReaderStream::new(file))
    };

    let resp = resp
        .body(body)
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    with_ui_headers(resp, served_path, &etag)
}

#[cfg(feature = "embed-ui")]
fn load_ui_asset_bytes(path: &str) -> Option<&'static [u8]> {
    static UI_DIST: include_dir::Dir<'static> =
        include_dir::include_dir!("$CARGO_MANIFEST_DIR/../../ui/dist");
    let file = UI_DIST.get_file(path)?;
    Some(file.contents())
}

#[cfg(not(feature = "embed-ui"))]
async fn try_open_ui_file(path: &str) -> Option<(tokio::fs::File, std::fs::Metadata)> {
    use std::path::PathBuf;

    let base = std::env::var("BASTION_UI_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("ui/dist"));
    let full = base.join(path);
    let file = tokio::fs::File::open(full).await.ok()?;
    let meta = file.metadata().await.ok()?;
    Some((file, meta))
}

fn is_safe_ui_path(path: &str) -> bool {
    use std::path::{Component, Path};
    Path::new(path)
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
}

fn cache_control_for_ui_path(path: &str) -> &'static str {
    if path == "index.html" {
        "no-cache"
    } else if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    }
}

fn etag_matches(headers: &HeaderMap, etag: &str) -> bool {
    let Some(value) = headers.get(axum::http::header::IF_NONE_MATCH) else {
        return false;
    };
    let Ok(value) = value.to_str() else {
        return false;
    };
    value.split(',').any(|v| v.trim() == etag)
}

fn ui_not_modified(etag: &str, served_path: &str) -> Response {
    let resp = Response::builder()
        .status(StatusCode::NOT_MODIFIED)
        .header(axum::http::header::ETAG, etag)
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    with_ui_headers(resp, served_path, etag)
}

fn with_ui_headers(mut resp: Response, served_path: &str, etag: &str) -> Response {
    let mime = mime_guess::from_path(served_path).first_or_octet_stream();
    resp.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_str(mime.as_ref())
            .unwrap_or_else(|_| axum::http::HeaderValue::from_static("application/octet-stream")),
    );

    let cache_control = cache_control_for_ui_path(served_path);
    let _ = resp.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static(cache_control),
    );
    let _ = resp.headers_mut().insert(
        axum::http::header::ETAG,
        axum::http::HeaderValue::from_str(etag)
            .unwrap_or_else(|_| axum::http::HeaderValue::from_static("")),
    );
    resp
}

#[cfg(feature = "embed-ui")]
fn embed_etag(bytes: &[u8]) -> String {
    let hash = fnv1a64(bytes);
    format!("W/\"{:x}-{:016x}\"", bytes.len(), hash)
}

#[cfg(not(feature = "embed-ui"))]
fn file_etag(meta: &std::fs::Metadata) -> String {
    use std::time::UNIX_EPOCH;

    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("W/\"{:x}-{:x}\"", meta.len(), modified)
}

#[cfg(feature = "embed-ui")]
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

#[cfg(test)]
mod ws_tests;

#[cfg(test)]
mod error_feedback_tests;
