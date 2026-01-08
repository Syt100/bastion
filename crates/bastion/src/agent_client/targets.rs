use bastion_backup as backup;
use bastion_core::agent_protocol::TargetResolvedV1;
use bastion_targets::WebdavCredentials;

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

pub(super) async fn store_artifacts_to_resolved_target(
    job_id: &str,
    run_id: &str,
    target: &TargetResolvedV1,
    artifacts: &backup::LocalRunArtifacts,
) -> Result<serde_json::Value, anyhow::Error> {
    match target {
        TargetResolvedV1::Webdav {
            base_url,
            username,
            password,
            ..
        } => {
            let creds = WebdavCredentials {
                username: username.clone(),
                password: password.clone(),
            };
            let run_url =
                targets::webdav::store_run(base_url, creds, job_id, run_id, artifacts).await?;
            Ok(serde_json::json!({ "type": "webdav", "run_url": run_url.as_str() }))
        }
        TargetResolvedV1::LocalDir { base_dir, .. } => {
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
