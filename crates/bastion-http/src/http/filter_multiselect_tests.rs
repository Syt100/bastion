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
async fn cleanup_list_accepts_multi_value_query_params() {
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

    // Seed a job, runs, and two cleanup tasks with distinct status + target_type.
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    sqlx::query(
        "INSERT INTO jobs (id, name, schedule, overlap_policy, spec_json, created_at, updated_at) VALUES (?, ?, NULL, 'queue', ?, ?, ?)",
    )
    .bind("job1")
    .bind("job1")
    .bind(r#"{"v":1,"type":"filesystem","source":{"root":"/"},"target":{"type":"local_dir","base_dir":"/tmp"}}"#)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .expect("insert job");

    sqlx::query("INSERT INTO runs (id, job_id, status, started_at, ended_at) VALUES (?, ?, 'failed', ?, ?)")
        .bind("run1")
        .bind("job1")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert run1");
    sqlx::query("INSERT INTO runs (id, job_id, status, started_at, ended_at) VALUES (?, ?, 'failed', ?, ?)")
        .bind("run2")
        .bind("job1")
        .bind(now + 1)
        .bind(now + 1)
        .execute(&pool)
        .await
        .expect("insert run2");

    let webdav_snapshot = serde_json::json!({
        "node_id": "hub",
        "target": { "type": "webdav", "base_url": "http://example/", "secret_name": "s" }
    });
    let local_snapshot = serde_json::json!({
        "node_id": "hub",
        "target": { "type": "local_dir", "base_dir": "/tmp" }
    });

    sqlx::query(
        r#"
        INSERT INTO incomplete_cleanup_tasks (
          run_id, job_id, node_id, target_type, target_snapshot_json,
          status, attempts, created_at, updated_at, next_attempt_at
        )
        VALUES (?, ?, 'hub', 'webdav', ?, 'queued', 0, ?, ?, ?)
        "#,
    )
    .bind("run1")
    .bind("job1")
    .bind(webdav_snapshot.to_string())
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .expect("insert cleanup task 1");

    sqlx::query(
        r#"
        INSERT INTO incomplete_cleanup_tasks (
          run_id, job_id, node_id, target_type, target_snapshot_json,
          status, attempts, created_at, updated_at, next_attempt_at
        )
        VALUES (?, ?, 'hub', 'local_dir', ?, 'done', 1, ?, ?, ?)
        "#,
    )
    .bind("run2")
    .bind("job1")
    .bind(local_snapshot.to_string())
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .expect("insert cleanup task 2");

    let app = super::router(super::AppState {
        config,
        db: pool.clone(),
        secrets,
        agent_manager: AgentManager::default(),
        run_queue_notify: Arc::new(tokio::sync::Notify::new()),
        incomplete_cleanup_notify: Arc::new(tokio::sync::Notify::new()),
        jobs_notify: Arc::new(tokio::sync::Notify::new()),
        notifications_notify: Arc::new(tokio::sync::Notify::new()),
        run_events_bus: Arc::new(bastion_engine::run_events_bus::RunEventsBus::new()),
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
    let cookie = format!("bastion_session={}", session.id);

    // Multi-value filters: should return only the matching row.
    for (path, expected_total) in [
        (
            "/api/maintenance/incomplete-cleanup?status[]=done",
            1,
        ),
        (
            "/api/maintenance/incomplete-cleanup?target_type[]=webdav",
            1,
        ),
        (
            "/api/maintenance/incomplete-cleanup?status[]=queued&status[]=done&target_type[]=webdav&target_type[]=local_dir",
            2,
        ),
        // Single-value (non-[]) style should still work.
        (
            "/api/maintenance/incomplete-cleanup?status=queued&target_type=webdav",
            1,
        ),
    ] {
        let resp = client
            .get(format!("{}{}", base_url(addr), path))
            .header("cookie", &cookie)
            .send()
            .await
            .expect("request");

        let status = resp.status();
        if status != StatusCode::OK {
            let text = resp.text().await.unwrap_or_default();
            panic!("expected 200, got {status}: {text}");
        }
        let body: serde_json::Value = resp.json().await.expect("json");
        assert_eq!(body["total"].as_i64(), Some(expected_total));
    }

    server.abort();
}

#[tokio::test]
async fn notifications_queue_list_accepts_multi_value_query_params() {
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

    // Seed job + run + 2 notifications with distinct status/channel.
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    sqlx::query(
        "INSERT INTO jobs (id, name, schedule, overlap_policy, spec_json, created_at, updated_at) VALUES (?, ?, NULL, 'queue', ?, ?, ?)",
    )
    .bind("job1")
    .bind("job1")
    .bind(r#"{"v":1,"type":"filesystem","source":{"root":"/"},"target":{"type":"local_dir","base_dir":"/tmp"}}"#)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .expect("insert job");
    sqlx::query("INSERT INTO runs (id, job_id, status, started_at, ended_at) VALUES (?, ?, 'success', ?, ?)")
        .bind("run1")
        .bind("job1")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert run");

    sqlx::query(
        "INSERT INTO notifications (id, run_id, channel, secret_name, status, attempts, next_attempt_at, created_at, updated_at, last_error) VALUES (?, ?, 'email', 'smtp1', 'failed', 0, ?, ?, ?, NULL)",
    )
    .bind("n1")
    .bind("run1")
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .expect("insert notification 1");
    sqlx::query(
        "INSERT INTO notifications (id, run_id, channel, secret_name, status, attempts, next_attempt_at, created_at, updated_at, last_error) VALUES (?, ?, 'wecom_bot', 'bot1', 'sent', 0, ?, ?, ?, NULL)",
    )
    .bind("n2")
    .bind("run1")
    .bind(now)
    .bind(now + 1)
    .bind(now + 1)
    .execute(&pool)
    .await
    .expect("insert notification 2");

    let app = super::router(super::AppState {
        config,
        db: pool.clone(),
        secrets,
        agent_manager: AgentManager::default(),
        run_queue_notify: Arc::new(tokio::sync::Notify::new()),
        incomplete_cleanup_notify: Arc::new(tokio::sync::Notify::new()),
        jobs_notify: Arc::new(tokio::sync::Notify::new()),
        notifications_notify: Arc::new(tokio::sync::Notify::new()),
        run_events_bus: Arc::new(bastion_engine::run_events_bus::RunEventsBus::new()),
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
    let cookie = format!("bastion_session={}", session.id);

    for (path, expected_total) in [
        (
            "/api/notifications/queue?status[]=sent&channel[]=wecom_bot",
            1,
        ),
        (
            "/api/notifications/queue?status[]=failed&status[]=sent&channel[]=email&channel[]=wecom_bot",
            2,
        ),
        (
            "/api/notifications/queue?status=failed&channel=email",
            1,
        ),
    ] {
        let resp = client
            .get(format!("{}{}", base_url(addr), path))
            .header("cookie", &cookie)
            .send()
            .await
            .expect("request");

        let status = resp.status();
        if status != StatusCode::OK {
            let text = resp.text().await.unwrap_or_default();
            panic!("expected 200, got {status}: {text}");
        }
        let body: serde_json::Value = resp.json().await.expect("json");
        assert_eq!(body["total"].as_i64(), Some(expected_total));
    }

    server.abort();
}
