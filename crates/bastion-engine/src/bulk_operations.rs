use std::sync::Arc;

use serde::Deserialize;
use sqlx::SqlitePool;
use tokio::sync::{Notify, Semaphore};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

use bastion_storage::{agent_labels_repo, bulk_operations_repo};

const BULK_CONCURRENCY: usize = 8;

#[derive(Debug)]
pub struct BulkOperationsArgs {
    pub db: SqlitePool,
    pub notify: Arc<Notify>,
    pub shutdown: CancellationToken,
}

pub fn spawn(args: BulkOperationsArgs) {
    tokio::spawn(run_loop(args));
}

async fn run_loop(args: BulkOperationsArgs) {
    let db = args.db;
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
                            tokio::spawn(async move {
                                let _permit = permit;
                                process_item(&db, item).await;
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

async fn process_item(db: &SqlitePool, item: bulk_operations_repo::ClaimedBulkOperationItem) {
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
                        db,
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
                    db,
                    &item.op_id,
                    &item.agent_id,
                    "invalid_payload",
                    "labels is required",
                )
                .await;
                return;
            }

            let result = if item.kind == "agent_labels_add" {
                agent_labels_repo::add_labels(db, &item.agent_id, &payload.labels).await
            } else {
                agent_labels_repo::remove_labels(db, &item.agent_id, &payload.labels).await
            };

            match result {
                Ok(()) => {
                    let _ =
                        bulk_operations_repo::mark_item_succeeded(db, &item.op_id, &item.agent_id)
                            .await;
                }
                Err(error) => {
                    let _ = bulk_operations_repo::mark_item_failed(
                        db,
                        &item.op_id,
                        &item.agent_id,
                        "internal_error",
                        &error.to_string(),
                    )
                    .await;
                }
            }
        }
        _ => {
            let _ = bulk_operations_repo::mark_item_failed(
                db,
                &item.op_id,
                &item.agent_id,
                "unknown_kind",
                "unknown bulk operation kind",
            )
            .await;
        }
    }
}
