use std::sync::Arc;

use axum::http::StatusCode;
use tempfile::TempDir;

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;
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
async fn list_job_runs_includes_consistency_digests() {
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
        "job1",
        None,
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({"v":1,"type":"filesystem"}),
    )
    .await
    .expect("create job");

    let run_no_consistency = runs_repo::create_run(
        &pool,
        &job.id,
        runs_repo::RunStatus::Success,
        1000,
        Some(1001),
        Some(serde_json::json!({ "executed_offline": false })),
        None,
    )
    .await
    .expect("create run");

    let run_with_consistency = runs_repo::create_run(
        &pool,
        &job.id,
        runs_repo::RunStatus::Success,
        2000,
        Some(2001),
        Some(serde_json::json!({
            "executed_offline": false,
            "filesystem": {
                "consistency": {
                    "v": 1,
                    "changed_total": 1,
                    "replaced_total": 2,
                    "deleted_total": 3,
                    "read_error_total": 0,
                    "sample_truncated": false,
                    "sample": []
                }
            }
        })),
        None,
    )
    .await
    .expect("create run");

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
        .get(format!("{}/api/jobs/{}/runs", base_url(addr), job.id))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");
    let items = body.as_array().expect("array");
    assert_eq!(items.len(), 2);

    for item in items {
        let id = item["id"].as_str().unwrap_or_default();
        let total = item["consistency_total"].as_u64().unwrap_or_default();
        let signal = item["consistency_signal_total"]
            .as_u64()
            .unwrap_or_default();
        if id == run_no_consistency.id {
            assert_eq!(total, 0);
            assert_eq!(signal, 0);
        } else if id == run_with_consistency.id {
            assert_eq!(total, 6);
            assert_eq!(signal, 5);
        }
    }

    server.abort();
}

#[tokio::test]
async fn list_job_runs_derives_consistency_from_latest_event_when_summary_missing() {
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
        "job1",
        None,
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({"v":1,"type":"filesystem"}),
    )
    .await
    .expect("create job");

    let run = runs_repo::create_run(
        &pool,
        &job.id,
        runs_repo::RunStatus::Running,
        2000,
        None,
        None,
        None,
    )
    .await
    .expect("create run");

    runs_repo::append_run_event(
        &pool,
        &run.id,
        "warn",
        "source_consistency",
        "source consistency warnings",
        Some(serde_json::json!({
            "v": 1,
            "changed_total": 1,
            "replaced_total": 2,
            "deleted_total": 0,
            "read_error_total": 0,
            "sample_truncated": false,
            "sample": []
        })),
    )
    .await
    .expect("append event");

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
        .get(format!("{}/api/jobs/{}/runs", base_url(addr), job.id))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");
    let items = body.as_array().expect("array");

    let got = items
        .iter()
        .find(|v| v.get("id").and_then(|x| x.as_str()) == Some(run.id.as_str()))
        .expect("run item present");
    assert_eq!(got["consistency_total"].as_u64().unwrap_or(0), 3);
    assert_eq!(got["consistency_signal_total"].as_u64().unwrap_or(0), 2);

    server.abort();
}

#[tokio::test]
async fn list_job_runs_summary_takes_precedence_over_event() {
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
        "job1",
        None,
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({"v":1,"type":"filesystem"}),
    )
    .await
    .expect("create job");

    let run = runs_repo::create_run(
        &pool,
        &job.id,
        runs_repo::RunStatus::Success,
        1000,
        Some(1001),
        Some(serde_json::json!({
            "filesystem": {
                "consistency": {
                    "v": 1,
                    "changed_total": 1,
                    "replaced_total": 0,
                    "deleted_total": 0,
                    "read_error_total": 0,
                    "sample_truncated": false,
                    "sample": []
                }
            }
        })),
        None,
    )
    .await
    .expect("create run");

    // If we were to use events, this would inflate the count, but summary_json MUST win.
    runs_repo::append_run_event(
        &pool,
        &run.id,
        "warn",
        "source_consistency",
        "source consistency warnings",
        Some(serde_json::json!({
            "v": 1,
            "changed_total": 9,
            "replaced_total": 9,
            "deleted_total": 9,
            "read_error_total": 9,
            "sample_truncated": false,
            "sample": []
        })),
    )
    .await
    .expect("append event");

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
        .get(format!("{}/api/jobs/{}/runs", base_url(addr), job.id))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");
    let items = body.as_array().expect("array");

    let got = items
        .iter()
        .find(|v| v.get("id").and_then(|x| x.as_str()) == Some(run.id.as_str()))
        .expect("run item present");
    assert_eq!(got["consistency_total"].as_u64().unwrap_or(0), 1);
    assert_eq!(got["consistency_signal_total"].as_u64().unwrap_or(0), 0);

    server.abort();
}

#[tokio::test]
async fn list_job_runs_includes_issues_digests() {
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
        "job1",
        None,
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({"v":1,"type":"filesystem"}),
    )
    .await
    .expect("create job");

    let run = runs_repo::create_run(
        &pool,
        &job.id,
        runs_repo::RunStatus::Success,
        1000,
        Some(1001),
        Some(serde_json::json!({
            "executed_offline": false,
            "filesystem": {
                "warnings_total": 2,
                "errors_total": 1,
                "consistency": {
                    "v": 1,
                    "changed_total": 0,
                    "replaced_total": 0,
                    "deleted_total": 0,
                    "read_error_total": 0,
                    "sample_truncated": false,
                    "sample": []
                }
            }
        })),
        None,
    )
    .await
    .expect("create run");

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
        .get(format!("{}/api/jobs/{}/runs", base_url(addr), job.id))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");
    let items = body.as_array().expect("array");

    let got = items
        .iter()
        .find(|v| v.get("id").and_then(|x| x.as_str()) == Some(run.id.as_str()))
        .expect("run item present");
    assert_eq!(got["issues_warnings_total"].as_u64().unwrap_or(0), 2);
    assert_eq!(got["issues_errors_total"].as_u64().unwrap_or(0), 1);

    server.abort();
}
