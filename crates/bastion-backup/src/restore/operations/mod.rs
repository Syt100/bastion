use std::path::PathBuf;

use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;
use tracing::warn;

use bastion_storage::operations_repo;
use bastion_storage::secrets::SecretsCrypto;

use super::{ConflictPolicy, RestoreDestination, RestoreSelection};

mod progress;
mod restore;
mod util;
mod verify;

#[derive(Debug)]
pub(super) struct OperationCanceled {
    pub(super) op_id: String,
}

impl std::fmt::Display for OperationCanceled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation canceled: {}", self.op_id)
    }
}

impl std::error::Error for OperationCanceled {}

pub(super) fn check_operation_canceled(
    op_id: &str,
    cancel_token: &CancellationToken,
) -> Result<(), anyhow::Error> {
    if cancel_token.is_cancelled() {
        return Err(anyhow::Error::new(OperationCanceled {
            op_id: op_id.to_string(),
        }));
    }
    Ok(())
}

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

async fn cancel_operation(db: &SqlitePool, data_dir: &std::path::Path, op_id: &str) {
    let _ = operations_repo::append_event(db, op_id, "info", "canceled", "canceled", None).await;
    let _ = operations_repo::complete_operation(
        db,
        op_id,
        operations_repo::OperationStatus::Canceled,
        None,
        Some("canceled"),
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
    destination: RestoreDestination,
    conflict: ConflictPolicy,
    selection: Option<RestoreSelection>,
    cancel_token: CancellationToken,
    on_finish: Option<Box<dyn FnOnce() + Send + 'static>>,
) {
    tokio::spawn(async move {
        struct FinishGuard(Option<Box<dyn FnOnce() + Send + 'static>>);
        impl Drop for FinishGuard {
            fn drop(&mut self) {
                if let Some(cb) = self.0.take() {
                    cb();
                }
            }
        }
        let _finish_guard = FinishGuard(on_finish);

        if let Err(error) = restore::restore_operation(
            &db,
            &secrets,
            &data_dir,
            &op_id,
            &run_id,
            &destination,
            conflict,
            selection,
            &cancel_token,
        )
        .await
        {
            if error.downcast_ref::<OperationCanceled>().is_some() {
                cancel_operation(&db, &data_dir, &op_id).await;
                return;
            }
            warn!(
                op_id = %op_id,
                run_id = %run_id,
                destination = ?destination,
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
    cancel_token: CancellationToken,
    on_finish: Option<Box<dyn FnOnce() + Send + 'static>>,
) {
    tokio::spawn(async move {
        struct FinishGuard(Option<Box<dyn FnOnce() + Send + 'static>>);
        impl Drop for FinishGuard {
            fn drop(&mut self) {
                if let Some(cb) = self.0.take() {
                    cb();
                }
            }
        }
        let _finish_guard = FinishGuard(on_finish);

        if let Err(error) =
            verify::verify_operation(&db, &secrets, &data_dir, &op_id, &run_id, &cancel_token).await
        {
            if error.downcast_ref::<OperationCanceled>().is_some() {
                cancel_operation(&db, &data_dir, &op_id).await;
                return;
            }
            warn!(op_id = %op_id, run_id = %run_id, error = %error, "verify operation failed");
            let msg = format!("{error:#}");
            fail_operation(&db, &data_dir, &op_id, &msg).await;
        }
    });
}
