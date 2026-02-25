use std::sync::Arc;

use sqlx::SqlitePool;

use bastion_core::HUB_NODE_ID;
use bastion_core::job_spec;
use bastion_driver_api::{StoreRunProgress, StoreRunRequest, TargetRequestLimits};
use bastion_driver_registry::builtins;
use bastion_driver_registry::target_runtime::{self, WebdavRuntimeAuth};
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;
use bastion_targets::WebdavCredentials;

use bastion_backup as backup;
use bastion_targets as targets;

fn to_driver_limits(limits: targets::WebdavRequestLimits) -> TargetRequestLimits {
    TargetRequestLimits {
        concurrency: Some(limits.concurrency),
        put_qps: limits.put_qps,
        head_qps: limits.head_qps,
        mkcol_qps: limits.mkcol_qps,
        burst: limits.burst,
        request_timeout_secs: limits.request_timeout_secs,
        connect_timeout_secs: limits.connect_timeout_secs,
        max_put_attempts: limits.max_put_attempts,
    }
}

async fn resolve_target_config_for_hub(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    target: &job_spec::TargetV1,
) -> Result<(bastion_driver_api::DriverId, serde_json::Value), anyhow::Error> {
    let webdav_auth = match target {
        job_spec::TargetV1::Webdav { secret_name, .. } => {
            let secret_name = secret_name.trim();
            if secret_name.is_empty() {
                anyhow::bail!("webdav.secret_name is required");
            }

            let cred_bytes =
                secrets_repo::get_secret(db, secrets, HUB_NODE_ID, "webdav", secret_name)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
            let credentials = WebdavCredentials::from_json(&cred_bytes)?;
            Some(WebdavRuntimeAuth {
                username: credentials.username,
                password: credentials.password,
                secret_name: Some(secret_name.to_string()),
            })
        }
        job_spec::TargetV1::LocalDir { .. } => None,
    };

    target_runtime::runtime_input_for_job_target(target, webdav_auth.as_ref())
        .map_err(anyhow::Error::new)
}

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
    let (driver_id, target_config) = resolve_target_config_for_hub(db, secrets, target).await?;

    let driver_progress = on_progress.map(|cb| {
        Arc::new(move |p: StoreRunProgress| {
            cb(targets::StoreRunProgress {
                bytes_done: p.bytes_done,
                bytes_total: p.bytes_total,
            })
        }) as Arc<dyn Fn(StoreRunProgress) + Send + Sync>
    });

    let mut writer = builtins::target_registry().open_writer(
        &driver_id,
        StoreRunRequest {
            job_id: job_id.to_string(),
            run_id: run_id.to_string(),
            target_config,
            artifacts: artifacts.clone(),
            limits: webdav_limits.map(to_driver_limits),
            on_progress: driver_progress,
        },
    )?;

    if let Err(error) = writer.upload().await {
        let _ = writer.abort().await;
        return Err(anyhow::Error::new(error));
    }

    let summary = writer.finalize().await?;
    Ok(summary)
}
