use std::sync::Arc;

use axum::http::StatusCode;
use tempfile::TempDir;

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;
use bastion_storage::auth;
use bastion_storage::db;
use bastion_storage::hub_runtime_config_repo;
use bastion_storage::jobs_repo;
use bastion_storage::runs_repo;
use bastion_storage::secrets::SecretsCrypto;
use sqlx::Row;

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
async fn retention_preview_requires_auth() {
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
            "{}/api/jobs/{}/retention/preview",
            base_url(addr),
            "job1"
        ))
        .json(&serde_json::json!({ "retention": { "enabled": true, "keep_last": 1 } }))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    server.abort();
}

#[tokio::test]
async fn create_job_inherits_default_retention_when_missing() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    // Save a hub default retention policy.
    hub_runtime_config_repo::upsert(
        &pool,
        &hub_runtime_config_repo::HubRuntimeConfig {
            default_backup_retention: bastion_core::job_spec::RetentionPolicyV1 {
                enabled: true,
                keep_last: Some(7),
                keep_days: Some(30),
                max_delete_per_tick: 20,
                max_delete_per_day: 100,
            },
            ..Default::default()
        },
    )
    .await
    .expect("upsert hub runtime config");

    let session = seed_admin_session(&pool).await;

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
        .post(format!("{}/api/jobs", base_url(addr)))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("X-CSRF-Token", &session.csrf_token)
        .json(&serde_json::json!({
          "name": "job",
          "agent_id": null,
          "schedule": null,
          "schedule_timezone": "UTC",
          "overlap_policy": "queue",
          "spec": {
            "v": 1,
            "type": "filesystem",
            "source": { "paths": ["/tmp"] },
            "target": { "type": "local_dir", "base_dir": "/tmp" }
          }
        }))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.json::<serde_json::Value>().await.expect("json");
    let retention = body
        .get("spec")
        .and_then(|v| v.as_object())
        .expect("spec")
        .get("retention")
        .and_then(|v| v.as_object())
        .expect("retention");
    assert_eq!(retention.get("enabled").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(retention.get("keep_last").and_then(|v| v.as_u64()), Some(7));
    assert_eq!(retention.get("keep_days").and_then(|v| v.as_u64()), Some(30));

    server.abort();
}

#[tokio::test]
async fn retention_preview_and_apply_work_and_are_bounded() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let session = seed_admin_session(&pool).await;

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
          "source": { "paths": ["/tmp"] },
          "target": { "type": "local_dir", "base_dir": "/tmp" }
        }),
    )
    .await
    .expect("create job");

    let now = time::OffsetDateTime::now_utc().unix_timestamp();

    // Three snapshots: newest two should be kept by keep_days, oldest should be deleted.
    let run_new = runs_repo::create_run(&pool, &job.id, runs_repo::RunStatus::Success, 1, None, None, None)
        .await
        .expect("run");
    let run_mid = runs_repo::create_run(&pool, &job.id, runs_repo::RunStatus::Success, 1, None, None, None)
        .await
        .expect("run");
    let run_old = runs_repo::create_run(&pool, &job.id, runs_repo::RunStatus::Success, 1, None, None, None)
        .await
        .expect("run");

    // Insert run_artifacts rows directly (tests focus on retention, not indexing).
    for (run_id, ended_at, pinned_at) in [
        (&run_new.id, now - 10, None::<i64>),
        (&run_mid.id, now - 20, None::<i64>),
        (&run_old.id, now - (2 * 24 * 60 * 60 + 10), None::<i64>),
    ] {
        sqlx::query(
            r#"
            INSERT INTO run_artifacts (
              run_id, job_id, node_id, target_type, target_snapshot_json,
              artifact_format, status, started_at, ended_at,
              pinned_at, pinned_by_user_id,
              created_at, updated_at
            ) VALUES (?, ?, 'hub', 'local_dir', ?, 'archive_v1', 'present', ?, ?, ?, NULL, ?, ?)
            "#,
        )
        .bind(run_id)
        .bind(&job.id)
        .bind(serde_json::json!({ "node_id": "hub", "target": { "type": "local_dir", "base_dir": "/tmp" } }).to_string())
        .bind(now - 100)
        .bind(ended_at)
        .bind(pinned_at)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert run_artifacts");
    }

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

    // Preview: keep_last=1, keep_days=1 -> keep newest two, delete oldest.
    let preview = client
        .post(format!(
            "{}/api/jobs/{}/retention/preview",
            base_url(addr),
            job.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .json(&serde_json::json!({
          "retention": { "enabled": true, "keep_last": 1, "keep_days": 1 }
        }))
        .send()
        .await
        .expect("request");
    assert_eq!(preview.status(), StatusCode::OK);
    let preview_body = preview.json::<serde_json::Value>().await.expect("json");
    assert_eq!(preview_body.get("keep_total").and_then(|v| v.as_u64()), Some(2));
    assert_eq!(preview_body.get("delete_total").and_then(|v| v.as_u64()), Some(1));

    // Apply with a strict per-tick limit so only one delete is enqueued.
    let apply = client
        .post(format!(
            "{}/api/jobs/{}/retention/apply",
            base_url(addr),
            job.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("X-CSRF-Token", &session.csrf_token)
        .json(&serde_json::json!({
          "retention": { "enabled": true, "keep_last": 1, "keep_days": 1, "max_delete_per_tick": 1, "max_delete_per_day": 1 }
        }))
        .send()
        .await
        .expect("request");
    assert_eq!(apply.status(), StatusCode::OK);

    let apply_body = apply.json::<serde_json::Value>().await.expect("json");
    assert_eq!(apply_body.get("enqueued").and_then(|v| v.as_array()).map(|a| a.len()), Some(1));

    // The oldest run should be marked deleting (retention selected it).
    let row = sqlx::query("SELECT status FROM run_artifacts WHERE run_id = ?")
        .bind(&run_old.id)
        .fetch_one(&pool)
        .await
        .expect("select");
    let status = row.get::<String, _>("status");
    assert_eq!(status, "deleting");

    server.abort();
}
