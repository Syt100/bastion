use std::sync::Arc;

use bastion_backup as backup;
use bastion_core::agent_protocol::TargetResolvedV1;
use bastion_driver_api::{StoreRunProgress, StoreRunRequest, TargetRequestLimits};
use bastion_driver_registry::builtins;
use bastion_driver_registry::target_runtime;

use bastion_targets as targets;

pub(super) fn target_part_size_bytes(target: &TargetResolvedV1) -> u64 {
    match target {
        TargetResolvedV1::Webdav {
            part_size_bytes, ..
        } => *part_size_bytes,
        TargetResolvedV1::LocalDir {
            part_size_bytes, ..
        } => *part_size_bytes,
    }
}

fn to_driver_limits(limits: targets::WebdavRequestLimits) -> TargetRequestLimits {
    TargetRequestLimits {
        concurrency: Some(limits.concurrency),
        put_qps: limits.put_qps,
        head_qps: limits.head_qps,
        mkcol_qps: limits.mkcol_qps,
        burst: limits.burst,
    }
}

fn resolve_target_config_for_agent(
    target: &TargetResolvedV1,
) -> Result<(bastion_driver_api::DriverId, serde_json::Value), anyhow::Error> {
    target_runtime::runtime_input_for_resolved_target(target).map_err(anyhow::Error::new)
}

pub(super) async fn store_artifacts_to_resolved_target(
    job_id: &str,
    run_id: &str,
    target: &TargetResolvedV1,
    artifacts: &backup::LocalRunArtifacts,
    webdav_limits: Option<targets::WebdavRequestLimits>,
    on_progress: Option<Arc<dyn Fn(targets::StoreRunProgress) + Send + Sync>>,
) -> Result<serde_json::Value, anyhow::Error> {
    let (driver_id, target_config) = resolve_target_config_for_agent(target)?;
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

#[cfg(test)]
mod tests {
    use bastion_core::agent_protocol::TargetResolvedV1;

    use super::target_part_size_bytes;

    #[test]
    fn target_part_size_bytes_reads_from_variant_field() {
        let webdav = TargetResolvedV1::Webdav {
            base_url: "https://example.com/".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            part_size_bytes: 123,
        };
        assert_eq!(target_part_size_bytes(&webdav), 123);

        let local = TargetResolvedV1::LocalDir {
            base_dir: "/tmp".to_string(),
            part_size_bytes: 456,
        };
        assert_eq!(target_part_size_bytes(&local), 456);
    }
}
