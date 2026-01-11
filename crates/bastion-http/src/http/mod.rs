use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
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
mod fs;
mod jobs;
mod maintenance;
mod middleware;
mod notifications;
mod operations;
mod runs;
mod secrets;
mod shared;
mod ui;

use error::AppError;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: SqlitePool,
    pub secrets: Arc<SecretsCrypto>,
    pub agent_manager: AgentManager,
    pub run_queue_notify: Arc<Notify>,
    pub incomplete_cleanup_notify: Arc<Notify>,
    pub jobs_notify: Arc<Notify>,
    pub notifications_notify: Arc<Notify>,
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
    error::set_debug_errors(state.config.debug_errors);

    const API_BODY_LIMIT_BYTES: usize = 2 * 1024 * 1024;
    const AGENT_BODY_LIMIT_BYTES: usize = 4 * 1024 * 1024;

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

    let api_router = Router::new()
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
            "/api/nodes/{node_id}/secrets/webdav",
            get(secrets::list_webdav_secrets_node),
        )
        .route(
            "/api/nodes/{node_id}/secrets/webdav/{name}",
            get(secrets::get_webdav_secret_node)
                .put(secrets::upsert_webdav_secret_node)
                .delete(secrets::delete_webdav_secret_node),
        )
        .route("/api/nodes/{node_id}/fs/list", get(fs::fs_list))
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
        .route("/api/jobs/{id}/archive", post(jobs::archive_job))
        .route("/api/jobs/{id}/unarchive", post(jobs::unarchive_job))
        .route("/api/jobs/{id}/run", post(jobs::trigger_job_run))
        .route("/api/jobs/{id}/runs", get(jobs::list_job_runs))
        .route("/api/runs/{id}/events", get(jobs::list_run_events))
        .route("/api/runs/{id}/events/ws", get(jobs::run_events_ws))
        .route("/api/runs/{id}/entries", get(runs::list_run_entries))
        .route("/api/runs/{id}/restore", post(operations::start_restore))
        .route("/api/runs/{id}/verify", post(operations::start_verify))
        .route(
            "/api/maintenance/incomplete-cleanup",
            get(maintenance::list_incomplete_cleanup_tasks),
        )
        .route(
            "/api/maintenance/incomplete-cleanup/{run_id}",
            get(maintenance::get_incomplete_cleanup_task),
        )
        .route(
            "/api/maintenance/incomplete-cleanup/{run_id}/retry-now",
            post(maintenance::retry_incomplete_cleanup_task_now),
        )
        .route(
            "/api/maintenance/incomplete-cleanup/{run_id}/ignore",
            post(maintenance::ignore_incomplete_cleanup_task),
        )
        .route(
            "/api/maintenance/incomplete-cleanup/{run_id}/unignore",
            post(maintenance::unignore_incomplete_cleanup_task),
        )
        .route(
            "/api/notifications/settings",
            get(notifications::get_settings).put(notifications::put_settings),
        )
        .route(
            "/api/notifications/destinations",
            get(notifications::list_destinations),
        )
        .route(
            "/api/notifications/destinations/{channel}/{name}/enabled",
            post(notifications::set_destination_enabled),
        )
        .route(
            "/api/notifications/destinations/{channel}/{name}/test",
            post(notifications::test_destination),
        )
        .route("/api/notifications/queue", get(notifications::list_queue))
        .route(
            "/api/notifications/queue/{id}/retry-now",
            post(notifications::retry_now),
        )
        .route(
            "/api/notifications/queue/{id}/cancel",
            post(notifications::cancel),
        )
        .route("/api/operations/{id}", get(operations::get_operation))
        .route(
            "/api/operations/{id}/events",
            get(operations::list_operation_events),
        )
        .layer(DefaultBodyLimit::max(API_BODY_LIMIT_BYTES));

    let agent_router = Router::new()
        .route("/agent/enroll", post(agents::agent_enroll))
        .route("/agent/runs/ingest", post(agents::agent_ingest_runs))
        .route("/agent/ws", get(agents::agent_ws))
        .layer(DefaultBodyLimit::max(AGENT_BODY_LIMIT_BYTES));

    api_router
        .merge(agent_router)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::require_secure_middleware,
        ))
        .layer(CookieManagerLayer::new())
        .layer(trace_layer)
        .layer(PropagateRequestIdLayer::new(request_id_header.clone()))
        .layer(SetRequestIdLayer::new(request_id_header, MakeRequestUuid))
        .with_state(state)
        .fallback(ui::ui_fallback)
}

#[cfg(test)]
mod ws_tests;

#[cfg(test)]
mod error_feedback_tests;

#[cfg(test)]
mod agents_ingest_tests;

#[cfg(test)]
mod filter_multiselect_tests;
