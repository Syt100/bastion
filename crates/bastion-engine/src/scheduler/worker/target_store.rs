use std::sync::Arc;

use sqlx::SqlitePool;

use bastion_core::HUB_NODE_ID;
use bastion_core::job_spec;
use bastion_driver_api::{DriverId, StoreRunProgress, StoreRunRequest, TargetRequestLimits};
use bastion_driver_registry::builtins;
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
    }
}

async fn resolve_target_config_for_hub(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    target: &job_spec::TargetV1,
) -> Result<(DriverId, serde_json::Value), anyhow::Error> {
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

            Ok((
                builtins::webdav_driver_id(),
                serde_json::json!({
                    "base_url": base_url,
                    "username": credentials.username,
                    "password": credentials.password,
                    "secret_name": secret_name,
                }),
            ))
        }
        job_spec::TargetV1::LocalDir { base_dir, .. } => Ok((
            builtins::local_dir_driver_id(),
            serde_json::json!({ "base_dir": base_dir }),
        )),
    }
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

    let summary = builtins::target_registry()
        .store_run(
            &driver_id,
            StoreRunRequest {
                job_id: job_id.to_string(),
                run_id: run_id.to_string(),
                target_config,
                artifacts: artifacts.clone(),
                limits: webdav_limits.map(to_driver_limits),
                on_progress: driver_progress,
            },
        )
        .await?;

    Ok(summary)
}
