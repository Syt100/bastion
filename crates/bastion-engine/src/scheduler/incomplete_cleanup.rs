use std::str::FromStr;
use std::sync::Arc;

use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use url::Url;

use bastion_core::HUB_NODE_ID;
use bastion_core::job_spec;
use bastion_storage::jobs_repo;
use bastion_storage::runs_repo;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;
use bastion_targets::WebdavClient;
use bastion_targets::WebdavCredentials;

pub(super) async fn run_incomplete_cleanup_loop(
    db: SqlitePool,
    secrets: Arc<SecretsCrypto>,
    incomplete_cleanup_days: i64,
    shutdown: CancellationToken,
) {
    let cutoff_seconds = incomplete_cleanup_days.saturating_mul(24 * 60 * 60);
    if cutoff_seconds <= 0 {
        return;
    }

    loop {
        if shutdown.is_cancelled() {
            break;
        }

        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        let cutoff_started_at = now.saturating_sub(cutoff_seconds);

        let mut deleted = 0_u64;
        loop {
            let candidates =
                match runs_repo::list_incomplete_cleanup_candidates(&db, cutoff_started_at, 100)
                    .await
                {
                    Ok(v) => v,
                    Err(error) => {
                        warn!(error = %error, "failed to list incomplete cleanup candidates");
                        break;
                    }
                };
            if candidates.is_empty() {
                break;
            }

            for run in candidates {
                debug!(
                    run_id = %run.id,
                    job_id = %run.job_id,
                    status = ?run.status,
                    started_at = run.started_at,
                    "incomplete cleanup candidate"
                );

                let Some(job) = jobs_repo::get_job(&db, &run.job_id).await.unwrap_or(None) else {
                    continue;
                };
                let spec = match job_spec::parse_value(&job.spec) {
                    Ok(v) => v,
                    Err(error) => {
                        warn!(
                            job_id = %job.id,
                            run_id = %run.id,
                            error = %error,
                            "invalid job spec while cleaning up incomplete run"
                        );
                        continue;
                    }
                };

                match extract_target(&spec) {
                    job_spec::TargetV1::LocalDir { base_dir, .. } => {
                        let base_dir = base_dir.clone();
                        let job_id = job.id.clone();
                        let run_id = run.id.clone();
                        let removed = tokio::task::spawn_blocking(move || {
                            cleanup_local_dir_run(&base_dir, &job_id, &run_id)
                        })
                        .await
                        .unwrap_or(Ok(false))
                        .unwrap_or(false);
                        if removed {
                            deleted = deleted.saturating_add(1);
                        }
                    }
                    job_spec::TargetV1::Webdav {
                        base_url,
                        secret_name,
                        ..
                    } => {
                        let node_id = job.agent_id.as_deref().unwrap_or(HUB_NODE_ID);
                        let removed = match cleanup_webdav_run(
                            &db,
                            &secrets,
                            node_id,
                            base_url,
                            secret_name,
                            &job.id,
                            &run.id,
                        )
                        .await
                        {
                            Ok(v) => v,
                            Err(error) => {
                                warn!(
                                    job_id = %job.id,
                                    run_id = %run.id,
                                    error = %error,
                                    "failed to cleanup stale webdav run"
                                );
                                false
                            }
                        };
                        if removed {
                            deleted = deleted.saturating_add(1);
                        }
                    }
                }
            }
        }

        if deleted > 0 {
            info!(
                deleted,
                incomplete_cleanup_days, "cleaned up stale incomplete target runs"
            );
        }

        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = tokio::time::sleep(std::time::Duration::from_secs(60 * 60)) => {}
        }
    }
}

fn extract_target(spec: &job_spec::JobSpecV1) -> &job_spec::TargetV1 {
    match spec {
        job_spec::JobSpecV1::Filesystem { target, .. } => target,
        job_spec::JobSpecV1::Sqlite { target, .. } => target,
        job_spec::JobSpecV1::Vaultwarden { target, .. } => target,
    }
}

fn cleanup_local_dir_run(
    base_dir: &str,
    job_id: &str,
    run_id: &str,
) -> Result<bool, anyhow::Error> {
    use bastion_backup::{COMPLETE_NAME, ENTRIES_INDEX_NAME, MANIFEST_NAME};

    let run_dir = std::path::Path::new(base_dir).join(job_id).join(run_id);
    if !run_dir.exists() {
        return Ok(false);
    }
    if run_dir.join(COMPLETE_NAME).exists() {
        return Ok(false);
    }

    let mut looks_like_bastion = false;
    if run_dir.join(MANIFEST_NAME).exists() || run_dir.join(ENTRIES_INDEX_NAME).exists() {
        looks_like_bastion = true;
    } else if let Ok(entries) = std::fs::read_dir(&run_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("payload.part") || name.ends_with(".partial") {
                looks_like_bastion = true;
                break;
            }
        }
    }
    if !looks_like_bastion {
        return Ok(false);
    }

    std::fs::remove_dir_all(&run_dir)?;
    Ok(true)
}

async fn cleanup_webdav_run(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    node_id: &str,
    base_url: &str,
    secret_name: &str,
    job_id: &str,
    run_id: &str,
) -> Result<bool, anyhow::Error> {
    use bastion_backup::COMPLETE_NAME;

    let cred_bytes = secrets_repo::get_secret(db, secrets, node_id, "webdav", secret_name)
        .await?
        .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
    let credentials = WebdavCredentials::from_json(&cred_bytes)?;

    let mut base_url = Url::from_str(base_url)?;
    if !base_url.path().ends_with('/') {
        base_url.set_path(&format!("{}/", base_url.path()));
    }

    let client = WebdavClient::new(base_url.clone(), credentials)?;
    let job_url = base_url.join(&format!("{job_id}/"))?;
    let run_url = job_url.join(&format!("{run_id}/"))?;
    let complete_url = run_url.join(COMPLETE_NAME)?;
    if client.head_size(&complete_url).await?.is_some() {
        return Ok(false);
    }

    client.delete(&run_url).await
}
