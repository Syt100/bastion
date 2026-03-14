use std::sync::Arc;

use axum::http::StatusCode;
use tempfile::TempDir;

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;
use bastion_engine::run_events_bus::RunEventsBus;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::{auth, db, jobs_repo, operations_repo, runs_repo};

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

fn build_app(
    pool: sqlx::SqlitePool,
    config: Arc<Config>,
    secrets: Arc<SecretsCrypto>,
) -> axum::Router {
    super::router(super::AppState {
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
    })
}

async fn create_authed_session(
    pool: &sqlx::SqlitePool,
) -> (reqwest::Client, String) {
    let user_password = uuid::Uuid::new_v4().to_string();
    auth::create_user(pool, "admin", &user_password)
        .await
        .expect("create user");
    let user = auth::find_user_by_username(pool, "admin")
        .await
        .expect("find user")
        .expect("user exists");
    let session = auth::create_session(pool, user.id)
        .await
        .expect("create session");
    (reqwest::Client::new(), session.id)
}

async fn insert_agent(
    pool: &sqlx::SqlitePool,
    agent_id: &str,
    name: &str,
    last_seen_at: Option<i64>,
    revoked_at: Option<i64>,
) {
    let key = bastion_core::agent::generate_token_b64_urlsafe(32);
    let key_hash = bastion_core::agent::sha256_urlsafe_token(&key).expect("hash");
    sqlx::query(
        "INSERT INTO agents (id, name, key_hash, created_at, revoked_at, last_seen_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(agent_id)
    .bind(name)
    .bind(key_hash)
    .bind(time::OffsetDateTime::now_utc().unix_timestamp())
    .bind(revoked_at)
    .bind(last_seen_at)
    .execute(pool)
    .await
    .expect("insert agent");
}

#[tokio::test]
async fn command_center_requires_auth() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));
    let app = build_app(pool, config, secrets);

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
        .get(format!("{}/api/command-center", base_url(addr)))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    server.abort();
}

#[tokio::test]
async fn command_center_returns_scoped_sections_and_ready_readiness() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let (client, session_id) = create_authed_session(&pool).await;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    insert_agent(&pool, "agent1", "Edge One", Some(now), None).await;
    insert_agent(&pool, "agent2", "Edge Two", Some(now - 180), None).await;

    let hub_job = jobs_repo::create_job(
        &pool,
        "Hub Job",
        None,
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({"v":1,"type":"filesystem"}),
    )
    .await
    .expect("create hub job");
    let agent_job = jobs_repo::create_job(
        &pool,
        "Agent Job",
        Some("agent1"),
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({"v":1,"type":"filesystem"}),
    )
    .await
    .expect("create agent job");

    let _hub_success = runs_repo::create_run(
        &pool,
        &hub_job.id,
        runs_repo::RunStatus::Success,
        now - 7200,
        Some(now - 7100),
        None,
        None,
    )
    .await
    .expect("hub success");

    let agent_success = runs_repo::create_run(
        &pool,
        &agent_job.id,
        runs_repo::RunStatus::Success,
        now - 5000,
        Some(now - 4900),
        None,
        None,
    )
    .await
    .expect("agent success");
    let agent_failed = runs_repo::create_run(
        &pool,
        &agent_job.id,
        runs_repo::RunStatus::Failed,
        now - 1800,
        Some(now - 1700),
        None,
        Some("remote upload failed"),
    )
    .await
    .expect("agent failed");

    let verify_op = operations_repo::create_operation(
        &pool,
        bastion_storage::operations_repo::OperationKind::Verify,
        Some(("run", &agent_success.id)),
    )
    .await
    .expect("create verify");
    operations_repo::complete_operation(
        &pool,
        &verify_op.id,
        bastion_storage::operations_repo::OperationStatus::Success,
        None,
        None,
    )
    .await
    .expect("complete verify");

    sqlx::query(
        "INSERT INTO notifications (id, run_id, channel, secret_name, status, attempts, next_attempt_at, created_at, updated_at, last_error)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("notification-1")
    .bind(&agent_failed.id)
    .bind("smtp")
    .bind("ops-mail")
    .bind("failed")
    .bind(2_i64)
    .bind(now - 100)
    .bind(now - 120)
    .bind(now - 90)
    .bind("smtp timeout")
    .execute(&pool)
    .await
    .expect("insert failed notification");

    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));
    let app = build_app(pool.clone(), config, secrets);

    let (listener, addr) = start_test_server().await;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve");
    });

    let resp = client
        .get(format!(
            "{}/api/command-center?scope=agent:agent1&range=24h",
            base_url(addr)
        ))
        .header("cookie", format!("bastion_session={session_id}"))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");

    assert_eq!(body["scope"]["requested"].as_str(), Some("agent:agent1"));
    assert_eq!(body["scope"]["effective"].as_str(), Some("agent:agent1"));
    assert_eq!(body["attention"]["state"].as_str(), Some("ready"));
    assert!(body["attention"]["items"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .any(|item| item["kind"].as_str() == Some("run_failed")));
    assert!(body["critical_activity"]["items"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .any(|item| item["kind"].as_str() == Some("operation_verify_success")));
    assert_eq!(
        body["recovery_readiness"]["overall"].as_str(),
        Some("healthy")
    );
    assert_eq!(
        body["recovery_readiness"]["backup"]["recent_job_id"].as_str(),
        Some(agent_job.id.as_str())
    );
    assert_eq!(
        body["recovery_readiness"]["verify"]["recent_run_id"].as_str(),
        Some(agent_success.id.as_str())
    );

    server.abort();
}

#[tokio::test]
async fn command_center_returns_empty_state_for_scope_without_data() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let (client, session_id) = create_authed_session(&pool).await;

    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));
    let app = build_app(pool, config, secrets);

    let (listener, addr) = start_test_server().await;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve");
    });

    let resp = client
        .get(format!(
            "{}/api/command-center?scope=agent:missing",
            base_url(addr)
        ))
        .header("cookie", format!("bastion_session={session_id}"))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");

    assert_eq!(body["attention"]["state"].as_str(), Some("empty"));
    assert_eq!(body["critical_activity"]["state"].as_str(), Some("empty"));
    assert_eq!(body["watchlist"]["state"].as_str(), Some("empty"));
    assert_eq!(body["recovery_readiness"]["overall"].as_str(), Some("empty"));

    server.abort();
}

#[tokio::test]
async fn command_center_readiness_is_degraded_when_verify_is_missing() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let (client, session_id) = create_authed_session(&pool).await;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let hub_job = jobs_repo::create_job(
        &pool,
        "Hub Job",
        None,
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({"v":1,"type":"filesystem"}),
    )
    .await
    .expect("create hub job");
    let _hub_success = runs_repo::create_run(
        &pool,
        &hub_job.id,
        runs_repo::RunStatus::Success,
        now - 3600,
        Some(now - 3500),
        None,
        None,
    )
    .await
    .expect("hub success");

    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));
    let app = build_app(pool, config, secrets);

    let (listener, addr) = start_test_server().await;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve");
    });

    let resp = client
        .get(format!("{}/api/command-center?scope=hub", base_url(addr)))
        .header("cookie", format!("bastion_session={session_id}"))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");

    assert_eq!(body["recovery_readiness"]["state"].as_str(), Some("degraded"));
    assert_eq!(
        body["recovery_readiness"]["overall"].as_str(),
        Some("degraded")
    );
    assert!(body["recovery_readiness"]["verify"]["recent_success_at"].is_null());
    assert!(body["recovery_readiness"]["blockers"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .any(|blocker| blocker["kind"].as_str() == Some("missing_verification")));

    server.abort();
}
