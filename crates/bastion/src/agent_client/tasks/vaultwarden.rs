use futures_util::Sink;
use tokio_tungstenite::tungstenite::Message;

use bastion_backup as backup;
use bastion_core::agent_protocol::PipelineResolvedV1;
use bastion_core::agent_protocol::TargetResolvedV1;
use bastion_core::job_spec::VaultwardenSource;

use super::super::targets::{store_artifacts_to_resolved_target, target_part_size_bytes};

pub(super) async fn run_vaultwarden_backup(
    tx: &mut (impl Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    ctx: &super::TaskContext<'_>,
    pipeline: PipelineResolvedV1,
    source: VaultwardenSource,
    target: TargetResolvedV1,
) -> Result<serde_json::Value, anyhow::Error> {
    super::send_run_event(tx, ctx.run_id, "info", "snapshot", "snapshot", None).await?;
    let vw_data_dir = source.data_dir.clone();
    let part_size = target_part_size_bytes(&target);
    let encryption = super::payload_encryption(pipeline.encryption);
    let artifact_format = pipeline.format;
    let started_at = ctx.started_at;

    let data_dir_buf = ctx.data_dir.to_path_buf();
    let job_id_clone = ctx.job_id.to_string();
    let run_id_clone = ctx.run_id.to_string();
    let artifacts = tokio::task::spawn_blocking(move || {
        backup::vaultwarden::build_vaultwarden_run(
            &data_dir_buf,
            &job_id_clone,
            &run_id_clone,
            started_at,
            artifact_format,
            &source,
            &encryption,
            part_size,
        )
    })
    .await??;

    super::send_run_event(tx, ctx.run_id, "info", "upload", "upload", None).await?;
    let target_summary =
        store_artifacts_to_resolved_target(ctx.job_id, ctx.run_id, &target, &artifacts).await?;
    let _ = tokio::fs::remove_dir_all(&artifacts.run_dir).await;

    Ok(serde_json::json!({
        "target": target_summary,
        "entries_count": artifacts.entries_count,
        "parts": artifacts.parts.len(),
        "vaultwarden": {
            "data_dir": vw_data_dir,
            "db": "db.sqlite3",
        }
    }))
}
