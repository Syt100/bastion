use std::path::Path;

use sqlx::SqlitePool;
use tracing::info;

use bastion_storage::operations_repo;
use bastion_storage::secrets::SecretsCrypto;

use super::super::engine::RestoreEngine;
use super::super::sinks::LocalFsSink;
use super::super::sources::{ArtifactSource, LocalDirSource, RunArtifactSource, WebdavSource};
use super::super::{ConflictPolicy, access, verify};

pub(super) async fn verify_operation(
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

    let op_dir = super::util::operation_dir(data_dir, op_id);
    let staging_dir = op_dir.join("staging");
    tokio::fs::create_dir_all(&staging_dir).await?;

    let handle = tokio::runtime::Handle::current();
    let source = match access {
        access::TargetAccess::Webdav { client, run_url } => RunArtifactSource::Webdav(Box::new(
            WebdavSource::new(handle.clone(), *client, run_url),
        )),
        access::TargetAccess::LocalDir { run_dir } => {
            RunArtifactSource::Local(LocalDirSource::new(run_dir))
        }
    };

    let manifest = source.read_manifest().await?;
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

    info!(
        op_id = %op_id,
        run_id = %run_id,
        parts_count = manifest.artifacts.len(),
        total_bytes = manifest.artifacts.iter().map(|p| p.size).sum::<u64>(),
        "backup parts ready for verify"
    );

    operations_repo::append_event(db, op_id, "info", "restore", "restore", None).await?;
    let temp_restore_dir = op_dir.join("restore");
    tokio::fs::create_dir_all(&temp_restore_dir).await?;

    let record_count = manifest.entry_index.count;
    let sqlite_paths = verify::sqlite_paths_for_verify(&run);
    let entries_path = source.fetch_entries_index(&staging_dir).await?;
    let source = source;
    let manifest = manifest.clone();

    let result = tokio::task::spawn_blocking(move || {
        let payload = source.open_payload_reader(&manifest, &staging_dir)?;
        let mut sink = LocalFsSink::new(temp_restore_dir.clone(), ConflictPolicy::Overwrite);
        let mut engine = RestoreEngine::new(&mut sink, decryption, None)?;
        engine.restore(payload)?;

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
