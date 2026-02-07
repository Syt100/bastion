use std::sync::Arc;

use sqlx::SqlitePool;

use bastion_core::HUB_NODE_ID;
use bastion_core::job_spec;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;
use bastion_targets::WebdavCredentials;

use bastion_backup as backup;
use bastion_targets as targets;

#[allow(clippy::too_many_arguments)]
pub(super) async fn store_run_artifacts_to_target(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    job_id: &str,
    run_id: &str,
    target: &job_spec::TargetV1,
    artifacts: &backup::LocalRunArtifacts,
    webdav_limits: Option<targets::WebdavRequestLimits>,
    on_progress: Option<Arc<dyn Fn(targets::StoreRunProgress) + Send + Sync>>,
) -> Result<serde_json::Value, anyhow::Error> {
    match target {
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

            let run_url = targets::webdav::store_run(
                base_url,
                credentials,
                job_id,
                run_id,
                artifacts,
                webdav_limits,
                on_progress.clone(),
            )
            .await?;
            Ok(serde_json::json!({ "type": "webdav", "run_url": run_url.as_str() }))
        }
        job_spec::TargetV1::LocalDir { base_dir, .. } => {
            let base_dir = base_dir.to_string();
            let job_id = job_id.to_string();
            let run_id = run_id.to_string();
            let artifacts = artifacts.clone();
            let run_dir = tokio::task::spawn_blocking(move || {
                targets::local_dir::store_run(
                    std::path::Path::new(&base_dir),
                    &job_id,
                    &run_id,
                    &artifacts,
                    on_progress.as_deref(),
                )
            })
            .await??;
            Ok(serde_json::json!({
                "type": "local_dir",
                "run_dir": run_dir.to_string_lossy().to_string()
            }))
        }
    }
}
