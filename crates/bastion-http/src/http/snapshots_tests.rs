use std::sync::Arc;

use axum::http::StatusCode;
use tempfile::TempDir;

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::{artifact_delete_repo, auth, db, jobs_repo, run_artifacts_repo, runs_repo};

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
        .get(format!("{}/api/jobs/{}/snapshots", base_url(addr), "job1"))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    server.abort();
}

#[tokio::test]
async fn delete_job_snapshot_requires_auth() {
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
        .post(format!(
            "{}/api/jobs/{}/snapshots/{}/delete",
            base_url(addr),
            "job1",
            "run1"
        ))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    server.abort();
}

#[tokio::test]
async fn delete_job_snapshot_requires_csrf() {
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
        .post(format!(
            "{}/api/jobs/{}/snapshots/{}/delete",
            base_url(addr),
            "job1",
            "run1"
        ))
        .header("cookie", format!("bastion_session={}", session.id))
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

    let run = runs_repo::create_run(
        &pool,
        &job.id,
        runs_repo::RunStatus::Queued,
        1,
        None,
        None,
        None,
    )
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
        .get(format!("{}/api/jobs/{}/snapshots", base_url(addr), job.id))
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

#[tokio::test]
async fn delete_job_snapshot_enqueues_task_and_event() {
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

    let run = runs_repo::create_run(
        &pool,
        &job.id,
        runs_repo::RunStatus::Queued,
        1,
        None,
        None,
        None,
    )
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
        .post(format!(
            "{}/api/jobs/{}/snapshots/{}/delete",
            base_url(addr),
            job.id,
            run.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token.clone())
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let resp = client
        .get(format!(
            "{}/api/jobs/{}/snapshots/{}/delete-task",
            base_url(addr),
            job.id,
            run.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("get task");
    assert_eq!(resp.status(), StatusCode::OK);
    let task_body: serde_json::Value = resp.json().await.expect("task json");
    assert_eq!(task_body["status"].as_str().unwrap_or_default(), "queued");

    let resp = client
        .get(format!(
            "{}/api/jobs/{}/snapshots/{}/delete-events",
            base_url(addr),
            job.id,
            run.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("get events");
    assert_eq!(resp.status(), StatusCode::OK);
    let events_body: serde_json::Value = resp.json().await.expect("events json");
    assert_eq!(events_body.as_array().map(|v| v.len()).unwrap_or(0), 1);

    let task = artifact_delete_repo::get_task(&pool, &run.id)
        .await
        .expect("get task")
        .expect("task exists");
    assert_eq!(task.status, "queued");

    let events = artifact_delete_repo::list_events(&pool, &run.id, 50)
        .await
        .expect("events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, "queued");

    server.abort();
}

#[tokio::test]
async fn delete_job_snapshots_bulk_rejects_empty_run_ids() {
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
        .post(format!(
            "{}/api/jobs/{}/snapshots/delete",
            base_url(addr),
            job.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token.clone())
        .json(&serde_json::json!({ "run_ids": [] }))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    server.abort();
}

#[tokio::test]
async fn pin_and_unpin_snapshot_and_force_delete_guardrail() {
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

    let run = runs_repo::create_run(
        &pool,
        &job.id,
        runs_repo::RunStatus::Queued,
        1,
        None,
        None,
        None,
    )
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

    // Pin.
    let resp = client
        .post(format!(
            "{}/api/jobs/{}/snapshots/{}/pin",
            base_url(addr),
            job.id,
            run.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token.clone())
        .send()
        .await
        .expect("pin");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Delete should require force while pinned.
    let resp = client
        .post(format!(
            "{}/api/jobs/{}/snapshots/{}/delete",
            base_url(addr),
            job.id,
            run.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token.clone())
        .send()
        .await
        .expect("delete");
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let body: serde_json::Value = resp.json().await.expect("body");
    assert_eq!(
        body["error"].as_str().unwrap_or_default(),
        "snapshot_pinned"
    );

    // Force delete.
    let resp = client
        .post(format!(
            "{}/api/jobs/{}/snapshots/{}/delete?force=true",
            base_url(addr),
            job.id,
            run.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token.clone())
        .send()
        .await
        .expect("delete force");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Unpin is idempotent and should succeed.
    let resp = client
        .post(format!(
            "{}/api/jobs/{}/snapshots/{}/unpin",
            base_url(addr),
            job.id,
            run.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token.clone())
        .send()
        .await
        .expect("unpin");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    server.abort();
}
