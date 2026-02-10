use std::sync::Arc;

use axum::http::StatusCode;
use tempfile::TempDir;

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::{auth, db};

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

async fn start_server(
    temp: &TempDir,
    pool: sqlx::SqlitePool,
) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let config = test_config(temp);
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
        run_events_bus: Arc::new(bastion_engine::run_events_bus::RunEventsBus::new()),
        hub_runtime_config: Default::default(),
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");

    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve");
    });

    (addr, server)
}

fn base_url(addr: std::net::SocketAddr) -> String {
    format!("http://{addr}")
}

#[tokio::test]
async fn setup_initialize_rejects_short_password() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let (addr, server) = start_server(&temp, pool.clone()).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/setup/initialize", base_url(addr)))
        .json(&serde_json::json!({ "username": "admin", "password": "short" }))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(
        body["error"].as_str().unwrap_or_default(),
        "invalid_password"
    );

    let count = auth::users_count(&pool).await.expect("users_count");
    assert_eq!(count, 0);

    server.abort();
}

#[tokio::test]
async fn login_rejects_blank_username() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let user_password = uuid::Uuid::new_v4().to_string();
    auth::create_user(&pool, "admin", &user_password)
        .await
        .expect("create user");

    let (addr, server) = start_server(&temp, pool).await;
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/auth/login", base_url(addr)))
        .json(&serde_json::json!({ "username": "   ", "password": "password-123" }))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(
        body["error"].as_str().unwrap_or_default(),
        "invalid_username"
    );

    server.abort();
}

#[tokio::test]
async fn setup_initialize_is_atomic_under_concurrency() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let (addr, server) = start_server(&temp, pool.clone()).await;

    let client = reqwest::Client::new();
    let url = format!("{}/api/setup/initialize", base_url(addr));

    let req1 = client
        .post(&url)
        .json(&serde_json::json!({
            "username": "admin1",
            "password": "test passphrase 111!"
        }))
        .send();
    let req2 = client
        .post(&url)
        .json(&serde_json::json!({
            "username": "admin2",
            "password": "test passphrase 222!"
        }))
        .send();

    let (resp1, resp2) = tokio::join!(req1, req2);
    let status1 = resp1.expect("resp1").status();
    let status2 = resp2.expect("resp2").status();

    let statuses = [status1, status2];
    assert!(statuses.contains(&StatusCode::NO_CONTENT));
    assert!(statuses.contains(&StatusCode::CONFLICT));

    let count = auth::users_count(&pool).await.expect("users_count");
    assert_eq!(count, 1);

    server.abort();
}
