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
    let user_password = uuid::Uuid::new_v4().to_string();
    auth::create_user(pool, "admin", &user_password)
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

fn app_state(
    config: Arc<Config>,
    db: sqlx::SqlitePool,
    secrets: Arc<SecretsCrypto>,
) -> super::AppState {
    super::AppState {
        config,
        db,
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
    }
}

#[tokio::test]
async fn list_jobs_workspace_returns_empty_state_with_filter_metadata() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let session = seed_admin_session(&pool).await;

    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));
    let app = super::router(app_state(config, pool.clone(), secrets));

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
            "{}/api/jobs/workspace?page=1&page_size=20",
            base_url(addr)
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.json::<serde_json::Value>().await.expect("json");
    assert_eq!(body["scope"]["requested"].as_str(), Some("all"));
    assert_eq!(body["scope"]["effective"].as_str(), Some("all"));
    assert_eq!(body["filters"]["q"].as_str(), Some(""));
    assert_eq!(body["filters"]["latest_status"].as_str(), Some("all"));
    assert_eq!(body["filters"]["schedule_mode"].as_str(), Some("all"));
    assert_eq!(body["filters"]["include_archived"].as_bool(), Some(false));
    assert_eq!(body["filters"]["sort"].as_str(), Some("updated_desc"));
    assert_eq!(body["page"].as_i64(), Some(1));
    assert_eq!(body["page_size"].as_i64(), Some(20));
    assert_eq!(body["total"].as_i64(), Some(0));
    assert_eq!(body["items"].as_array().map(|items| items.len()), Some(0));

    server.abort();
}

#[tokio::test]
async fn list_jobs_workspace_reports_degraded_and_archived_rows() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let session = seed_admin_session(&pool).await;

    let failed_job = jobs_repo::create_job(
        &pool,
        "failed-manual",
        None,
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({
          "v": 1,
          "type": "filesystem",
          "source": { "paths": ["/srv/data"] },
          "target": { "type": "local_dir", "base_dir": "/srv/backups" }
        }),
    )
    .await
    .expect("create failed job");

    runs_repo::create_run(
        &pool,
        &failed_job.id,
        runs_repo::RunStatus::Failed,
        200,
        Some(210),
        None,
        Some("boom"),
    )
    .await
    .expect("create failed run");

    let archived_job = jobs_repo::create_job(
        &pool,
        "archived-node",
        Some("node-1"),
        Some("0 * * * *"),
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({
          "v": 1,
          "type": "filesystem",
          "source": { "paths": ["/srv/node"] },
          "target": { "type": "webdav", "base_url": "https://dav.example.com", "secret_name": "dav-main" }
        }),
    )
    .await
    .expect("create archived job");

    jobs_repo::archive_job(&pool, &archived_job.id)
        .await
        .expect("archive job");

    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));
    let app = super::router(app_state(config, pool.clone(), secrets));

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
            "{}/api/jobs/workspace?page=1&page_size=20&include_archived=true&sort=name_asc",
            base_url(addr)
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.json::<serde_json::Value>().await.expect("json");
    assert_eq!(body["filters"]["include_archived"].as_bool(), Some(true));
    assert_eq!(body["filters"]["sort"].as_str(), Some("name_asc"));
    assert_eq!(body["total"].as_i64(), Some(2));

    let items = body["items"].as_array().expect("items array");

    let archived_row = items
        .iter()
        .find(|item| item["id"].as_str() == Some(archived_job.id.as_str()))
        .expect("archived row");
    assert_eq!(archived_row["scope"].as_str(), Some("agent:node-1"));
    assert_eq!(archived_row["health"].as_str(), Some("archived"));
    assert_eq!(
        archived_row["warnings"]
            .as_array()
            .expect("warnings")
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>(),
        vec!["archived"]
    );
    assert_eq!(
        archived_row["capabilities"]["can_run_now"].as_bool(),
        Some(false)
    );
    assert_eq!(
        archived_row["capabilities"]["can_edit"].as_bool(),
        Some(false)
    );
    assert_eq!(
        archived_row["capabilities"]["can_unarchive"].as_bool(),
        Some(true)
    );

    let failed_row = items
        .iter()
        .find(|item| item["id"].as_str() == Some(failed_job.id.as_str()))
        .expect("failed row");
    assert_eq!(failed_row["scope"].as_str(), Some("hub"));
    assert_eq!(failed_row["health"].as_str(), Some("critical"));
    assert_eq!(failed_row["latest_failure_at"].as_i64(), Some(210));
    assert_eq!(failed_row["next_run_at"].as_i64(), None);
    let warnings = failed_row["warnings"]
        .as_array()
        .expect("warnings")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    assert!(warnings.contains(&"latest_run_failed"));
    assert!(warnings.contains(&"no_successful_backup"));
    assert!(warnings.contains(&"manual_only"));
    assert_eq!(
        failed_row["capabilities"]["can_run_now"].as_bool(),
        Some(true)
    );
    assert_eq!(
        failed_row["capabilities"]["can_archive"].as_bool(),
        Some(true)
    );

    server.abort();
}

#[tokio::test]
async fn get_job_workspace_returns_summary_recent_runs_and_capabilities() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let session = seed_admin_session(&pool).await;

    let job = jobs_repo::create_job(
        &pool,
        "nightly-db",
        None,
        Some("0 0 * * *"),
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({
          "v": 1,
          "type": "filesystem",
          "source": { "paths": ["/var/lib/postgresql"] },
          "target": { "type": "webdav", "base_url": "https://dav.example.com/backups", "secret_name": "dav-main" }
        }),
    )
    .await
    .expect("create job");

    let success_run = runs_repo::create_run(
        &pool,
        &job.id,
        runs_repo::RunStatus::Success,
        100,
        Some(110),
        None,
        None,
    )
    .await
    .expect("create success run");
    let failed_run = runs_repo::create_run(
        &pool,
        &job.id,
        runs_repo::RunStatus::Failed,
        200,
        Some(210),
        None,
        Some("boom"),
    )
    .await
    .expect("create failed run");

    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));
    let app = super::router(app_state(config, pool.clone(), secrets));

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
        .get(format!("{}/api/jobs/{}/workspace", base_url(addr), job.id))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.json::<serde_json::Value>().await.expect("json");
    assert_eq!(body["job"]["id"].as_str(), Some(job.id.as_str()));
    assert_eq!(body["summary"]["latest_success_at"].as_i64(), Some(110));
    assert_eq!(body["summary"]["latest_failure_at"].as_i64(), Some(210));
    assert_eq!(
        body["summary"]["latest_run_status"].as_str(),
        Some("failed")
    );
    assert_eq!(body["summary"]["latest_run_started_at"].as_i64(), Some(200));
    assert_eq!(body["summary"]["latest_run_ended_at"].as_i64(), Some(210));
    assert_eq!(body["summary"]["target_type"].as_str(), Some("webdav"));
    assert_eq!(
        body["summary"]["target_label"].as_str(),
        Some("https://dav.example.com/backups")
    );
    assert_eq!(
        body["summary"]["schedule_label"].as_str(),
        Some("0 0 * * *")
    );
    assert_eq!(body["readiness"]["state"].as_str(), Some("critical"));
    assert_eq!(body["readiness"]["last_success_at"].as_i64(), Some(110));
    assert_eq!(body["capabilities"]["can_run_now"].as_bool(), Some(true));
    assert_eq!(body["capabilities"]["can_edit"].as_bool(), Some(true));
    assert_eq!(body["capabilities"]["can_delete"].as_bool(), Some(true));

    let warnings = body["warnings"]
        .as_array()
        .expect("warnings")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    assert!(warnings.contains(&"latest_run_failed"));
    assert!(!warnings.contains(&"manual_only"));

    let recent_runs = body["recent_runs"].as_array().expect("recent runs");
    assert_eq!(recent_runs.len(), 2);
    assert_eq!(recent_runs[0]["id"].as_str(), Some(failed_run.id.as_str()));
    assert_eq!(recent_runs[1]["id"].as_str(), Some(success_run.id.as_str()));

    server.abort();
}
