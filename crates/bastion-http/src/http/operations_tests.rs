use std::sync::Arc;

use axum::extract::ws::Message as WsMessage;
use axum::http::StatusCode;
use tempfile::TempDir;

use bastion_config::Config;
use bastion_core::agent_protocol::{HubToAgentMessageV1, PROTOCOL_VERSION};
use bastion_engine::agent_manager::AgentManager;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::{agent_tasks_repo, auth, db, jobs_repo, operations_repo, runs_repo};

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
async fn list_run_operations_returns_linked_ops() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let user_password = uuid::Uuid::new_v4().to_string();
    auth::create_user(&pool, "admin", &user_password)
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
        Some(1010),
        None,
        None,
    )
    .await
    .expect("create run");

    let op1 = operations_repo::create_operation(
        &pool,
        operations_repo::OperationKind::Restore,
        Some(("run", run.id.as_str())),
    )
    .await
    .expect("create op1");
    let op2 = operations_repo::create_operation(
        &pool,
        operations_repo::OperationKind::Verify,
        Some(("run", run.id.as_str())),
    )
    .await
    .expect("create op2");

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
        .get(format!("{}/api/runs/{}/operations", base_url(addr), run.id))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.expect("json");
    let ids = body
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|v| v.get("id").and_then(|x| x.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&op1.id.as_str()));
    assert!(ids.contains(&op2.id.as_str()));

    server.abort();
}

#[tokio::test]
async fn list_run_operations_returns_404_for_missing_run() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let user_password = uuid::Uuid::new_v4().to_string();
    auth::create_user(&pool, "admin", &user_password)
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
        .get(format!(
            "{}/api/runs/{}/operations",
            base_url(addr),
            "00000000-0000-0000-0000-000000000000"
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("request");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["error"].as_str().unwrap_or_default(), "run_not_found");

    server.abort();
}

#[tokio::test]
async fn cancel_operation_marks_running_operation_cancel_requested() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let user_password = uuid::Uuid::new_v4().to_string();
    auth::create_user(&pool, "admin", &user_password)
        .await
        .expect("create user");
    let user = auth::find_user_by_username(&pool, "admin")
        .await
        .expect("find user")
        .expect("user exists");
    let session = auth::create_session(&pool, user.id)
        .await
        .expect("create session");

    let op = operations_repo::create_operation(&pool, operations_repo::OperationKind::Verify, None)
        .await
        .expect("create operation");

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
            "{}/api/operations/{}/cancel",
            base_url(addr),
            op.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token.clone())
        .json(&serde_json::json!({ "reason": "manual" }))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["status"].as_str().unwrap_or_default(), "running");
    assert_eq!(body["cancel_reason"].as_str().unwrap_or_default(), "manual");
    assert!(body["cancel_requested_at"].as_i64().is_some());

    let fetched = operations_repo::get_operation(&pool, &op.id)
        .await
        .expect("get operation")
        .expect("present");
    assert_eq!(fetched.status, operations_repo::OperationStatus::Running);
    assert!(fetched.cancel_requested_at.is_some());
    assert_eq!(fetched.cancel_requested_by_user_id, Some(user.id));

    server.abort();
}

#[tokio::test]
async fn cancel_operation_dispatches_agent_cancel_message_when_task_exists() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let user_password = uuid::Uuid::new_v4().to_string();
    auth::create_user(&pool, "admin", &user_password)
        .await
        .expect("create user");
    let user = auth::find_user_by_username(&pool, "admin")
        .await
        .expect("find user")
        .expect("user exists");
    let session = auth::create_session(&pool, user.id)
        .await
        .expect("create session");

    let op = operations_repo::create_operation(&pool, operations_repo::OperationKind::Verify, None)
        .await
        .expect("create operation");
    agent_tasks_repo::upsert_task(
        &pool,
        &op.id,
        "agent-1",
        "run-1",
        "sent",
        &serde_json::json!({}),
    )
    .await
    .expect("upsert task");

    let agent_manager = AgentManager::default();
    let (agent_tx, mut agent_rx) = tokio::sync::mpsc::channel::<WsMessage>(8);
    agent_manager
        .register("agent-1".to_string(), agent_tx)
        .await;

    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));
    let app = super::router(super::AppState {
        config,
        db: pool.clone(),
        secrets,
        agent_manager: agent_manager.clone(),
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
            "{}/api/operations/{}/cancel",
            base_url(addr),
            op.id
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token.clone())
        .json(&serde_json::json!({ "reason": "manual" }))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::OK);

    let msg = tokio::time::timeout(std::time::Duration::from_secs(2), agent_rx.recv())
        .await
        .expect("cancel msg timeout")
        .expect("cancel msg");
    let WsMessage::Text(text) = msg else {
        panic!("expected text message");
    };
    let decoded = serde_json::from_str::<HubToAgentMessageV1>(&text).expect("decode protocol");
    match decoded {
        HubToAgentMessageV1::CancelOperationTask { v, op_id } => {
            assert_eq!(v, PROTOCOL_VERSION);
            assert_eq!(op_id, op.id);
        }
        other => panic!("unexpected message: {other:?}"),
    }

    server.abort();
}
