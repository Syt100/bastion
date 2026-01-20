use std::path::Path;

use tracing::info;

use sqlx::SqlitePool;

use bastion_storage::operations_repo;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;

use super::super::engine::RestoreEngine;
use super::super::sinks::{LocalFsSink, WebdavSink};
use super::super::sources::{ArtifactSource, LocalDirSource, RunArtifactSource, WebdavSource};
use super::super::{ConflictPolicy, RestoreDestination, RestoreSelection, access};
use bastion_core::HUB_NODE_ID;
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
) -> Result<(), anyhow::Error> {
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

    let access::ResolvedRunAccess { access, .. } =
        access::resolve_success_run_access(db, secrets, run_id).await?;

    let op_dir = super::util::operation_dir(data_dir, op_id);
    tokio::fs::create_dir_all(op_dir.join("staging")).await?;

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

    enum ResolvedDestination {
        LocalFs { directory: std::path::PathBuf },
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
        parts_count = manifest.artifacts.len(),
        total_bytes = manifest.artifacts.iter().map(|p| p.size).sum::<u64>(),
        "backup parts ready for restore"
    );

    operations_repo::append_event(db, op_id, "info", "restore", "restore", None).await?;
    let op_id_for_blocking = op_id.to_string();
    let source = source;
    let manifest = manifest.clone();
    let selection = selection.clone();
    let summary = tokio::task::spawn_blocking(move || {
        let payload = source.open_payload_reader(&manifest, &staging_dir)?;
        match resolved_destination {
            ResolvedDestination::LocalFs { directory } => {
                let mut sink = LocalFsSink::new(directory.clone(), conflict);
                let mut engine = RestoreEngine::new(&mut sink, decryption, selection.as_ref())?;
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
                let mut engine = RestoreEngine::new(&mut sink, decryption, selection.as_ref())?;
                engine.restore(payload)?;
                Ok::<_, anyhow::Error>(serde_json::json!({
                    "destination": { "type": "webdav", "prefix_url": prefix_url.as_str() },
                    "conflict_policy": conflict.as_str(),
                }))
            }
        }
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
