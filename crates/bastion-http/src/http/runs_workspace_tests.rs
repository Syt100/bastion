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

async fn create_authed_session(pool: &sqlx::SqlitePool) -> (reqwest::Client, String) {
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
async fn list_runs_workspace_filters_by_scope_and_status() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let now = time::OffsetDateTime::now_utc().unix_timestamp();

    insert_agent(&pool, "agent-1", "Agent One", Some(100), None).await;
    let hub_job = jobs_repo::create_job(
        &pool,
        "Hub job",
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
        "Agent job",
        Some("agent-1"),
        None,
        Some("UTC"),
        jobs_repo::OverlapPolicy::Queue,
        serde_json::json!({"v":1,"type":"filesystem"}),
    )
    .await
    .expect("create agent job");

    let _hub_run = runs_repo::create_run(
        &pool,
        &hub_job.id,
        runs_repo::RunStatus::Success,
        now - 300,
        Some(now - 250),
        None,
        None,
    )
    .await
    .expect("create hub run");

    let failed_run = runs_repo::create_run(
        &pool,
        &agent_job.id,
        runs_repo::RunStatus::Failed,
        now - 200,
        Some(now - 120),
        None,
        Some("upload_failed"),
    )
    .await
    .expect("create failed run");
    runs_repo::set_run_progress(
        &pool,
        &failed_run.id,
        Some(serde_json::json!({"kind":"backup","stage":"upload"})),
    )
    .await
    .expect("set progress");

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

    let (client, session_id) = create_authed_session(&pool).await;
    let resp = client
        .get(format!(
            "{}/api/runs?scope=agent:agent-1&status=failed&range=24h",
            base_url(addr)
        ))
        .header("cookie", format!("bastion_session={session_id}"))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["scope"]["effective"].as_str(), Some("agent:agent-1"));
    assert_eq!(body["filters"]["status"].as_str(), Some("failed"));
    assert_eq!(body["total"].as_i64(), Some(1));
    assert_eq!(body["items"][0]["job_name"].as_str(), Some("Agent job"));
    assert_eq!(body["items"][0]["scope"].as_str(), Some("agent:agent-1"));
    assert_eq!(body["items"][0]["status"].as_str(), Some("failed"));
    assert_eq!(body["items"][0]["kind"].as_str(), Some("backup"));

    server.abort();
}

#[tokio::test]
async fn get_run_workspace_returns_structured_diagnostics() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let job = jobs_repo::create_job(
        &pool,
        "Nightly DB",
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
        runs_repo::RunStatus::Failed,
        1_760_000_000,
        Some(1_760_000_100),
        Some(serde_json::json!({"target":{"type":"webdav","run_url":"https://dav.example/r1"}})),
        Some("upload_failed"),
    )
    .await
    .expect("create run");
    runs_repo::set_run_progress(
        &pool,
        &run.id,
        Some(serde_json::json!({"kind":"backup","stage":"upload"})),
    )
    .await
    .expect("set progress");
    runs_repo::append_run_event(
        &pool,
        &run.id,
        "info",
        "upload_started",
        "upload started",
        None,
    )
    .await
    .expect("append info event");
    runs_repo::append_run_event(
        &pool,
        &run.id,
        "error",
        "upload_failed",
        "WebDAV upload failed",
        Some(serde_json::json!({
            "stage": "upload",
            "error_envelope": {
                "kind": "transport",
                "retriable": { "value": true },
                "transport": { "protocol": "webdav" }
            }
        })),
    )
    .await
    .expect("append error event");

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

    let (client, session_id) = create_authed_session(&pool).await;
    let resp = client
        .get(format!("{}/api/runs/{}/workspace", base_url(addr), run.id))
        .header("cookie", format!("bastion_session={session_id}"))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["run"]["id"].as_str(), Some(run.id.as_str()));
    assert_eq!(body["run"]["kind"].as_str(), Some("backup"));
    assert_eq!(body["diagnostics"]["state"].as_str(), Some("structured"));
    assert_eq!(body["diagnostics"]["failure_kind"].as_str(), Some("transport"));
    assert_eq!(body["diagnostics"]["failure_stage"].as_str(), Some("upload"));
    assert_eq!(
        body["diagnostics"]["failure_title"].as_str(),
        Some("WebDAV upload failed")
    );
    assert_eq!(body["diagnostics"]["first_error_event_seq"].as_i64(), Some(2));
    assert_eq!(body["capabilities"]["can_restore"].as_bool(), Some(false));
    assert_eq!(body["capabilities"]["can_cancel"].as_bool(), Some(false));

    server.abort();
}

#[tokio::test]
async fn run_event_console_supports_filters_and_windows() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let job = jobs_repo::create_job(
        &pool,
        "Nightly FS",
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
        runs_repo::RunStatus::Failed,
        1_760_000_000,
        Some(1_760_000_100),
        None,
        Some("failed"),
    )
    .await
    .expect("create run");
    runs_repo::append_run_event(&pool, &run.id, "info", "queued", "queued", None)
        .await
        .expect("append queued");
    runs_repo::append_run_event(
        &pool,
        &run.id,
        "error",
        "upload_failed",
        "first upload failure",
        Some(serde_json::json!({"error_envelope":{"kind":"transport","retriable":{"value":false},"transport":{"protocol":"webdav"}}})),
    )
    .await
    .expect("append first error");
    runs_repo::append_run_event(
        &pool,
        &run.id,
        "warn",
        "retry_scheduled",
        "retry scheduled",
        None,
    )
    .await
    .expect("append warn");
    runs_repo::append_run_event(
        &pool,
        &run.id,
        "error",
        "verify_failed",
        "second error",
        None,
    )
    .await
    .expect("append second error");

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

    let (client, session_id) = create_authed_session(&pool).await;
    let resp = client
        .get(format!(
            "{}/api/runs/{}/event-console?levels=error&anchor=first_error&limit=1",
            base_url(addr),
            run.id
        ))
        .header("cookie", format!("bastion_session={session_id}"))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["filters"]["levels"][0].as_str(), Some("error"));
    assert_eq!(body["locators"]["first_error_seq"].as_i64(), Some(2));
    assert_eq!(body["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(body["items"][0]["seq"].as_i64(), Some(2));
    assert_eq!(body["window"]["has_newer"].as_bool(), Some(true));

    let resp = client
        .get(format!(
            "{}/api/runs/{}/event-console?levels=error&after_seq=2&limit=1",
            base_url(addr),
            run.id
        ))
        .header("cookie", format!("bastion_session={session_id}"))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["items"][0]["seq"].as_i64(), Some(4));
    assert_eq!(body["window"]["has_older"].as_bool(), Some(true));

    server.abort();
}
