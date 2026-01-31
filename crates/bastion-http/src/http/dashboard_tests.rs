use std::sync::Arc;

use axum::http::StatusCode;
use tempfile::TempDir;

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;
use bastion_engine::run_events_bus::RunEventsBus;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::{auth, db, jobs_repo, runs_repo};

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
async fn dashboard_overview_requires_auth() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let config = test_config(&temp);
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
        .get(format!("{}/api/dashboard/overview", base_url(addr)))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    server.abort();
}

#[tokio::test]
async fn dashboard_overview_returns_stats_trend_and_recent_runs() {
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

    // Agents: 1 online, 1 offline, 1 revoked.
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let key = bastion_core::agent::generate_token_b64_urlsafe(32);
    let key_hash = bastion_core::agent::sha256_urlsafe_token(&key).expect("hash");
    sqlx::query(
        "INSERT INTO agents (id, name, key_hash, created_at, last_seen_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind("agent1")
    .bind("Agent One")
    .bind(key_hash.clone())
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .expect("insert agent1");
    sqlx::query(
        "INSERT INTO agents (id, name, key_hash, created_at, last_seen_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind("agent2")
    .bind("Agent Two")
    .bind(key_hash.clone())
    .bind(now)
    .bind(now - 120)
    .execute(&pool)
    .await
    .expect("insert agent2");
    sqlx::query("INSERT INTO agents (id, name, key_hash, created_at, revoked_at, last_seen_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind("agent3")
        .bind("Revoked")
        .bind(key_hash)
        .bind(now)
        .bind(now - 1)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert agent3");

    // Jobs: 1 hub active, 1 agent active, 1 archived.
    let job_hub = jobs_repo::create_job(
        &pool,
        "Hub Job",
        None,
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({"v":1,"type":"filesystem"}),
    )
    .await
    .expect("create job hub");
    let job_agent = jobs_repo::create_job(
        &pool,
        "Agent Job",
        Some("agent1"),
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({"v":1,"type":"filesystem"}),
    )
    .await
    .expect("create job agent");
    let job_archived = jobs_repo::create_job(
        &pool,
        "Archived Job",
        None,
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({"v":1,"type":"filesystem"}),
    )
    .await
    .expect("create job archived");
    jobs_repo::archive_job(&pool, &job_archived.id)
        .await
        .expect("archive job");

    // Runs: 1 success + 1 failed within 24h and within 7d, plus a running run.
    let _success = runs_repo::create_run(
        &pool,
        &job_hub.id,
        runs_repo::RunStatus::Success,
        now - 4000,
        Some(now - 3600),
        None,
        None,
    )
    .await
    .expect("create success run");
    let failed = runs_repo::create_run(
        &pool,
        &job_agent.id,
        runs_repo::RunStatus::Failed,
        now - 3000,
        Some(now - 3500),
        Some(serde_json::json!({"executed_offline": true})),
        Some("boom"),
    )
    .await
    .expect("create failed run");
    let _running = runs_repo::create_run(
        &pool,
        &job_hub.id,
        runs_repo::RunStatus::Running,
        now - 10,
        None,
        None,
        None,
    )
    .await
    .expect("create running run");

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
        run_events_bus: Arc::new(RunEventsBus::new()),
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
        .get(format!("{}/api/dashboard/overview", base_url(addr)))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");

    // Stats shape.
    assert!(body.get("stats").is_some());
    assert_eq!(
        body["stats"]["agents"]["online"]
            .as_i64()
            .unwrap_or_default(),
        1
    );
    assert_eq!(
        body["stats"]["jobs"]["archived"]
            .as_i64()
            .unwrap_or_default(),
        1
    );
    assert_eq!(
        body["stats"]["runs"]["failed_24h"]
            .as_i64()
            .unwrap_or_default(),
        1
    );

    // Trend shape.
    let trend = body["trend_7d"].as_array().expect("trend array");
    assert_eq!(trend.len(), 7);

    // Recent runs include our latest failed run and preserve executed_offline.
    let recent = body["recent_runs"].as_array().expect("recent runs");
    assert!(!recent.is_empty());
    let found = recent
        .iter()
        .find(|r| r["run_id"] == failed.id)
        .expect("failed run present");
    assert_eq!(found["executed_offline"].as_bool(), Some(true));
    assert_eq!(found["node_id"].as_str(), Some("agent1"));
    assert_eq!(found["job_name"].as_str(), Some("Agent Job"));

    server.abort();
}
