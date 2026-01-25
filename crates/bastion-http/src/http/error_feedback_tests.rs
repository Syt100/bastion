use std::sync::Arc;

use axum::http::StatusCode;
use tempfile::TempDir;

use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::{auth, db};

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;

async fn start_test_server() -> (tokio::net::TcpListener, std::net::SocketAddr) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    (listener, addr)
}

fn test_config(temp: &TempDir) -> Arc<Config> {
    Arc::new(Config {
        bind: "127.0.0.1:0".parse().expect("bind"),
        data_dir: temp.path().to_path_buf(),
        insecure_http: true,
        debug_errors: false,
        hub_timezone: "UTC".to_string(),
        run_retention_days: 180,
        incomplete_cleanup_days: 7,
        trusted_proxies: vec![
            "127.0.0.1/32".parse().expect("proxy"),
            "::1/128".parse().expect("proxy"),
        ],
    })
}

fn base_url(addr: std::net::SocketAddr) -> String {
    format!("http://{addr}")
}

#[tokio::test]
async fn invalid_wecom_webhook_returns_400_with_details_field() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    auth::create_user(&pool, "admin", "pw")
        .await
        .expect("create user");
    let user = auth::find_user_by_username(&pool, "admin")
        .await
        .expect("find user")
        .expect("user exists");
    let session = auth::create_session(&pool, user.id)
        .await
        .expect("create session");

    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));

    let app = super::router(super::AppState {
        config,
        db: pool.clone(),
        secrets,
        agent_manager: AgentManager::default(),
        run_queue_notify: Arc::new(tokio::sync::Notify::new()),
        incomplete_cleanup_notify: Arc::new(tokio::sync::Notify::new()),
        artifact_delete_notify: Arc::new(tokio::sync::Notify::new()),
        jobs_notify: Arc::new(tokio::sync::Notify::new()),
        notifications_notify: Arc::new(tokio::sync::Notify::new()),
        bulk_ops_notify: Arc::new(tokio::sync::Notify::new()),
        run_events_bus: Arc::new(bastion_engine::run_events_bus::RunEventsBus::new()),
        hub_runtime_config: Default::default(),
    });

    let (listener, addr) = start_test_server().await;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve");
    });

    let client = reqwest::Client::new();
    let resp = client
        .put(format!("{}/api/secrets/wecom-bot/test", base_url(addr)))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token)
        .json(&serde_json::json!({ "webhook_url": "not a url" }))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(
        body["error"].as_str().unwrap_or_default(),
        "invalid_webhook_url"
    );
    assert_eq!(
        body["details"]["field"].as_str().unwrap_or_default(),
        "webhook_url"
    );

    server.abort();
}

#[tokio::test]
async fn invalid_smtp_from_returns_400_with_details_field() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    auth::create_user(&pool, "admin", "pw")
        .await
        .expect("create user");
    let user = auth::find_user_by_username(&pool, "admin")
        .await
        .expect("find user")
        .expect("user exists");
    let session = auth::create_session(&pool, user.id)
        .await
        .expect("create session");

    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));

    let app = super::router(super::AppState {
        config,
        db: pool.clone(),
        secrets,
        agent_manager: AgentManager::default(),
        run_queue_notify: Arc::new(tokio::sync::Notify::new()),
        incomplete_cleanup_notify: Arc::new(tokio::sync::Notify::new()),
        artifact_delete_notify: Arc::new(tokio::sync::Notify::new()),
        jobs_notify: Arc::new(tokio::sync::Notify::new()),
        notifications_notify: Arc::new(tokio::sync::Notify::new()),
        bulk_ops_notify: Arc::new(tokio::sync::Notify::new()),
        run_events_bus: Arc::new(bastion_engine::run_events_bus::RunEventsBus::new()),
        hub_runtime_config: Default::default(),
    });

    let (listener, addr) = start_test_server().await;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve");
    });

    let client = reqwest::Client::new();
    let resp = client
        .put(format!("{}/api/secrets/smtp/test", base_url(addr)))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token)
        .json(&serde_json::json!({
          "host": "smtp.example.com",
          "port": 587,
          "username": "",
          "password": "",
          "from": "not-an-email",
          "to": ["a@example.com"],
          "tls": "starttls"
        }))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["error"].as_str().unwrap_or_default(), "invalid_from");
    assert_eq!(
        body["details"]["field"].as_str().unwrap_or_default(),
        "from"
    );

    server.abort();
}

#[tokio::test]
async fn login_rate_limit_includes_retry_after_details() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let locked_until = now + 120;
    sqlx::query(
        "INSERT INTO login_throttle (ip, failures, first_failed_at, last_failed_at, locked_until) VALUES (?, ?, ?, ?, ?)",
    )
    .bind("127.0.0.1")
    .bind(10_i64)
    .bind(now)
    .bind(now)
    .bind(locked_until)
    .execute(&pool)
    .await
    .expect("insert throttle");

    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));

    let app = super::router(super::AppState {
        config,
        db: pool.clone(),
        secrets,
        agent_manager: AgentManager::default(),
        run_queue_notify: Arc::new(tokio::sync::Notify::new()),
        incomplete_cleanup_notify: Arc::new(tokio::sync::Notify::new()),
        artifact_delete_notify: Arc::new(tokio::sync::Notify::new()),
        jobs_notify: Arc::new(tokio::sync::Notify::new()),
        notifications_notify: Arc::new(tokio::sync::Notify::new()),
        bulk_ops_notify: Arc::new(tokio::sync::Notify::new()),
        run_events_bus: Arc::new(bastion_engine::run_events_bus::RunEventsBus::new()),
        hub_runtime_config: Default::default(),
    });

    let (listener, addr) = start_test_server().await;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve");
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/auth/login", base_url(addr)))
        .json(&serde_json::json!({ "username": "admin", "password": "bad" }))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["error"].as_str().unwrap_or_default(), "rate_limited");
    assert!(
        body["details"]["retry_after_seconds"]
            .as_i64()
            .unwrap_or_default()
            > 0
    );

    server.abort();
}

#[tokio::test]
async fn agent_enroll_invalid_token_format_returns_401_with_details() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));

    let app = super::router(super::AppState {
        config,
        db: pool.clone(),
        secrets,
        agent_manager: AgentManager::default(),
        run_queue_notify: Arc::new(tokio::sync::Notify::new()),
        incomplete_cleanup_notify: Arc::new(tokio::sync::Notify::new()),
        artifact_delete_notify: Arc::new(tokio::sync::Notify::new()),
        jobs_notify: Arc::new(tokio::sync::Notify::new()),
        notifications_notify: Arc::new(tokio::sync::Notify::new()),
        bulk_ops_notify: Arc::new(tokio::sync::Notify::new()),
        run_events_bus: Arc::new(bastion_engine::run_events_bus::RunEventsBus::new()),
        hub_runtime_config: Default::default(),
    });

    let (listener, addr) = start_test_server().await;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve");
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/agent/enroll", base_url(addr)))
        .json(&serde_json::json!({ "token": "%%% not base64 %%%", "name": null }))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["error"].as_str().unwrap_or_default(), "invalid_token");
    assert_eq!(
        body["details"]["field"].as_str().unwrap_or_default(),
        "token"
    );

    server.abort();
}

#[tokio::test]
async fn create_job_missing_webdav_secret_returns_400_with_details_field() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    auth::create_user(&pool, "admin", "pw")
        .await
        .expect("create user");
    let user = auth::find_user_by_username(&pool, "admin")
        .await
        .expect("find user")
        .expect("user exists");
    let session = auth::create_session(&pool, user.id)
        .await
        .expect("create session");

    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));

    let app = super::router(super::AppState {
        config,
        db: pool.clone(),
        secrets,
        agent_manager: AgentManager::default(),
        run_queue_notify: Arc::new(tokio::sync::Notify::new()),
        incomplete_cleanup_notify: Arc::new(tokio::sync::Notify::new()),
        artifact_delete_notify: Arc::new(tokio::sync::Notify::new()),
        jobs_notify: Arc::new(tokio::sync::Notify::new()),
        notifications_notify: Arc::new(tokio::sync::Notify::new()),
        bulk_ops_notify: Arc::new(tokio::sync::Notify::new()),
        run_events_bus: Arc::new(bastion_engine::run_events_bus::RunEventsBus::new()),
        hub_runtime_config: Default::default(),
    });

    let (listener, addr) = start_test_server().await;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve");
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/jobs", base_url(addr)))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token)
        .json(&serde_json::json!({
          "name": "job1",
          "agent_id": null,
          "schedule": null,
          "overlap_policy": "queue",
          "spec": {
            "v": 1,
            "type": "filesystem",
            "source": { "root": "/data" },
            "target": {
              "type": "webdav",
              "base_url": "http://example.com/backup",
              "secret_name": "missing",
              "part_size_bytes": 268435456
            }
          }
        }))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(
        body["error"].as_str().unwrap_or_default(),
        "invalid_webdav_secret"
    );
    assert_eq!(
        body["details"]["field"].as_str().unwrap_or_default(),
        "spec.target.secret_name"
    );

    server.abort();
}
