use std::sync::Arc;

use serde::Deserialize;
use sqlx::SqlitePool;
use tokio::sync::{Notify, Semaphore};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::{agent_labels_repo, bulk_operations_repo, jobs_repo, secrets_repo};

use crate::agent_job_resolver;
use crate::agent_manager::AgentManager;
use crate::agent_snapshots::{SendConfigSnapshotOutcome, send_node_config_snapshot_with_outcome};

const BULK_CONCURRENCY: usize = 8;

pub struct BulkOperationsArgs {
    pub db: SqlitePool,
    pub secrets: Arc<SecretsCrypto>,
    pub agent_manager: AgentManager,
    pub notify: Arc<Notify>,
    pub shutdown: CancellationToken,
}

pub fn spawn(args: BulkOperationsArgs) {
    tokio::spawn(run_loop(args));
}

async fn run_loop(args: BulkOperationsArgs) {
    let db = args.db;
    let secrets = args.secrets;
    let agent_manager = args.agent_manager;
    let notify = args.notify;
    let shutdown = args.shutdown;

    let semaphore = Arc::new(Semaphore::new(BULK_CONCURRENCY));

    loop {
        if shutdown.is_cancelled() {
            break;
        }

        let available = semaphore.available_permits();
        if available > 0 {
            match bulk_operations_repo::claim_next_items(&db, available as i64).await {
                Ok(claimed) => {
                    if !claimed.is_empty() {
                        for item in claimed {
                            let permit = match semaphore.clone().acquire_owned().await {
                                Ok(p) => p,
                                Err(_) => break,
                            };
                            let db = db.clone();
                            let secrets = secrets.clone();
                            let agent_manager = agent_manager.clone();
                            tokio::spawn(async move {
                                let _permit = permit;
                                process_item(db, secrets, agent_manager, item).await;
                            });
                        }
                        continue;
                    }
                }
                Err(error) => {
                    warn!(error = %error, "bulk worker failed to claim items");
                }
            }
        }

        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = notify.notified() => {},
            _ = tokio::time::sleep(std::time::Duration::from_secs(2)) => {},
        }
    }
}

#[derive(Debug, Deserialize)]
struct AgentLabelsPayload {
    labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct WebdavDistributePayload {
    name: String,
    overwrite: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WebdavDistributeOutcome {
    Skipped,
    Updated,
}

#[derive(Debug, Deserialize)]
struct JobDeployPayload {
    source_job_id: String,
    name_template: String,
}

struct JobDeployFailure {
    kind: &'static str,
    message: String,
}

async fn distribute_webdav_secret_to_agent(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    agent_id: &str,
    name: &str,
    overwrite: bool,
) -> Result<WebdavDistributeOutcome, anyhow::Error> {
    let exists = secrets_repo::secret_exists(db, agent_id, "webdav", name).await?;
    if exists && !overwrite {
        return Ok(WebdavDistributeOutcome::Skipped);
    }

    let source = secrets_repo::get_secret_hub(db, secrets, "webdav", name)
        .await?
        .ok_or_else(|| anyhow::anyhow!("source secret not found"))?;

    secrets_repo::upsert_secret(db, secrets, agent_id, "webdav", name, &source).await?;

    let _ = send_node_config_snapshot_with_outcome(db, secrets, agent_manager, agent_id).await?;
    Ok(WebdavDistributeOutcome::Updated)
}

fn render_name_template(template: &str, source_name: &str, node_id: &str) -> String {
    template
        .replace("{name}", source_name)
        .replace("{node}", node_id)
        .trim()
        .to_string()
}

async fn disambiguate_job_name(
    db: &SqlitePool,
    agent_id: &str,
    candidate: &str,
) -> Result<String, anyhow::Error> {
    let existing = jobs_repo::list_jobs_for_agent(db, agent_id).await?;
    let names = existing
        .into_iter()
        .map(|j| j.name)
        .collect::<std::collections::HashSet<String>>();

    if !names.contains(candidate) {
        return Ok(candidate.to_string());
    }

    let mut i = 2;
    loop {
        let name = format!("{candidate} #{i}");
        if !names.contains(&name) {
            return Ok(name);
        }
        i += 1;
    }
}

async fn deploy_job_to_agent(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    target_agent_id: &str,
    source_job: &jobs_repo::Job,
    name_template: &str,
) -> Result<Option<String>, JobDeployFailure> {
    use bastion_core::job_spec;

    let planned_base = render_name_template(name_template, &source_job.name, target_agent_id);
    if planned_base.is_empty() {
        return Err(JobDeployFailure {
            kind: "invalid_payload",
            message: "name_template produced empty name".to_string(),
        });
    }
    let planned_name = disambiguate_job_name(db, target_agent_id, &planned_base)
        .await
        .map_err(|e| JobDeployFailure {
            kind: "internal_error",
            message: e.to_string(),
        })?;

    let spec = job_spec::parse_value(&source_job.spec).map_err(|e| JobDeployFailure {
        kind: "invalid_payload",
        message: e.to_string(),
    })?;
    job_spec::validate(&spec).map_err(|e| JobDeployFailure {
        kind: "invalid_payload",
        message: e.to_string(),
    })?;

    let _ = agent_job_resolver::resolve_job_spec_for_agent(db, secrets, target_agent_id, spec)
        .await
        .map_err(|e| JobDeployFailure {
            kind: "validation_failed",
            message: e.to_string(),
        })?;

    let _created = jobs_repo::create_job(
        db,
        &planned_name,
        Some(target_agent_id),
        source_job.schedule.as_deref(),
        Some(&source_job.schedule_timezone),
        source_job.overlap_policy,
        source_job.spec.clone(),
    )
    .await
    .map_err(|e| JobDeployFailure {
        kind: "create_failed",
        message: e.to_string(),
    })?;

    match send_node_config_snapshot_with_outcome(db, secrets, agent_manager, target_agent_id).await
    {
        Ok(_) => Ok(None),
        Err(error) => Ok(Some(error.to_string())),
    }
}

async fn process_item(
    db: SqlitePool,
    secrets: Arc<SecretsCrypto>,
    agent_manager: AgentManager,
    item: bulk_operations_repo::ClaimedBulkOperationItem,
) {
    debug!(
        op_id = %item.op_id,
        agent_id = %item.agent_id,
        kind = %item.kind,
        "processing bulk operation item"
    );

    match item.kind.as_str() {
        "agent_labels_add" | "agent_labels_remove" => {
            let payload: AgentLabelsPayload = match serde_json::from_str(&item.payload_json) {
                Ok(v) => v,
                Err(error) => {
                    let _ = bulk_operations_repo::mark_item_failed(
                        &db,
                        &item.op_id,
                        &item.agent_id,
                        "invalid_payload",
                        &error.to_string(),
                    )
                    .await;
                    return;
                }
            };

            if payload.labels.is_empty() {
                let _ = bulk_operations_repo::mark_item_failed(
                    &db,
                    &item.op_id,
                    &item.agent_id,
                    "invalid_payload",
                    "labels is required",
                )
                .await;
                return;
            }

            let result = if item.kind == "agent_labels_add" {
                agent_labels_repo::add_labels(&db, &item.agent_id, &payload.labels).await
            } else {
                agent_labels_repo::remove_labels(&db, &item.agent_id, &payload.labels).await
            };

            match result {
                Ok(()) => {
                    let _ =
                        bulk_operations_repo::mark_item_succeeded(&db, &item.op_id, &item.agent_id)
                            .await;
                }
                Err(error) => {
                    let _ = bulk_operations_repo::mark_item_failed(
                        &db,
                        &item.op_id,
                        &item.agent_id,
                        "internal_error",
                        &error.to_string(),
                    )
                    .await;
                }
            }
        }
        "sync_config_now" => {
            match send_node_config_snapshot_with_outcome(
                &db,
                secrets.as_ref(),
                &agent_manager,
                &item.agent_id,
            )
            .await
            {
                Ok(
                    SendConfigSnapshotOutcome::Sent
                    | SendConfigSnapshotOutcome::Unchanged
                    | SendConfigSnapshotOutcome::PendingOffline,
                ) => {
                    let _ =
                        bulk_operations_repo::mark_item_succeeded(&db, &item.op_id, &item.agent_id)
                            .await;
                }
                Err(error) => {
                    let _ = bulk_operations_repo::mark_item_failed(
                        &db,
                        &item.op_id,
                        &item.agent_id,
                        "send_failed",
                        &error.to_string(),
                    )
                    .await;
                }
            }
        }
        "webdav_secret_distribute" => {
            let payload: WebdavDistributePayload = match serde_json::from_str(&item.payload_json) {
                Ok(v) => v,
                Err(error) => {
                    let _ = bulk_operations_repo::mark_item_failed(
                        &db,
                        &item.op_id,
                        &item.agent_id,
                        "invalid_payload",
                        &error.to_string(),
                    )
                    .await;
                    return;
                }
            };

            let name = payload.name.trim();
            if name.is_empty() {
                let _ = bulk_operations_repo::mark_item_failed(
                    &db,
                    &item.op_id,
                    &item.agent_id,
                    "invalid_payload",
                    "name is required",
                )
                .await;
                return;
            }

            match distribute_webdav_secret_to_agent(
                &db,
                secrets.as_ref(),
                &agent_manager,
                &item.agent_id,
                name,
                payload.overwrite,
            )
            .await
            {
                Ok(WebdavDistributeOutcome::Skipped) => {
                    let _ = bulk_operations_repo::mark_item_succeeded_with_note(
                        &db,
                        &item.op_id,
                        &item.agent_id,
                        "skipped",
                        "already exists",
                    )
                    .await;
                }
                Ok(WebdavDistributeOutcome::Updated) => {
                    let _ =
                        bulk_operations_repo::mark_item_succeeded(&db, &item.op_id, &item.agent_id)
                            .await;
                }
                Err(error) => {
                    let kind = if error.to_string().contains("source secret not found") {
                        "source_missing"
                    } else {
                        "internal_error"
                    };
                    let _ = bulk_operations_repo::mark_item_failed(
                        &db,
                        &item.op_id,
                        &item.agent_id,
                        kind,
                        &error.to_string(),
                    )
                    .await;
                }
            }
        }
        "job_deploy" => {
            let payload: JobDeployPayload = match serde_json::from_str(&item.payload_json) {
                Ok(v) => v,
                Err(error) => {
                    let _ = bulk_operations_repo::mark_item_failed(
                        &db,
                        &item.op_id,
                        &item.agent_id,
                        "invalid_payload",
                        &error.to_string(),
                    )
                    .await;
                    return;
                }
            };

            let source_job_id = payload.source_job_id.trim();
            if source_job_id.is_empty() {
                let _ = bulk_operations_repo::mark_item_failed(
                    &db,
                    &item.op_id,
                    &item.agent_id,
                    "invalid_payload",
                    "source_job_id is required",
                )
                .await;
                return;
            }

            let source_job = match jobs_repo::get_job(&db, source_job_id).await {
                Ok(Some(job)) => job,
                Ok(None) => {
                    let _ = bulk_operations_repo::mark_item_failed(
                        &db,
                        &item.op_id,
                        &item.agent_id,
                        "source_not_found",
                        "source job not found",
                    )
                    .await;
                    return;
                }
                Err(error) => {
                    let _ = bulk_operations_repo::mark_item_failed(
                        &db,
                        &item.op_id,
                        &item.agent_id,
                        "internal_error",
                        &error.to_string(),
                    )
                    .await;
                    return;
                }
            };

            let template = payload.name_template.trim();
            if template.is_empty() {
                let _ = bulk_operations_repo::mark_item_failed(
                    &db,
                    &item.op_id,
                    &item.agent_id,
                    "invalid_payload",
                    "name_template is required",
                )
                .await;
                return;
            }

            match deploy_job_to_agent(
                &db,
                secrets.as_ref(),
                &agent_manager,
                &item.agent_id,
                &source_job,
                template,
            )
            .await
            {
                Ok(Some(note)) => {
                    let _ = bulk_operations_repo::mark_item_succeeded_with_note(
                        &db,
                        &item.op_id,
                        &item.agent_id,
                        "config_send_failed",
                        &note,
                    )
                    .await;
                }
                Ok(None) => {
                    let _ =
                        bulk_operations_repo::mark_item_succeeded(&db, &item.op_id, &item.agent_id)
                            .await;
                }
                Err(failure) => {
                    let _ = bulk_operations_repo::mark_item_failed(
                        &db,
                        &item.op_id,
                        &item.agent_id,
                        failure.kind,
                        &failure.message,
                    )
                    .await;
                }
            }
        }
        _ => {
            let _ = bulk_operations_repo::mark_item_failed(
                &db,
                &item.op_id,
                &item.agent_id,
                "unknown_kind",
                "unknown bulk operation kind",
            )
            .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sqlx::Row;
    use tempfile::TempDir;

    use bastion_core::agent;
    use bastion_core::job_spec;
    use bastion_storage::auth;
    use bastion_storage::db;
    use bastion_storage::secrets::SecretsCrypto;
    use bastion_storage::{bulk_operations_repo, jobs_repo, secrets_repo};

    use super::{WebdavDistributeOutcome, distribute_webdav_secret_to_agent};
    use crate::agent_manager::AgentManager;

    async fn insert_agent(pool: &sqlx::SqlitePool, agent_id: &str) {
        let key = agent::generate_token_b64_urlsafe(32);
        let key_hash = agent::sha256_urlsafe_token(&key).expect("hash");
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        sqlx::query("INSERT INTO agents (id, name, key_hash, created_at) VALUES (?, NULL, ?, ?)")
            .bind(agent_id)
            .bind(key_hash)
            .bind(now)
            .execute(pool)
            .await
            .expect("insert agent");
    }

    async fn create_user_id(pool: &sqlx::SqlitePool) -> i64 {
        auth::create_user(pool, "admin", "pw")
            .await
            .expect("create user");
        let user = auth::find_user_by_username(pool, "admin")
            .await
            .expect("find user")
            .expect("user");
        user.id
    }

    fn example_webdav_job_spec(secret_name: &str) -> serde_json::Value {
        let spec = job_spec::JobSpecV1::Filesystem {
            v: 1,
            pipeline: Default::default(),
            notifications: Default::default(),
            retention: Default::default(),
            source: job_spec::FilesystemSource {
                pre_scan: true,
                paths: vec![],
                root: "/".to_string(),
                include: vec![],
                exclude: vec![],
                symlink_policy: Default::default(),
                hardlink_policy: Default::default(),
                error_policy: Default::default(),
            },
            target: job_spec::TargetV1::Webdav {
                base_url: "http://example.com/backup".to_string(),
                secret_name: secret_name.to_string(),
                part_size_bytes: 1024 * 1024,
            },
        };
        serde_json::to_value(spec).expect("spec to value")
    }

    async fn insert_webdav_secret(
        pool: &sqlx::SqlitePool,
        crypto: &SecretsCrypto,
        agent_id: &str,
        name: &str,
    ) {
        secrets_repo::upsert_secret(
            pool,
            crypto,
            agent_id,
            "webdav",
            name,
            &serde_json::to_vec(&serde_json::json!({"username":"u","password":"p"})).expect("json"),
        )
        .await
        .expect("upsert secret");
    }

    #[tokio::test]
    async fn distribute_skips_existing_by_default() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");
        let crypto = SecretsCrypto::load_or_create(temp.path()).expect("crypto");
        let agent_manager = AgentManager::default();

        insert_agent(&pool, "agent1").await;

        bastion_storage::secrets_repo::upsert_secret(
            &pool, &crypto, "hub", "webdav", "primary", b"new",
        )
        .await
        .expect("hub secret");
        bastion_storage::secrets_repo::upsert_secret(
            &pool, &crypto, "agent1", "webdav", "primary", b"old",
        )
        .await
        .expect("agent secret");

        let out = distribute_webdav_secret_to_agent(
            &pool,
            &crypto,
            &agent_manager,
            "agent1",
            "primary",
            false,
        )
        .await
        .expect("distribute");
        assert_eq!(out, WebdavDistributeOutcome::Skipped);

        let v = bastion_storage::secrets_repo::get_secret(
            &pool, &crypto, "agent1", "webdav", "primary",
        )
        .await
        .expect("get")
        .expect("present");
        assert_eq!(v, b"old");
    }

    #[tokio::test]
    async fn distribute_overwrite_replaces_secret_and_marks_pending_offline() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");
        let crypto = SecretsCrypto::load_or_create(temp.path()).expect("crypto");
        let agent_manager = AgentManager::default();

        insert_agent(&pool, "agent1").await;

        bastion_storage::secrets_repo::upsert_secret(
            &pool, &crypto, "hub", "webdav", "primary", b"new",
        )
        .await
        .expect("hub secret");
        bastion_storage::secrets_repo::upsert_secret(
            &pool, &crypto, "agent1", "webdav", "primary", b"old",
        )
        .await
        .expect("agent secret");

        let out = distribute_webdav_secret_to_agent(
            &pool,
            &crypto,
            &agent_manager,
            "agent1",
            "primary",
            true,
        )
        .await
        .expect("distribute");
        assert_eq!(out, WebdavDistributeOutcome::Updated);

        let v = bastion_storage::secrets_repo::get_secret(
            &pool, &crypto, "agent1", "webdav", "primary",
        )
        .await
        .expect("get")
        .expect("present");
        assert_eq!(v, b"new");

        let row = sqlx::query(
            "SELECT desired_config_snapshot_id, last_config_sync_error_kind FROM agents WHERE id = ? LIMIT 1",
        )
        .bind("agent1")
        .fetch_one(&pool)
        .await
        .expect("row");
        assert!(
            row.get::<Option<String>, _>("desired_config_snapshot_id")
                .is_some()
        );
        assert!(
            row.get::<Option<String>, _>("last_config_sync_error_kind")
                .is_none()
        );
    }

    #[tokio::test]
    async fn job_deploy_creates_jobs_only_for_valid_agents() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");
        let crypto = SecretsCrypto::load_or_create(temp.path()).expect("crypto");
        let secrets = Arc::new(crypto);
        let agent_manager = AgentManager::default();
        let user_id = create_user_id(&pool).await;

        insert_agent(&pool, "agent_ok").await;
        insert_agent(&pool, "agent_bad").await;

        insert_webdav_secret(&pool, secrets.as_ref(), "agent_ok", "primary").await;

        let source_job = jobs_repo::create_job(
            &pool,
            "Backup",
            None,
            Some("0 0 * * *"),
            Some("UTC"),
            jobs_repo::OverlapPolicy::Reject,
            example_webdav_job_spec("primary"),
        )
        .await
        .expect("create source job");

        let op_id = bulk_operations_repo::create_operation(
            &pool,
            user_id,
            "job_deploy",
            &serde_json::json!({"node_ids":["agent_ok","agent_bad"]}),
            &serde_json::json!({
                "source_job_id": source_job.id,
                "name_template": "{name} ({node})"
            }),
            &["agent_ok".to_string(), "agent_bad".to_string()],
        )
        .await
        .expect("create op");

        let claimed = bulk_operations_repo::claim_next_items(&pool, 10)
            .await
            .expect("claim");
        assert_eq!(claimed.len(), 2);

        for item in claimed {
            super::process_item(pool.clone(), secrets.clone(), agent_manager.clone(), item).await;
        }

        let ok_jobs = jobs_repo::list_jobs_for_agent(&pool, "agent_ok")
            .await
            .expect("list ok jobs");
        assert_eq!(ok_jobs.len(), 1);
        assert_eq!(ok_jobs[0].name, "Backup (agent_ok)");

        let bad_jobs = jobs_repo::list_jobs_for_agent(&pool, "agent_bad")
            .await
            .expect("list bad jobs");
        assert_eq!(bad_jobs.len(), 0);

        let op = bulk_operations_repo::get_operation(&pool, &op_id)
            .await
            .expect("get op")
            .expect("op exists");
        let ok_item = op
            .items
            .iter()
            .find(|it| it.agent_id == "agent_ok")
            .expect("ok item");
        assert_eq!(ok_item.status, "success");

        let bad_item = op
            .items
            .iter()
            .find(|it| it.agent_id == "agent_bad")
            .expect("bad item");
        assert_eq!(bad_item.status, "failed");
        assert_eq!(
            bad_item.last_error_kind.as_deref(),
            Some("validation_failed")
        );
        assert!(
            bad_item
                .last_error
                .as_deref()
                .unwrap_or_default()
                .contains("missing webdav secret: primary")
        );
    }

    #[tokio::test]
    async fn job_deploy_disambiguates_name_collisions() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");
        let crypto = SecretsCrypto::load_or_create(temp.path()).expect("crypto");
        let secrets = Arc::new(crypto);
        let agent_manager = AgentManager::default();
        let user_id = create_user_id(&pool).await;

        insert_agent(&pool, "agent1").await;
        insert_webdav_secret(&pool, secrets.as_ref(), "agent1", "primary").await;

        let source_job = jobs_repo::create_job(
            &pool,
            "Backup",
            None,
            Some("0 0 * * *"),
            Some("UTC"),
            jobs_repo::OverlapPolicy::Reject,
            example_webdav_job_spec("primary"),
        )
        .await
        .expect("create source job");

        jobs_repo::create_job(
            &pool,
            "Backup (agent1)",
            Some("agent1"),
            Some("0 0 * * *"),
            Some("UTC"),
            jobs_repo::OverlapPolicy::Reject,
            example_webdav_job_spec("primary"),
        )
        .await
        .expect("create existing job");

        let _op_id = bulk_operations_repo::create_operation(
            &pool,
            user_id,
            "job_deploy",
            &serde_json::json!({"node_ids":["agent1"]}),
            &serde_json::json!({
                "source_job_id": source_job.id,
                "name_template": "{name} ({node})"
            }),
            &["agent1".to_string()],
        )
        .await
        .expect("create op");

        let claimed = bulk_operations_repo::claim_next_items(&pool, 10)
            .await
            .expect("claim");
        assert_eq!(claimed.len(), 1);

        super::process_item(
            pool.clone(),
            secrets.clone(),
            agent_manager.clone(),
            claimed.into_iter().next().expect("item"),
        )
        .await;

        let jobs = jobs_repo::list_jobs_for_agent(&pool, "agent1")
            .await
            .expect("list");
        assert_eq!(jobs.len(), 2);
        assert!(jobs.iter().any(|j| j.name == "Backup (agent1) #2"));
    }
}
