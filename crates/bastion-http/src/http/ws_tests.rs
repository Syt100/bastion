use std::sync::Arc;

use futures_util::StreamExt;
use tempfile::TempDir;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::{auth, db, jobs_repo, runs_repo};

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;
use bastion_engine::run_events;
use bastion_engine::run_events_bus::RunEventsBus;

#[tokio::test]
async fn run_events_ws_supports_after_seq_and_push() {
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
        runs_repo::RunStatus::Queued,
        1000,
        None,
        None,
        None,
    )
    .await
    .expect("create run");

    let run_events_bus = Arc::new(RunEventsBus::new_with_options(8, 60, 1));
    let _ =
        run_events::append_and_broadcast(&pool, &run_events_bus, &run.id, "info", "e1", "e1", None)
            .await
            .expect("event1");
    let _ =
        run_events::append_and_broadcast(&pool, &run_events_bus, &run.id, "info", "e2", "e2", None)
            .await
            .expect("event2");

    let config = Arc::new(Config {
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
    });
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
        run_events_bus: run_events_bus.clone(),
        hub_runtime_config: Default::default(),
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");

    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve");
    });

    let url = format!("ws://{}/api/runs/{}/events/ws?after=1", addr, run.id);
    let mut req = url.into_client_request().expect("ws request");
    req.headers_mut()
        .insert("origin", format!("http://{addr}").parse().expect("origin"));
    req.headers_mut().insert(
        "cookie",
        format!("bastion_session={}", session.id)
            .parse()
            .expect("cookie"),
    );

    let (mut socket, _) = tokio_tungstenite::connect_async(req)
        .await
        .expect("ws connect");

    let msg = tokio::time::timeout(std::time::Duration::from_secs(1), socket.next())
        .await
        .expect("recv timeout")
        .expect("recv some")
        .expect("recv ok");
    let text = msg.into_text().expect("text");
    let first: serde_json::Value = serde_json::from_str(&text).expect("json");
    assert_eq!(first["seq"].as_i64().unwrap_or_default(), 2);

    let _ =
        run_events::append_and_broadcast(&pool, &run_events_bus, &run.id, "info", "e3", "e3", None)
            .await
            .expect("event3");

    let msg = tokio::time::timeout(std::time::Duration::from_secs(1), socket.next())
        .await
        .expect("recv timeout")
        .expect("recv some")
        .expect("recv ok");
    let text = msg.into_text().expect("text");
    let second: serde_json::Value = serde_json::from_str(&text).expect("json");
    assert_eq!(second["seq"].as_i64().unwrap_or_default(), 3);

    server.abort();
}
