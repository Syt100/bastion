use std::sync::Arc;

use axum::http::StatusCode;
use tempfile::TempDir;

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::{auth, db, jobs_repo, run_artifacts_repo, runs_repo};

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
async fn list_job_snapshots_requires_auth() {
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
        .get(format!("{}/api/jobs/{}/snapshots", base_url(addr), "job1"))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    server.abort();
}

#[tokio::test]
async fn list_job_snapshots_returns_items() {
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

    let job = jobs_repo::create_job(
        &pool,
        "job",
        None,
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({
            "v": 1,
            "type": "filesystem",
            "source": { "root": "/" },
            "target": { "type": "local_dir", "base_dir": "/tmp" }
        }),
    )
    .await
    .expect("create job");

    let run = runs_repo::create_run(&pool, &job.id, runs_repo::RunStatus::Queued, 1, None, None, None)
        .await
        .expect("create run");

    runs_repo::set_run_target_snapshot(
        &pool,
        &run.id,
        serde_json::json!({
            "node_id": "hub",
            "target": { "type": "local_dir", "base_dir": "/tmp" }
        }),
    )
    .await
    .expect("set snapshot");

    runs_repo::complete_run(
        &pool,
        &run.id,
        runs_repo::RunStatus::Success,
        Some(serde_json::json!({ "artifact_format": "archive_v1" })),
        None,
    )
    .await
    .expect("complete run");

    run_artifacts_repo::upsert_run_artifact_from_successful_run(&pool, &run.id)
        .await
        .expect("index");

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
        .get(format!(
            "{}/api/jobs/{}/snapshots",
            base_url(addr),
            job.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["items"].as_array().map(|a| a.len()).unwrap_or(0), 1);
    assert_eq!(
        body["items"][0]["run_id"].as_str().unwrap_or_default(),
        run.id
    );

    server.abort();
}

