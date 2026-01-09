use std::path::PathBuf;

use sqlx::SqlitePool;
use tracing::warn;

use bastion_storage::operations_repo;
use bastion_storage::secrets::SecretsCrypto;

use super::{ConflictPolicy, RestoreSelection};

mod restore;
mod util;
mod verify;

async fn fail_operation(db: &SqlitePool, data_dir: &std::path::Path, op_id: &str, msg: &str) {
    let _ = operations_repo::append_event(db, op_id, "error", "failed", msg, None).await;
    let _ = operations_repo::complete_operation(
        db,
        op_id,
        operations_repo::OperationStatus::Failed,
        None,
        Some(msg),
    )
    .await;
    let _ = tokio::fs::remove_dir_all(util::operation_dir(data_dir, op_id)).await;
}

#[allow(clippy::too_many_arguments)]
pub async fn spawn_restore_operation(
    db: SqlitePool,
    secrets: std::sync::Arc<SecretsCrypto>,
    data_dir: PathBuf,
    op_id: String,
    run_id: String,
    destination_dir: PathBuf,
    conflict: ConflictPolicy,
    selection: Option<RestoreSelection>,
) {
    tokio::spawn(async move {
        if let Err(error) = restore::restore_operation(
            &db,
            &secrets,
            &data_dir,
            &op_id,
            &run_id,
            &destination_dir,
            conflict,
            selection,
        )
        .await
        {
            warn!(
                op_id = %op_id,
                run_id = %run_id,
                destination_dir = %destination_dir.display(),
                error = %error,
                "restore operation failed"
            );
            let msg = format!("{error:#}");
            fail_operation(&db, &data_dir, &op_id, &msg).await;
        }
    });
}

pub async fn spawn_verify_operation(
    db: SqlitePool,
    secrets: std::sync::Arc<SecretsCrypto>,
    data_dir: PathBuf,
    op_id: String,
    run_id: String,
) {
    tokio::spawn(async move {
        if let Err(error) =
            verify::verify_operation(&db, &secrets, &data_dir, &op_id, &run_id).await
        {
            warn!(op_id = %op_id, run_id = %run_id, error = %error, "verify operation failed");
            let msg = format!("{error:#}");
            fail_operation(&db, &data_dir, &op_id, &msg).await;
        }
    });
}
