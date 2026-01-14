use std::sync::Arc;

use serde::Deserialize;
use sqlx::SqlitePool;
use tokio::sync::{Notify, Semaphore};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::{agent_labels_repo, bulk_operations_repo, secrets_repo};

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
    use sqlx::Row;
    use tempfile::TempDir;

    use bastion_core::agent;
    use bastion_storage::db;
    use bastion_storage::secrets::SecretsCrypto;

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
}
