use std::sync::Arc;

use axum::http::StatusCode;
use tempfile::TempDir;

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::{db, jobs_repo, runs_repo};

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

async fn insert_agent(pool: &sqlx::SqlitePool, agent_id: &str) -> (String, String) {
    let agent_key = bastion_core::agent::generate_token_b64_urlsafe(32);
    let hash = bastion_core::agent::sha256_urlsafe_token(&agent_key).expect("hash");
    sqlx::query("INSERT INTO agents (id, name, key_hash, created_at) VALUES (?, NULL, ?, ?)")
        .bind(agent_id)
        .bind(hash)
        .bind(1_i64)
        .execute(pool)
        .await
        .expect("insert agent");
    (agent_id.to_string(), agent_key)
}

#[tokio::test]
async fn agent_ingest_runs_inserts_run_and_events_and_dedupes() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let (agent_id, agent_key) = insert_agent(&pool, "agent1").await;

    let job = jobs_repo::create_job(
        &pool,
        "job1",
        Some(&agent_id),
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({
            "v": 1,
            "type": "filesystem",
            "source": { "root": "/data" },
            "target": { "type": "local_dir", "base_dir": "/tmp", "part_size_bytes": 1024 }
        }),
    )
    .await
    .expect("create job");

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

    let run_id = "run1";
    let req = serde_json::json!({
        "run": {
            "id": run_id,
            "job_id": job.id,
            "status": "success",
            "started_at": 100,
            "ended_at": 120,
            "summary": { "executed_offline": true },
            "events": [
                { "seq": 1, "ts": 101, "level": "info", "kind": "k1", "message": "m1" },
                { "seq": 2, "ts": 102, "level": "info", "kind": "k2", "message": "m2", "fields": {"a": 1} }
            ]
        }
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/agent/runs/ingest", base_url(addr)))
        .header("authorization", format!("Bearer {agent_key}"))
        .json(&req)
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let run = runs_repo::get_run(&pool, run_id)
        .await
        .expect("get run")
        .expect("run exists");
    assert_eq!(run.job_id, job.id);
    assert_eq!(run.status, runs_repo::RunStatus::Success);
    assert_eq!(
        run.summary
            .as_ref()
            .and_then(|v| v.get("executed_offline"))
            .and_then(|v| v.as_bool()),
        Some(true)
    );

    let events = runs_repo::list_run_events(&pool, run_id, 10)
        .await
        .expect("events");
    assert_eq!(events.len(), 2);

    // Re-ingest same payload: should be idempotent and not duplicate events.
    let resp = client
        .post(format!("{}/agent/runs/ingest", base_url(addr)))
        .header("authorization", format!("Bearer {agent_key}"))
        .json(&req)
        .send()
        .await
        .expect("request2");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let events = runs_repo::list_run_events(&pool, run_id, 10)
        .await
        .expect("events2");
    assert_eq!(events.len(), 2);

    server.abort();
}

#[tokio::test]
async fn agent_ingest_runs_requires_auth() {
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
        .post(format!("{}/agent/runs/ingest", base_url(addr)))
        .json(&serde_json::json!({"run":{"id":"r","job_id":"j","status":"success","started_at":1,"ended_at":2,"events":[]}}))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["error"].as_str().unwrap_or_default(), "unauthorized");

    server.abort();
}

#[tokio::test]
async fn agent_ingest_runs_rejects_jobs_not_assigned_to_agent() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let (agent_id, agent_key) = insert_agent(&pool, "agent1").await;

    let job = jobs_repo::create_job(
        &pool,
        "job1",
        Some("other-agent"),
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({
            "v": 1,
            "type": "filesystem",
            "source": { "root": "/data" },
            "target": { "type": "local_dir", "base_dir": "/tmp" }
        }),
    )
    .await
    .expect("create job");

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
        .post(format!("{}/agent/runs/ingest", base_url(addr)))
        .header("authorization", format!("Bearer {agent_key}"))
        .json(&serde_json::json!({
            "run": {
                "id": "r1",
                "job_id": job.id,
                "status": "success",
                "started_at": 1,
                "ended_at": 2,
                "events": []
            }
        }))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["error"].as_str().unwrap_or_default(), "invalid_job_id");
    assert_eq!(agent_id, "agent1");

    server.abort();
}

#[tokio::test]
async fn agent_ingest_runs_limits_event_count() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let (agent_id, agent_key) = insert_agent(&pool, "agent1").await;

    let job = jobs_repo::create_job(
        &pool,
        "job1",
        Some(&agent_id),
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({
            "v": 1,
            "type": "filesystem",
            "source": { "root": "/data" },
            "target": { "type": "local_dir", "base_dir": "/tmp" }
        }),
    )
    .await
    .expect("create job");

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

    let mut events = Vec::new();
    for _ in 0..2001 {
        events.push(
            serde_json::json!({"seq": 1, "ts": 1, "level": "info", "kind": "k", "message": "m"}),
        );
    }

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/agent/runs/ingest", base_url(addr)))
        .header("authorization", format!("Bearer {agent_key}"))
        .json(&serde_json::json!({
            "run": {
                "id": "r1",
                "job_id": job.id,
                "status": "success",
                "started_at": 1,
                "ended_at": 2,
                "events": events
            }
        }))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(
        body["error"].as_str().unwrap_or_default(),
        "too_many_events"
    );

    server.abort();
}

#[tokio::test]
async fn agent_ingest_runs_upserts_existing_run_metadata() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let (agent_id, agent_key) = insert_agent(&pool, "agent1").await;

    let job = jobs_repo::create_job(
        &pool,
        "job1",
        Some(&agent_id),
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({
            "v": 1,
            "type": "filesystem",
            "source": { "root": "/data" },
            "target": { "type": "local_dir", "base_dir": "/tmp" }
        }),
    )
    .await
    .expect("create job");

    let run_id = "run1";
    sqlx::query(
        "INSERT INTO runs (id, job_id, status, started_at, ended_at, summary_json, error) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(&job.id)
    .bind("failed")
    .bind(100_i64)
    .bind(120_i64)
    .bind(Option::<String>::None)
    .bind(Some("old_error"))
    .execute(&pool)
    .await
    .expect("insert run");

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
        .post(format!("{}/agent/runs/ingest", base_url(addr)))
        .header("authorization", format!("Bearer {agent_key}"))
        .json(&serde_json::json!({
            "run": {
                "id": run_id,
                "job_id": job.id,
                "status": "success",
                "started_at": 100,
                "ended_at": 120,
                "summary": { "executed_offline": true },
                "events": []
            }
        }))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let run = runs_repo::get_run(&pool, run_id)
        .await
        .expect("get run")
        .expect("run exists");
    assert_eq!(run.status, runs_repo::RunStatus::Success);
    assert_eq!(run.error, None);

    server.abort();
}

#[tokio::test]
async fn agent_ingest_runs_validates_ended_at() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let (agent_id, agent_key) = insert_agent(&pool, "agent1").await;

    let job = jobs_repo::create_job(
        &pool,
        "job1",
        Some(&agent_id),
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({
            "v": 1,
            "type": "filesystem",
            "source": { "root": "/data" },
            "target": { "type": "local_dir", "base_dir": "/tmp" }
        }),
    )
    .await
    .expect("create job");

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
        .post(format!("{}/agent/runs/ingest", base_url(addr)))
        .header("authorization", format!("Bearer {agent_key}"))
        .json(&serde_json::json!({
            "run": {
                "id": "r1",
                "job_id": job.id,
                "status": "success",
                "started_at": 10,
                "ended_at": 9,
                "events": []
            }
        }))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(
        body["error"].as_str().unwrap_or_default(),
        "invalid_ended_at"
    );

    server.abort();
}

#[tokio::test]
async fn agent_ingest_runs_enforces_body_size_limit() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let (agent_id, agent_key) = insert_agent(&pool, "agent1").await;

    let job = jobs_repo::create_job(
        &pool,
        "job1",
        Some(&agent_id),
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({
            "v": 1,
            "type": "filesystem",
            "source": { "root": "/data" },
            "target": { "type": "local_dir", "base_dir": "/tmp" }
        }),
    )
    .await
    .expect("create job");

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

    // Intentionally exceed the configured Agent body limit with a large message payload.
    let big_message = "x".repeat(5 * 1024 * 1024);
    let payload = serde_json::to_vec(&serde_json::json!({
        "run": {
            "id": "r1",
            "job_id": job.id,
            "status": "success",
            "started_at": 1,
            "ended_at": 2,
            "events": [
                { "seq": 1, "ts": 1, "level": "info", "kind": "k", "message": big_message }
            ]
        }
    }))
    .expect("payload");

    // Use `oneshot` instead of a real TCP client to avoid platform-specific behavior where
    // the server may reset the connection while the client is still streaming a too-large body.
    let peer: std::net::SocketAddr = "127.0.0.1:1234".parse().expect("peer");
    let mut req = axum::http::Request::builder()
        .method("POST")
        .uri("/agent/runs/ingest")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {agent_key}"))
        .body(axum::body::Body::from(payload))
        .expect("request");
    req.extensions_mut()
        .insert(axum::extract::ConnectInfo(peer));

    let resp = tower::ServiceExt::oneshot(app, req)
        .await
        .expect("response");

    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
