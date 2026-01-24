use std::path::Path;

use sqlx::SqlitePool;

use bastion_core::HUB_NODE_ID;
use bastion_core::job_spec;
use bastion_core::manifest::ArtifactFormatV1;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;
use bastion_targets::WebdavCredentials;

use bastion_backup as backup;
use bastion_targets as targets;

/// Prepare a rolling uploader for `archive_v1` part files.
///
/// Returns:
/// - `on_part_finished` hook to pass into the archive builder
/// - a join handle that completes once all received parts have been stored (and local files deleted)
pub(super) async fn prepare_archive_part_uploader(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    target: &job_spec::TargetV1,
    job_id: &str,
    run_id: &str,
    artifact_format: ArtifactFormatV1,
) -> Result<
    (
        Option<Box<dyn Fn(backup::LocalArtifact) -> std::io::Result<()> + Send>>,
        Option<tokio::task::JoinHandle<Result<(), anyhow::Error>>>,
    ),
    anyhow::Error,
> {
    if artifact_format != ArtifactFormatV1::ArchiveV1 {
        return Ok((None, None));
    }

    let (tx, rx) = tokio::sync::mpsc::channel::<backup::LocalArtifact>(1);

    let handle: tokio::task::JoinHandle<Result<(), anyhow::Error>> = match target {
        job_spec::TargetV1::Webdav {
            base_url,
            secret_name,
            ..
        } => {
            let cred_bytes =
                secrets_repo::get_secret(db, secrets, HUB_NODE_ID, "webdav", secret_name)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
            let credentials = WebdavCredentials::from_json(&cred_bytes)?;

            let base_url = base_url.to_string();
            let job_id = job_id.to_string();
            let run_id = run_id.to_string();

            tokio::spawn(async move {
                targets::webdav::store_run_parts_rolling(
                    &base_url,
                    credentials,
                    &job_id,
                    &run_id,
                    rx,
                )
                .await
                .map(|_| ())
            })
        }
        job_spec::TargetV1::LocalDir { base_dir, .. } => {
            let base_dir = base_dir.to_string();
            let job_id = job_id.to_string();
            let run_id = run_id.to_string();

            tokio::task::spawn_blocking(move || {
                targets::local_dir::store_run_parts_rolling(
                    Path::new(&base_dir),
                    &job_id,
                    &run_id,
                    rx,
                )
                .map(|_| ())
            })
        }
    };

    let on_part_finished: Box<dyn Fn(backup::LocalArtifact) -> std::io::Result<()> + Send> =
        Box::new(move |part| {
            tx.blocking_send(part)
                .map_err(|_| std::io::Error::other("rolling uploader dropped"))?;
            Ok(())
        });

    Ok((Some(on_part_finished), Some(handle)))
}
