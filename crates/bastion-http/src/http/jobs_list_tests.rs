use std::sync::Arc;

use axum::http::StatusCode;
use tempfile::TempDir;

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;
use bastion_storage::auth;
use bastion_storage::db;
use bastion_storage::jobs_repo;
use bastion_storage::runs_repo;
use bastion_storage::secrets::SecretsCrypto;

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

async fn seed_admin_session(pool: &sqlx::SqlitePool) -> auth::SessionRow {
    auth::create_user(pool, "admin", "pw")
        .await
        .expect("create user");
    let user = auth::find_user_by_username(pool, "admin")
        .await
        .expect("find user")
        .expect("user exists");
    auth::create_session(pool, user.id)
        .await
        .expect("create session")
}

#[tokio::test]
async fn list_jobs_includes_latest_run_fields() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let session = seed_admin_session(&pool).await;

    let job1 = jobs_repo::create_job(
        &pool,
        "job1",
        None,
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({
          "v": 1,
          "type": "filesystem",
          "source": { "paths": ["/tmp"] },
          "target": { "type": "local_dir", "base_dir": "/tmp" }
        }),
    )
    .await
    .expect("create job1");

    let _job2 = jobs_repo::create_job(
        &pool,
        "job2",
        None,
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({
          "v": 1,
          "type": "filesystem",
          "source": { "paths": ["/tmp"] },
          "target": { "type": "local_dir", "base_dir": "/tmp" }
        }),
    )
    .await
    .expect("create job2");

    let _run1 = runs_repo::create_run(
        &pool,
        &job1.id,
        runs_repo::RunStatus::Success,
        100,
        Some(110),
        None,
        None,
    )
    .await
    .expect("create run1");
    let run2 = runs_repo::create_run(
        &pool,
        &job1.id,
        runs_repo::RunStatus::Failed,
        200,
        Some(210),
        None,
        None,
    )
    .await
    .expect("create run2");

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
        .get(format!("{}/api/jobs", base_url(addr)))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.json::<serde_json::Value>().await.expect("json");
    let arr = body.as_array().expect("array");

    let job1_row = arr
        .iter()
        .find(|v| v.get("id").and_then(|x| x.as_str()) == Some(job1.id.as_str()))
        .expect("job1 in list");
    assert_eq!(
        job1_row.get("latest_run_id").and_then(|v| v.as_str()),
        Some(run2.id.as_str())
    );
    assert_eq!(
        job1_row.get("latest_run_status").and_then(|v| v.as_str()),
        Some("failed")
    );
    assert_eq!(
        job1_row
            .get("latest_run_started_at")
            .and_then(|v| v.as_i64()),
        Some(200)
    );
    assert_eq!(
        job1_row.get("latest_run_ended_at").and_then(|v| v.as_i64()),
        Some(210)
    );

    let job2_row = arr
        .iter()
        .find(|v| v.get("name").and_then(|x| x.as_str()) == Some("job2"))
        .expect("job2 in list");
    assert!(
        job2_row
            .get("latest_run_status")
            .map(|v| v.is_null())
            .unwrap_or(false)
    );

    server.abort();
}
