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
async fn archive_job_without_cascade_does_not_enqueue_snapshot_deletes() {
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
        .post(format!("{}/api/jobs/{}/archive", base_url(addr), job.id))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token.clone())
        .send()
        .await
        .expect("archive");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let got = jobs_repo::get_job(&pool, &job.id)
        .await
        .expect("get job")
        .expect("job exists");
    assert!(got.archived_at.is_some());

    // No snapshot deletes should be queued.
    let tasks =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM artifact_delete_tasks WHERE job_id = ?")
            .bind(&job.id)
            .fetch_one(&pool)
            .await
            .expect("count");
    assert_eq!(tasks, 0);

    server.abort();
}

#[tokio::test]
async fn archive_job_with_cascade_enqueues_deletes_for_unpinned_snapshots_only() {
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

    let run_unpinned = runs_repo::create_run(
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
        &run_unpinned.id,
        serde_json::json!({
            "node_id": "hub",
            "target": { "type": "local_dir", "base_dir": "/tmp" }
        }),
    )
    .await
    .expect("set snapshot");
    runs_repo::complete_run(
        &pool,
        &run_unpinned.id,
        runs_repo::RunStatus::Success,
        Some(serde_json::json!({ "artifact_format": "archive_v1" })),
        None,
    )
    .await
    .expect("complete run");
    run_artifacts_repo::upsert_run_artifact_from_successful_run(&pool, &run_unpinned.id)
        .await
        .expect("index");

    let run_pinned = runs_repo::create_run(
        &pool,
        &job.id,
        runs_repo::RunStatus::Queued,
        2,
        None,
        None,
        None,
    )
    .await
    .expect("create run pinned");
    runs_repo::set_run_target_snapshot(
        &pool,
        &run_pinned.id,
        serde_json::json!({
            "node_id": "hub",
            "target": { "type": "local_dir", "base_dir": "/tmp" }
        }),
    )
    .await
    .expect("set snapshot pinned");
    runs_repo::complete_run(
        &pool,
        &run_pinned.id,
        runs_repo::RunStatus::Success,
        Some(serde_json::json!({ "artifact_format": "archive_v1" })),
        None,
    )
    .await
    .expect("complete run pinned");
    run_artifacts_repo::upsert_run_artifact_from_successful_run(&pool, &run_pinned.id)
        .await
        .expect("index pinned");

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    run_artifacts_repo::pin_run_artifact(&pool, &run_pinned.id, user.id, now)
        .await
        .expect("pin");

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
            "{}/api/jobs/{}/archive?cascade_snapshots=true",
            base_url(addr),
            job.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token.clone())
        .send()
        .await
        .expect("archive");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Unpinned snapshot is enqueued and marked deleting.
    let task = artifact_delete_repo::get_task(&pool, &run_unpinned.id)
        .await
        .expect("get task")
        .expect("task exists");
    assert_eq!(task.status, "queued");
    let art = run_artifacts_repo::get_run_artifact(&pool, &run_unpinned.id)
        .await
        .expect("get artifact")
        .expect("artifact exists");
    assert_eq!(art.status, "deleting");

    // Pinned snapshot is excluded.
    let pinned_task = artifact_delete_repo::get_task(&pool, &run_pinned.id)
        .await
        .expect("get pinned task");
    assert!(pinned_task.is_none());
    let pinned_art = run_artifacts_repo::get_run_artifact(&pool, &run_pinned.id)
        .await
        .expect("get pinned artifact")
        .expect("artifact exists");
    assert_eq!(pinned_art.status, "present");

    server.abort();
}

#[tokio::test]
async fn archive_job_with_cascade_enqueues_deletes_across_pages() {
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

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let snapshot_json =
        serde_json::json!({ "node_id": "hub", "target": { "type": "local_dir", "base_dir": "/tmp" } })
            .to_string();

    // Create more than one page worth of snapshots so OFFSET-based pagination would skip rows when
    // we mark status=deleting while scanning status=present.
    const N: usize = 450;
    for i in 0..N {
        let ended_at = now - i as i64;
        let run = runs_repo::create_run(
            &pool,
            &job.id,
            runs_repo::RunStatus::Success,
            ended_at - 10,
            Some(ended_at),
            Some(serde_json::json!({ "artifact_format": "archive_v1" })),
            None,
        )
        .await
        .expect("create run");

        sqlx::query(
            r#"
            INSERT INTO run_artifacts (
              run_id, job_id, node_id, target_type, target_snapshot_json,
              artifact_format, status, started_at, ended_at,
              created_at, updated_at
            ) VALUES (?, ?, 'hub', 'local_dir', ?, 'archive_v1', 'present', ?, ?, ?, ?)
            "#,
        )
        .bind(&run.id)
        .bind(&job.id)
        .bind(&snapshot_json)
        .bind(ended_at - 10)
        .bind(ended_at)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert artifact");
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
    let resp = client
        .post(format!(
            "{}/api/jobs/{}/archive?cascade_snapshots=true",
            base_url(addr),
            job.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token.clone())
        .send()
        .await
        .expect("archive");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let tasks =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM artifact_delete_tasks WHERE job_id = ?")
            .bind(&job.id)
            .fetch_one(&pool)
            .await
            .expect("count");
    assert_eq!(tasks as usize, N);

    server.abort();
}
