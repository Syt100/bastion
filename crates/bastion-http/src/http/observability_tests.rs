use std::sync::Arc;

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use tempfile::TempDir;
use tower::ServiceExt as _;

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;
use bastion_engine::run_events_bus::RunEventsBus;
use bastion_storage::db;
use bastion_storage::secrets::SecretsCrypto;

#[tokio::test]
async fn ready_is_ok_and_allowed_insecure() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let config = Arc::new(Config {
        bind: "127.0.0.1:0".parse().expect("bind"),
        data_dir: temp.path().to_path_buf(),
        insecure_http: false,
        debug_errors: false,
        hub_timezone: "UTC".to_string(),
        run_retention_days: 180,
        incomplete_cleanup_days: 7,
        trusted_proxies: Vec::new(),
    });
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));

    let app = super::router(super::AppState {
        config,
        db: pool,
        secrets,
        agent_manager: AgentManager::default(),
        run_queue_notify: Arc::new(tokio::sync::Notify::new()),
        incomplete_cleanup_notify: Arc::new(tokio::sync::Notify::new()),
        artifact_delete_notify: Arc::new(tokio::sync::Notify::new()),
        jobs_notify: Arc::new(tokio::sync::Notify::new()),
        notifications_notify: Arc::new(tokio::sync::Notify::new()),
        bulk_ops_notify: Arc::new(tokio::sync::Notify::new()),
        run_events_bus: Arc::new(RunEventsBus::new()),
        hub_runtime_config: Default::default(),
    });

    // Use a non-loopback peer to ensure `/api/ready` bypasses HTTPS enforcement.
    let peer: std::net::SocketAddr = "1.2.3.4:5555".parse().expect("peer");
    let mut req = Request::builder()
        .method("GET")
        .uri("/api/ready")
        .body(Body::empty())
        .expect("request");
    req.extensions_mut().insert(ConnectInfo(peer));

    let resp = app.oneshot(req).await.expect("response");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .expect("body bytes");
    let value: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(value.get("ok").and_then(|v| v.as_bool()), Some(true));
}
