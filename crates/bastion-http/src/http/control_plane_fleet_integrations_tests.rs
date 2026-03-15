use std::sync::Arc;

use axum::http::StatusCode;
use tempfile::TempDir;

use bastion_config::Config;
use bastion_engine::agent_manager::AgentManager;
use bastion_engine::run_events_bus::RunEventsBus;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::{auth, db, hub_runtime_config_repo, secrets_repo};

use super::{AppState, ConfigValueSource, HubRuntimeConfigMeta, HubRuntimeConfigSources};

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
    hub_runtime_config: HubRuntimeConfigMeta,
) -> AppState {
    AppState {
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
        run_events_bus: Arc::new(RunEventsBus::new()),
        hub_runtime_config,
    }
}

async fn insert_agent(pool: &sqlx::SqlitePool, agent: InsertAgent<'_>) {
    let key = bastion_core::agent::generate_token_b64_urlsafe(32);
    let key_hash = bastion_core::agent::sha256_urlsafe_token(&key).expect("hash");
    sqlx::query(
        r#"
        INSERT INTO agents (
          id, name, key_hash, created_at, revoked_at, last_seen_at,
          desired_config_snapshot_id, desired_config_snapshot_at,
          applied_config_snapshot_id, applied_config_snapshot_at,
          last_config_sync_attempt_at, last_config_sync_error_kind, last_config_sync_error, last_config_sync_error_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(agent.agent_id)
    .bind(agent.name)
    .bind(key_hash)
    .bind(10i64)
    .bind(agent.revoked_at)
    .bind(agent.last_seen_at)
    .bind(agent.desired_snapshot_id)
    .bind(10i64)
    .bind(agent.applied_snapshot_id)
    .bind(10i64)
    .bind(10i64)
    .bind(agent.last_config_sync_error_kind)
    .bind(
        agent
            .last_config_sync_error_kind
            .map(|kind| format!("{kind} details")),
    )
    .bind(agent.last_config_sync_error_kind.map(|_| 10i64))
    .execute(pool)
    .await
    .expect("insert agent");
}

struct InsertAgent<'a> {
    agent_id: &'a str,
    name: &'a str,
    last_seen_at: Option<i64>,
    revoked_at: Option<i64>,
    desired_snapshot_id: Option<&'a str>,
    applied_snapshot_id: Option<&'a str>,
    last_config_sync_error_kind: Option<&'a str>,
}

async fn insert_job(
    pool: &sqlx::SqlitePool,
    job_id: &str,
    agent_id: Option<&str>,
    name: &str,
    spec_json: &str,
) {
    sqlx::query(
        "INSERT INTO jobs (id, name, agent_id, schedule, overlap_policy, spec_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(job_id)
    .bind(name)
    .bind(agent_id)
    .bind("0 * * * *")
    .bind("queue")
    .bind(spec_json)
    .bind(11i64)
    .bind(12i64)
    .execute(pool)
    .await
    .expect("insert job");
}

async fn insert_run(pool: &sqlx::SqlitePool, run_id: &str, job_id: &str, status: &str) {
    sqlx::query(
        "INSERT INTO runs (id, job_id, status, started_at, ended_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(job_id)
    .bind(status)
    .bind(20i64)
    .bind(21i64)
    .execute(pool)
    .await
    .expect("insert run");
}

#[tokio::test]
async fn public_metadata_requires_auth() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));
    let app = super::router(app_state(config, pool, secrets, Default::default()));

    let (listener, addr) = start_test_server().await;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve");
    });

    let resp = reqwest::Client::new()
        .get(format!(
            "{}/api/control-plane/public-metadata",
            base_url(addr)
        ))
        .send()
        .await
        .expect("request");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    server.abort();
}

#[tokio::test]
async fn runtime_config_persists_normalized_public_base_url_and_public_metadata_exposes_effective_value()
 {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let session = seed_admin_session(&pool).await;
    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));

    let app = super::router(app_state(
        config,
        pool.clone(),
        secrets,
        HubRuntimeConfigMeta {
            sources: HubRuntimeConfigSources {
                public_base_url: ConfigValueSource::Env,
                ..HubRuntimeConfigSources::default()
            },
            public_base_url: Some("https://ops.example.com/control".to_string()),
            ..HubRuntimeConfigMeta::default()
        },
    ));

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
    let save_resp = client
        .put(format!(
            "{}/api/settings/hub-runtime-config",
            base_url(addr)
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .header("x-csrf-token", session.csrf_token.clone())
        .json(&serde_json::json!({
            "public_base_url": " https://backup.example.com/bastion/ ",
            "default_backup_retention": {
                "enabled": false,
                "keep_last": null,
                "keep_days": null,
                "max_delete_per_tick": 50,
                "max_delete_per_day": 200
            }
        }))
        .send()
        .await
        .expect("save request");
    assert_eq!(save_resp.status(), StatusCode::NO_CONTENT);

    let saved = hub_runtime_config_repo::get(&pool)
        .await
        .expect("load saved")
        .expect("saved exists");
    assert_eq!(
        saved.public_base_url.as_deref(),
        Some("https://backup.example.com/bastion")
    );

    let get_resp = client
        .get(format!(
            "{}/api/settings/hub-runtime-config",
            base_url(addr)
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("get request");
    assert_eq!(get_resp.status(), StatusCode::OK);
    let body = get_resp.json::<serde_json::Value>().await.expect("json");
    assert_eq!(
        body["saved"]["public_base_url"].as_str(),
        Some("https://backup.example.com/bastion")
    );
    assert_eq!(
        body["effective"]["public_base_url"].as_str(),
        Some("https://ops.example.com/control")
    );
    assert_eq!(
        body["fields"]["public_base_url"]["source"].as_str(),
        Some("env")
    );

    let public_resp = client
        .get(format!(
            "{}/api/control-plane/public-metadata",
            base_url(addr)
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("public metadata request");
    assert_eq!(public_resp.status(), StatusCode::OK);
    let public_body = public_resp.json::<serde_json::Value>().await.expect("json");
    assert_eq!(
        public_body["public_base_url"].as_str(),
        Some("https://ops.example.com/control")
    );
    assert_eq!(
        public_body["command_generation_ready"].as_bool(),
        Some(true)
    );

    server.abort();
}

#[tokio::test]
async fn fleet_endpoints_return_summary_and_agent_detail() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let session = seed_admin_session(&pool).await;
    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    insert_agent(
        &pool,
        InsertAgent {
            agent_id: "edge-a",
            name: "DB Node A",
            last_seen_at: Some(now),
            revoked_at: None,
            desired_snapshot_id: Some("cfg-1"),
            applied_snapshot_id: Some("cfg-1"),
            last_config_sync_error_kind: None,
        },
    )
    .await;
    insert_agent(
        &pool,
        InsertAgent {
            agent_id: "edge-b",
            name: "DB Node B",
            last_seen_at: Some(now),
            revoked_at: None,
            desired_snapshot_id: Some("cfg-2"),
            applied_snapshot_id: Some("cfg-1"),
            last_config_sync_error_kind: Some("send_failed"),
        },
    )
    .await;
    insert_agent(
        &pool,
        InsertAgent {
            agent_id: "edge-c",
            name: "DB Node C",
            last_seen_at: None,
            revoked_at: Some(30),
            desired_snapshot_id: None,
            applied_snapshot_id: None,
            last_config_sync_error_kind: None,
        },
    )
    .await;

    sqlx::query(
        "INSERT INTO agent_labels (agent_id, label, created_at, updated_at) VALUES ('edge-a', 'db', 10, 10), ('edge-b', 'edge', 10, 10)",
    )
        .execute(&pool)
        .await
        .expect("labels");

    insert_job(
        &pool,
        "job-a",
        Some("edge-a"),
        "Nightly MySQL",
        r#"{"v":1,"type":"filesystem","source":{"root":"/var/lib/mysql"},"target":{"type":"local_dir","base_dir":"/tmp"}}"#,
    )
    .await;
    insert_job(
        &pool,
        "job-b",
        Some("edge-b"),
        "Warehouse Upload",
        r#"{"v":1,"type":"filesystem","source":{"root":"/warehouse"},"target":{"type":"webdav","base_url":"https://dav.example.com","secret_name":"missing-secret"}}"#,
    )
    .await;
    insert_run(&pool, "run-b1", "job-b", "failed").await;
    sqlx::query(
        "INSERT INTO agent_tasks (id, agent_id, run_id, status, payload_json, created_at, updated_at) VALUES (?, ?, ?, 'queued', '{}', ?, ?)",
    )
    .bind("task-b1")
    .bind("edge-b")
    .bind("run-b1")
    .bind(30i64)
    .bind(30i64)
    .execute(&pool)
    .await
    .expect("task");

    let app = super::router(app_state(
        config,
        pool.clone(),
        secrets,
        HubRuntimeConfigMeta {
            public_base_url: Some("https://backup.example.com".to_string()),
            sources: HubRuntimeConfigSources {
                public_base_url: ConfigValueSource::Db,
                ..HubRuntimeConfigSources::default()
            },
            ..HubRuntimeConfigMeta::default()
        },
    ));

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
    let list_resp = client
        .get(format!(
            "{}/api/fleet?status=online&q=edge-b&page=1&page_size=1",
            base_url(addr)
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("fleet list");
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_body = list_resp.json::<serde_json::Value>().await.expect("json");
    assert_eq!(list_body["summary"]["total"].as_i64(), Some(1));
    assert_eq!(list_body["summary"]["online"].as_i64(), Some(1));
    assert_eq!(list_body["summary"]["drifted"].as_i64(), Some(1));
    assert_eq!(list_body["page"].as_i64(), Some(1));
    assert_eq!(list_body["page_size"].as_i64(), Some(1));
    assert_eq!(list_body["total"].as_i64(), Some(1));
    assert_eq!(
        list_body["onboarding"]["public_base_url"].as_str(),
        Some("https://backup.example.com")
    );
    assert_eq!(
        list_body["items"].as_array().map(|items| items.len()),
        Some(1)
    );
    assert_eq!(
        list_body["items"][0]["assigned_jobs_total"].as_i64(),
        Some(1)
    );
    assert_eq!(
        list_body["items"][0]["pending_tasks_total"].as_i64(),
        Some(1)
    );

    let detail_resp = client
        .get(format!("{}/api/fleet/edge-b", base_url(addr)))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("fleet detail");
    assert_eq!(detail_resp.status(), StatusCode::OK);
    let detail_body = detail_resp.json::<serde_json::Value>().await.expect("json");
    assert_eq!(detail_body["agent"]["id"].as_str(), Some("edge-b"));
    assert_eq!(detail_body["sync"]["state"].as_str(), Some("error"));
    assert_eq!(
        detail_body["sync"]["last_error_kind"].as_str(),
        Some("send_failed")
    );
    assert_eq!(
        detail_body["related_jobs"]
            .as_array()
            .map(|items| items.len()),
        Some(1)
    );
    assert_eq!(
        detail_body["recent_activity"]
            .as_array()
            .map(|items| items.len()),
        Some(1)
    );
    assert_eq!(
        detail_body["capabilities"]["can_rotate_key"].as_bool(),
        Some(true)
    );

    server.abort();
}

#[tokio::test]
async fn integrations_summary_reports_empty_and_degraded_domains() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let session = seed_admin_session(&pool).await;
    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));

    let empty_app = super::router(app_state(
        config.clone(),
        pool.clone(),
        secrets.clone(),
        Default::default(),
    ));
    let (listener, addr) = start_test_server().await;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            empty_app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve");
    });

    let client = reqwest::Client::new();
    let empty_resp = client
        .get(format!("{}/api/integrations", base_url(addr)))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("empty integrations");
    assert_eq!(empty_resp.status(), StatusCode::OK);
    let empty_body = empty_resp.json::<serde_json::Value>().await.expect("json");
    assert_eq!(empty_body["storage"]["state"].as_str(), Some("empty"));
    assert_eq!(empty_body["notifications"]["state"].as_str(), Some("empty"));
    assert_eq!(empty_body["distribution"]["state"].as_str(), Some("empty"));

    server.abort();

    insert_agent(
        &pool,
        InsertAgent {
            agent_id: "edge-z",
            name: "Edge Z",
            last_seen_at: Some(time::OffsetDateTime::now_utc().unix_timestamp()),
            revoked_at: None,
            desired_snapshot_id: Some("cfg-9"),
            applied_snapshot_id: Some("cfg-1"),
            last_config_sync_error_kind: Some("send_failed"),
        },
    )
    .await;
    insert_job(
        &pool,
        "job-webdav",
        Some("edge-z"),
        "WebDAV Backup",
        r#"{"v":1,"type":"filesystem","source":{"root":"/data"},"target":{"type":"webdav","base_url":"https://dav.example.com","secret_name":"missing-dav"}}"#,
    )
    .await;
    insert_run(&pool, "run-webdav", "job-webdav", "failed").await;
    sqlx::query(
        "INSERT INTO notifications (id, run_id, channel, secret_name, status, attempts, next_attempt_at, created_at, updated_at, last_error) VALUES (?, ?, 'email', 'smtp-missing', 'failed', 1, ?, ?, ?, ?)",
    )
    .bind("notif-1")
    .bind("run-webdav")
    .bind(40i64)
    .bind(40i64)
    .bind(41i64)
    .bind("smtp failed")
    .execute(&pool)
    .await
    .expect("notification");

    // Add at least one notification destination so the domain is no longer empty.
    secrets_repo::upsert_secret(&pool, &secrets, "hub", "smtp", "smtp-a", br#"{}"#)
        .await
        .expect("smtp secret");

    let degraded_app = super::router(app_state(config, pool.clone(), secrets, Default::default()));
    let (listener, addr) = start_test_server().await;
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            degraded_app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve");
    });

    let degraded_resp = client
        .get(format!("{}/api/integrations", base_url(addr)))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("degraded integrations");
    assert_eq!(degraded_resp.status(), StatusCode::OK);
    let degraded_body = degraded_resp
        .json::<serde_json::Value>()
        .await
        .expect("json");
    assert_eq!(degraded_body["storage"]["state"].as_str(), Some("degraded"));
    assert_eq!(
        degraded_body["storage"]["summary"]["invalid_total"].as_i64(),
        Some(1)
    );
    assert_eq!(
        degraded_body["notifications"]["state"].as_str(),
        Some("degraded")
    );
    assert_eq!(
        degraded_body["notifications"]["summary"]["recent_failures_total"].as_i64(),
        Some(1)
    );
    assert_eq!(
        degraded_body["distribution"]["state"].as_str(),
        Some("degraded")
    );
    assert_eq!(
        degraded_body["distribution"]["summary"]["failed_total"].as_i64(),
        Some(1)
    );
    assert_eq!(
        degraded_body["distribution"]["summary"]["offline_total"].as_i64(),
        Some(0)
    );

    server.abort();
}

#[tokio::test]
async fn integrations_detail_endpoints_expose_storage_usage_and_distribution_scope_context() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");
    let session = seed_admin_session(&pool).await;
    let config = test_config(&temp);
    let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    insert_agent(
        &pool,
        InsertAgent {
            agent_id: "edge-a",
            name: "Edge A",
            last_seen_at: Some(now),
            revoked_at: None,
            desired_snapshot_id: Some("cfg-2"),
            applied_snapshot_id: Some("cfg-1"),
            last_config_sync_error_kind: Some("send_failed"),
        },
    )
    .await;
    insert_agent(
        &pool,
        InsertAgent {
            agent_id: "edge-b",
            name: "Edge B",
            last_seen_at: None,
            revoked_at: None,
            desired_snapshot_id: Some("cfg-3"),
            applied_snapshot_id: Some("cfg-2"),
            last_config_sync_error_kind: None,
        },
    )
    .await;

    secrets_repo::upsert_secret(&pool, &secrets, "edge-a", "webdav", "edge-dav", br#"{}"#)
        .await
        .expect("edge secret");

    insert_job(
        &pool,
        "job-edge-dav",
        Some("edge-a"),
        "Edge WebDAV",
        r#"{"v":1,"type":"filesystem","source":{"root":"/data"},"target":{"type":"webdav","base_url":"https://dav.example.com","secret_name":"edge-dav"}}"#,
    )
    .await;
    insert_job(
        &pool,
        "job-edge-missing",
        Some("edge-a"),
        "Edge Missing Credential",
        r#"{"v":1,"type":"filesystem","source":{"root":"/warehouse"},"target":{"type":"webdav","base_url":"https://dav.example.com","secret_name":"missing-edge-dav"}}"#,
    )
    .await;
    insert_run(&pool, "run-edge-dav", "job-edge-dav", "failed").await;
    sqlx::query(
        "INSERT INTO agent_tasks (id, agent_id, run_id, status, payload_json, created_at, updated_at) VALUES (?, ?, ?, 'queued', '{}', ?, ?)",
    )
    .bind("task-edge-a")
    .bind("edge-a")
    .bind("run-edge-dav")
    .bind(50i64)
    .bind(50i64)
    .execute(&pool)
    .await
    .expect("task");

    let app = super::router(app_state(config, pool.clone(), secrets, Default::default()));

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
    let storage_resp = client
        .get(format!(
            "{}/api/integrations/storage?node_id=edge-a",
            base_url(addr)
        ))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("storage details");
    assert_eq!(storage_resp.status(), StatusCode::OK);
    let storage_body = storage_resp
        .json::<serde_json::Value>()
        .await
        .expect("json");
    assert_eq!(storage_body["node_id"].as_str(), Some("edge-a"));
    assert_eq!(storage_body["summary"]["items_total"].as_i64(), Some(1));
    assert_eq!(storage_body["summary"]["invalid_total"].as_i64(), Some(1));
    assert_eq!(storage_body["items"][0]["name"].as_str(), Some("edge-dav"));
    assert_eq!(storage_body["items"][0]["usage_total"].as_i64(), Some(1));
    assert_eq!(
        storage_body["items"][0]["usage"][0]["job_name"].as_str(),
        Some("Edge WebDAV")
    );
    assert_eq!(
        storage_body["items"][0]["health"]["state"].as_str(),
        Some("attention")
    );

    let distribution_resp = client
        .get(format!("{}/api/integrations/distribution", base_url(addr)))
        .header("cookie", format!("bastion_session={}", session.id))
        .send()
        .await
        .expect("distribution details");
    assert_eq!(distribution_resp.status(), StatusCode::OK);
    let distribution_body = distribution_resp
        .json::<serde_json::Value>()
        .await
        .expect("json");
    assert_eq!(
        distribution_body["summary"]["coverage_total"].as_i64(),
        Some(2)
    );
    assert_eq!(
        distribution_body["summary"]["offline_total"].as_i64(),
        Some(1)
    );
    assert_eq!(
        distribution_body["items"][0]["distribution_state"].as_str(),
        Some("failed")
    );
    assert_eq!(
        distribution_body["items"][0]["pending_tasks_total"].as_i64(),
        Some(1)
    );

    server.abort();
}
