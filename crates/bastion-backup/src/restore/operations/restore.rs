use std::path::Path;

use tracing::info;

use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

use bastion_core::progress::{ProgressKindV1, ProgressUnitsV1};
use bastion_storage::operations_repo;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;

use super::super::engine::RestoreEngine;
use super::super::raw_tree;
use super::super::sinks::{LocalFsSink, WebdavSink};
use super::super::sources::{ArtifactSource, DriverSource, RunArtifactSource};
use super::super::{ConflictPolicy, RestoreDestination, RestoreSelection, access};
use super::progress::{OperationProgressUpdate, spawn_operation_progress_writer};
use bastion_core::HUB_NODE_ID;
use bastion_core::manifest::ArtifactFormatV1;
use bastion_targets::{WebdavClient, WebdavCredentials};
use url::Url;

#[allow(clippy::too_many_arguments)]
pub(super) async fn restore_operation(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    data_dir: &Path,
    op_id: &str,
    run_id: &str,
    destination: &RestoreDestination,
    conflict: ConflictPolicy,
    selection: Option<RestoreSelection>,
    cancel_token: &CancellationToken,
) -> Result<(), anyhow::Error> {
    super::check_operation_canceled(op_id, cancel_token)?;
    let destination_label = match destination {
        RestoreDestination::LocalFs { directory } => directory.display().to_string(),
        RestoreDestination::Webdav {
            base_url,
            secret_name,
            prefix,
        } => format!("webdav:{base_url} secret={secret_name} prefix={prefix}"),
    };
    info!(
        op_id = %op_id,
        run_id = %run_id,
        destination = %destination_label,
        conflict = %conflict.as_str(),
        selection_files = selection.as_ref().map(|s| s.files.len()).unwrap_or(0),
        selection_dirs = selection.as_ref().map(|s| s.dirs.len()).unwrap_or(0),
        "restore operation started"
    );
    operations_repo::append_event(db, op_id, "info", "start", "start", None).await?;
    let progress_tx =
        spawn_operation_progress_writer(db.clone(), op_id.to_string(), ProgressKindV1::Restore);
    super::check_operation_canceled(op_id, cancel_token)?;

    let access::ResolvedRunAccess { access, .. } =
        access::resolve_success_run_access(db, secrets, run_id).await?;
    super::check_operation_canceled(op_id, cancel_token)?;

    let op_dir = super::util::operation_dir(data_dir, op_id);
    tokio::fs::create_dir_all(op_dir.join("staging")).await?;
    super::check_operation_canceled(op_id, cancel_token)?;

    let handle = tokio::runtime::Handle::current();
    let source = RunArtifactSource::Driver(DriverSource::new(handle, access.reader()));

    let manifest = source.read_manifest().await?;
    super::check_operation_canceled(op_id, cancel_token)?;
    let artifact_format = manifest.pipeline.format.clone();
    operations_repo::append_event(
        db,
        op_id,
        "info",
        "manifest",
        "manifest",
        Some(serde_json::json!({
            "format": format!("{:?}", artifact_format),
            "artifacts": manifest.artifacts.len(),
            "entries_count": manifest.entry_index.count,
        })),
    )
    .await?;

    let decryption = super::util::resolve_payload_decryption(db, secrets, &manifest).await?;
    super::check_operation_canceled(op_id, cancel_token)?;

    enum ResolvedDestination {
        LocalFs {
            directory: std::path::PathBuf,
        },
        Webdav {
            prefix_url: Url,
            credentials: WebdavCredentials,
        },
    }

    let resolved_destination = match destination {
        RestoreDestination::LocalFs { directory } => ResolvedDestination::LocalFs {
            directory: directory.clone(),
        },
        RestoreDestination::Webdav {
            base_url,
            secret_name,
            prefix,
        } => {
            let secret_name = secret_name.trim();
            if secret_name.is_empty() {
                anyhow::bail!("webdav.secret_name is required");
            }

            let cred_bytes =
                secrets_repo::get_secret(db, secrets, HUB_NODE_ID, "webdav", secret_name)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
            let credentials = WebdavCredentials::from_json(&cred_bytes)?;

            let mut base_url = Url::parse(base_url.trim())?;
            if !base_url.path().ends_with('/') {
                base_url.set_path(&format!("{}/", base_url.path()));
            }

            let mut prefix_url = base_url;
            {
                let mut segs = prefix_url
                    .path_segments_mut()
                    .map_err(|_| anyhow::anyhow!("webdav base_url cannot be a base"))?;
                for part in prefix
                    .trim()
                    .trim_matches('/')
                    .split('/')
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                {
                    segs.push(part);
                }
            }
            if !prefix_url.path().ends_with('/') {
                prefix_url.set_path(&format!("{}/", prefix_url.path()));
            }

            ResolvedDestination::Webdav {
                prefix_url,
                credentials,
            }
        }
    };

    let staging_dir = op_dir.join("staging");
    info!(
        op_id = %op_id,
        run_id = %run_id,
        artifact_format = ?artifact_format,
        parts_count = manifest.artifacts.len(),
        total_bytes = manifest.artifacts.iter().map(|p| p.size).sum::<u64>(),
        "backup parts ready for restore"
    );

    operations_repo::append_event(db, op_id, "info", "restore", "restore", None).await?;
    let op_id_for_blocking = op_id.to_string();
    let op_id_for_cancel = op_id.to_string();
    let source = source;
    let manifest = manifest.clone();
    let entries_index_path = if artifact_format == ArtifactFormatV1::RawTreeV1 {
        Some(source.fetch_entries_index(&staging_dir).await?)
    } else {
        None
    };
    super::check_operation_canceled(op_id, cancel_token)?;
    let selection = selection.clone();
    let progress_tx_restore = progress_tx.clone();
    let cancel_token = cancel_token.clone();
    let cancel_token_for_blocking = cancel_token.clone();
    let summary = tokio::task::spawn_blocking(move || {
        let on_progress = |done: ProgressUnitsV1| {
            let _ = progress_tx_restore.send(Some(OperationProgressUpdate {
                stage: "restore",
                done,
                total: None,
            }));
        };
        let cancel_check =
            || super::check_operation_canceled(&op_id_for_cancel, &cancel_token_for_blocking);
        cancel_check()?;
        match artifact_format {
            ArtifactFormatV1::ArchiveV1 => {
                let payload = source.open_payload_reader(&manifest, &staging_dir)?;
                cancel_check()?;
                match resolved_destination {
                    ResolvedDestination::LocalFs { directory } => {
                        let mut sink = LocalFsSink::new(directory.clone(), conflict);
                        let mut engine = RestoreEngine::new_with_cancel(
                            &mut sink,
                            decryption,
                            selection.as_ref(),
                            Some(&on_progress),
                            Some(&cancel_check),
                        )?;
                        engine.restore(payload)?;
                        Ok::<_, anyhow::Error>(serde_json::json!({
                            "destination": { "type": "local_fs", "directory": directory.to_string_lossy().to_string() },
                            "conflict_policy": conflict.as_str(),
                        }))
                    }
                    ResolvedDestination::Webdav {
                        prefix_url,
                        credentials,
                    } => {
                        let client = WebdavClient::new(prefix_url.clone(), credentials)?;
                        let handle = tokio::runtime::Handle::current();
                        let mut sink = WebdavSink::new(
                            handle,
                            client,
                            prefix_url.clone(),
                            conflict,
                            op_id_for_blocking,
                            staging_dir.join("webdav_sink"),
                        )?;
                        let mut engine = RestoreEngine::new_with_cancel(
                            &mut sink,
                            decryption,
                            selection.as_ref(),
                            Some(&on_progress),
                            Some(&cancel_check),
                        )?;
                        engine.restore(payload)?;
                        Ok::<_, anyhow::Error>(serde_json::json!({
                            "destination": { "type": "webdav", "prefix_url": prefix_url.as_str() },
                            "conflict_policy": conflict.as_str(),
                        }))
                    }
                }
            }
            ArtifactFormatV1::RawTreeV1 => {
                let entries_index_path = entries_index_path
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("missing entries index path"))?;
                match resolved_destination {
                    ResolvedDestination::LocalFs { directory } => {
                        raw_tree::restore_raw_tree_to_local_fs_with_cancel_check(
                            &source,
                            entries_index_path,
                            &staging_dir,
                            &directory,
                            conflict,
                            selection.as_ref(),
                            Some(&on_progress),
                            Some(&cancel_check),
                        )?;
                        Ok::<_, anyhow::Error>(serde_json::json!({
                            "destination": { "type": "local_fs", "directory": directory.to_string_lossy().to_string() },
                            "conflict_policy": conflict.as_str(),
                        }))
                    }
                    ResolvedDestination::Webdav {
                        prefix_url,
                        credentials,
                    } => {
                        let client = WebdavClient::new(prefix_url.clone(), credentials)?;
                        let handle = tokio::runtime::Handle::current();
                        let mut sink = WebdavSink::new(
                            handle,
                            client,
                            prefix_url.clone(),
                            conflict,
                            op_id_for_blocking,
                            staging_dir.join("webdav_sink"),
                        )?;
                        raw_tree::restore_raw_tree_to_webdav_with_cancel_check(
                            &source,
                            entries_index_path,
                            &staging_dir,
                            &mut sink,
                            selection.as_ref(),
                            Some(&on_progress),
                            Some(&cancel_check),
                        )?;
                        Ok::<_, anyhow::Error>(serde_json::json!({
                            "destination": { "type": "webdav", "prefix_url": prefix_url.as_str() },
                            "conflict_policy": conflict.as_str(),
                        }))
                    }
                }
            }
        }
    })
    .await??;
    super::check_operation_canceled(op_id, &cancel_token)?;

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
