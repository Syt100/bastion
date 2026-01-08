use std::path::{Path, PathBuf};

use bastion_core::manifest::ManifestV1;
use sqlx::SqlitePool;
use tracing::{info, warn};

use bastion_storage::operations_repo;
use bastion_storage::secrets::SecretsCrypto;

use super::unpack::PayloadDecryption;
use super::{ConflictPolicy, RestoreSelection, access, entries_index, parts, unpack, verify};

async fn resolve_payload_decryption(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    manifest: &ManifestV1,
) -> Result<PayloadDecryption, anyhow::Error> {
    match manifest.pipeline.encryption.as_str() {
        "none" => Ok(PayloadDecryption::None),
        "age" => {
            let key_name = manifest
                .pipeline
                .encryption_key
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| anyhow::anyhow!("missing manifest.pipeline.encryption_key"))?;

            let identity = crate::backup_encryption::get_age_identity(db, secrets, key_name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing backup age identity: {}", key_name))?;
            Ok(PayloadDecryption::AgeX25519 { identity })
        }
        other => anyhow::bail!("unsupported manifest.pipeline.encryption: {}", other),
    }
}

fn operation_dir(data_dir: &Path, op_id: &str) -> PathBuf {
    data_dir.join("operations").join(op_id)
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
        if let Err(error) = restore_operation(
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
            let _ = operations_repo::append_event(&db, &op_id, "error", "failed", &msg, None).await;
            let _ = operations_repo::complete_operation(
                &db,
                &op_id,
                operations_repo::OperationStatus::Failed,
                None,
                Some(&msg),
            )
            .await;
            let _ = tokio::fs::remove_dir_all(operation_dir(&data_dir, &op_id)).await;
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
        if let Err(error) = verify_operation(&db, &secrets, &data_dir, &op_id, &run_id).await {
            warn!(op_id = %op_id, run_id = %run_id, error = %error, "verify operation failed");
            let msg = format!("{error:#}");
            let _ = operations_repo::append_event(&db, &op_id, "error", "failed", &msg, None).await;
            let _ = operations_repo::complete_operation(
                &db,
                &op_id,
                operations_repo::OperationStatus::Failed,
                None,
                Some(&msg),
            )
            .await;
            let _ = tokio::fs::remove_dir_all(operation_dir(&data_dir, &op_id)).await;
        }
    });
}

#[allow(clippy::too_many_arguments)]
async fn restore_operation(
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

    let op_dir = operation_dir(data_dir, op_id);
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

    let decryption = resolve_payload_decryption(db, secrets, &manifest).await?;

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

async fn verify_operation(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    data_dir: &Path,
    op_id: &str,
    run_id: &str,
) -> Result<(), anyhow::Error> {
    info!(op_id = %op_id, run_id = %run_id, "verify operation started");
    operations_repo::append_event(db, op_id, "info", "start", "start", None).await?;

    let access::ResolvedRunAccess { run, access } =
        access::resolve_success_run_access(db, secrets, run_id).await?;

    let op_dir = operation_dir(data_dir, op_id);
    let staging_dir = op_dir.join("staging");
    tokio::fs::create_dir_all(&staging_dir).await?;

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

    let decryption = resolve_payload_decryption(db, secrets, &manifest).await?;

    let entries_path = entries_index::fetch_entries_index(&access, &staging_dir).await?;
    let parts = parts::fetch_parts(&access, &manifest, &staging_dir).await?;
    info!(
        op_id = %op_id,
        run_id = %run_id,
        parts_count = parts.len(),
        total_bytes = manifest.artifacts.iter().map(|p| p.size).sum::<u64>(),
        "backup parts ready for verify"
    );

    operations_repo::append_event(db, op_id, "info", "restore", "restore", None).await?;
    let temp_restore_dir = op_dir.join("restore");
    tokio::fs::create_dir_all(&temp_restore_dir).await?;

    let record_count = manifest.entry_index.count;
    let sqlite_paths = verify::sqlite_paths_for_verify(&run);

    let result = tokio::task::spawn_blocking(move || {
        unpack::restore_from_parts(
            &parts,
            &temp_restore_dir,
            ConflictPolicy::Overwrite,
            decryption,
            None,
        )?;
        let verify = verify::verify_restored(&entries_path, &temp_restore_dir, record_count)?;

        let sqlite_results = verify::verify_sqlite_files(&temp_restore_dir, &sqlite_paths)?;
        Ok::<_, anyhow::Error>((verify, sqlite_results))
    })
    .await??;

    let verify = result.0;
    let sqlite_results = result.1;

    operations_repo::append_event(
        db,
        op_id,
        if verify.ok && sqlite_results.ok {
            "info"
        } else {
            "error"
        },
        "verify",
        "verify",
        Some(serde_json::json!({
            "files_total": verify.files_total,
            "files_ok": verify.files_ok,
            "files_failed": verify.files_failed,
            "sample_errors": verify.sample_errors,
            "sqlite": sqlite_results.details,
        })),
    )
    .await?;

    let summary = serde_json::json!({
        "ok": verify.ok && sqlite_results.ok,
        "files_total": verify.files_total,
        "files_ok": verify.files_ok,
        "files_failed": verify.files_failed,
        "sqlite_ok": sqlite_results.ok,
        "sqlite": sqlite_results.details,
    });

    operations_repo::complete_operation(
        db,
        op_id,
        if verify.ok && sqlite_results.ok {
            operations_repo::OperationStatus::Success
        } else {
            operations_repo::OperationStatus::Failed
        },
        Some(summary),
        None,
    )
    .await?;

    let _ = tokio::fs::remove_dir_all(&op_dir).await;

    info!(
        op_id = %op_id,
        run_id = %run_id,
        ok = verify.ok && sqlite_results.ok,
        "verify operation completed"
    );
    Ok(())
}
