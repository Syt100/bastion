use std::sync::Arc;

use axum::http::StatusCode;
use tempfile::TempDir;

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::{auth, db};

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
async fn bulk_operations_create_requires_auth_and_csrf() {
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

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    sqlx::query("INSERT INTO agents (id, name, key_hash, created_at) VALUES (?, NULL, ?, ?)")
        .bind("a")
        .bind(vec![0u8; 32])
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert agent");

    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));
    let app = super::router(super::AppState {
        config,
        db: pool.clone(),
        secrets,
        agent_manager: AgentManager::default(),
        run_queue_notify: Arc::new(tokio::sync::Notify::new()),
        incomplete_cleanup_notify: Arc::new(tokio::sync::Notify::new()),
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

    // Unauthenticated -> 401.
    let resp = client
        .post(format!("{}/api/bulk-operations", base_url(addr)))
        .json(&serde_json::json!({
            "kind": "agent_labels_add",
            "selector": { "node_ids": ["a"] },
            "payload": { "labels": ["prod"] }
        }))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Authenticated but missing CSRF -> 401.
    let resp = client
        .post(format!("{}/api/bulk-operations", base_url(addr)))
        .header("cookie", format!("bastion_session={}", session.id))
        .json(&serde_json::json!({
            "kind": "agent_labels_add",
            "selector": { "node_ids": ["a"] },
            "payload": { "labels": ["prod"] }
        }))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    server.abort();
}

#[tokio::test]
async fn bulk_operations_create_and_get_round_trip() {
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

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    for id in ["a", "b"] {
        sqlx::query("INSERT INTO agents (id, name, key_hash, created_at) VALUES (?, NULL, ?, ?)")
            .bind(id)
            .bind(vec![0u8; 32])
            .bind(now)
            .execute(&pool)
            .await
            .expect("insert agent");
    }

    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));
    let app = super::router(super::AppState {
        config,
        db: pool.clone(),
        secrets,
        agent_manager: AgentManager::default(),
        run_queue_notify: Arc::new(tokio::sync::Notify::new()),
        incomplete_cleanup_notify: Arc::new(tokio::sync::Notify::new()),
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
        .post(format!("{}/api/bulk-operations", base_url(addr)))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token)
        .json(&serde_json::json!({
            "kind": "agent_labels_add",
            "selector": { "node_ids": ["a", "b"] },
            "payload": { "labels": ["prod"] }
        }))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");
    let op_id = body["op_id"].as_str().unwrap_or_default().to_string();
    assert!(!op_id.is_empty());

    let resp = client
        .get(format!("{}/api/bulk-operations/{}", base_url(addr), op_id))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::OK);
    let detail: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(detail["id"].as_str().unwrap_or_default(), op_id);
    assert_eq!(detail["total"].as_i64(), Some(2));

    server.abort();
}
