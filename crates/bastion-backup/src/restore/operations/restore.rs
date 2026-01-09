use std::path::Path;

use tracing::info;

use sqlx::SqlitePool;

use bastion_storage::operations_repo;
use bastion_storage::secrets::SecretsCrypto;

use super::super::{ConflictPolicy, RestoreSelection, access, parts, unpack};

#[allow(clippy::too_many_arguments)]
pub(super) async fn restore_operation(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    data_dir: &Path,
    op_id: &str,
    run_id: &str,
    destination_dir: &Path,
    conflict: ConflictPolicy,
    selection: Option<RestoreSelection>,
) -> Result<(), anyhow::Error> {
    info!(
        op_id = %op_id,
        run_id = %run_id,
        destination_dir = %destination_dir.display(),
        conflict = %conflict.as_str(),
        selection_files = selection.as_ref().map(|s| s.files.len()).unwrap_or(0),
        selection_dirs = selection.as_ref().map(|s| s.dirs.len()).unwrap_or(0),
        "restore operation started"
    );
    operations_repo::append_event(db, op_id, "info", "start", "start", None).await?;

    let access::ResolvedRunAccess { access, .. } =
        access::resolve_success_run_access(db, secrets, run_id).await?;

    let op_dir = super::util::operation_dir(data_dir, op_id);
    tokio::fs::create_dir_all(op_dir.join("staging")).await?;

    let manifest = access::read_manifest(&access).await?;
    operations_repo::append_event(
        db,
        op_id,
        "info",
        "manifest",
        "manifest",
        Some(serde_json::json!({
            "artifacts": manifest.artifacts.len(),
            "entries_count": manifest.entry_index.count,
        })),
    )
    .await?;

    let decryption = super::util::resolve_payload_decryption(db, secrets, &manifest).await?;

    let staging_dir = op_dir.join("staging");
    let parts = parts::fetch_parts(&access, &manifest, &staging_dir).await?;
    info!(
        op_id = %op_id,
        run_id = %run_id,
        parts_count = parts.len(),
        total_bytes = manifest.artifacts.iter().map(|p| p.size).sum::<u64>(),
        "backup parts ready for restore"
    );

    operations_repo::append_event(db, op_id, "info", "restore", "restore", None).await?;
    let dest = destination_dir.to_path_buf();
    let selection = selection.clone();
    let summary = tokio::task::spawn_blocking(move || {
        unpack::restore_from_parts(&parts, &dest, conflict, decryption, selection.as_ref())?;
        Ok::<_, anyhow::Error>(serde_json::json!({
            "destination_dir": dest.to_string_lossy().to_string(),
            "conflict_policy": conflict.as_str(),
        }))
    })
    .await??;

    operations_repo::append_event(db, op_id, "info", "complete", "complete", None).await?;
    operations_repo::complete_operation(
        db,
        op_id,
        operations_repo::OperationStatus::Success,
        Some(summary),
        None,
    )
    .await?;

    let _ = tokio::fs::remove_dir_all(&op_dir).await;

    info!(op_id = %op_id, run_id = %run_id, "restore operation completed");
    Ok(())
}
