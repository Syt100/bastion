use futures_util::Sink;
use tokio_tungstenite::tungstenite::Message;

use bastion_backup as backup;
use bastion_core::agent_protocol::PipelineResolvedV1;
use bastion_core::agent_protocol::TargetResolvedV1;
use bastion_core::job_spec::SqliteSource;

use super::super::targets::{store_artifacts_to_resolved_target, target_part_size_bytes};

pub(super) async fn run_sqlite_backup(
    tx: &mut (impl Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    ctx: &super::TaskContext<'_>,
    pipeline: PipelineResolvedV1,
    source: SqliteSource,
    target: TargetResolvedV1,
) -> Result<serde_json::Value, anyhow::Error> {
    super::send_run_event(tx, ctx.run_id, "info", "snapshot", "snapshot", None).await?;
    let sqlite_path = source.path.clone();
    let part_size = target_part_size_bytes(&target);
    let encryption = super::payload_encryption(pipeline.encryption);
    let artifact_format = pipeline.format;
    let started_at = ctx.started_at;

    let data_dir_buf = ctx.data_dir.to_path_buf();
    let job_id_clone = ctx.job_id.to_string();
    let run_id_clone = ctx.run_id.to_string();
    let build = tokio::task::spawn_blocking(move || {
        backup::sqlite::build_sqlite_run(
            &data_dir_buf,
            &job_id_clone,
            &run_id_clone,
            started_at,
            &source,
            backup::BuildPipelineOptions {
                artifact_format,
                encryption: &encryption,
                part_size_bytes: part_size,
            },
        )
    })
    .await??;

    if let Some(check) = build.integrity_check.as_ref() {
        let data = serde_json::json!({
            "ok": check.ok,
            "truncated": check.truncated,
            "lines": check.lines,
        });
        super::send_run_event(
            tx,
            ctx.run_id,
            if check.ok { "info" } else { "error" },
            "integrity_check",
            "integrity_check",
            Some(data),
        )
        .await?;
        if !check.ok {
            let first = check.lines.first().cloned().unwrap_or_default();
            anyhow::bail!("sqlite integrity_check failed: {}", first);
        }
    }

    super::send_run_event(tx, ctx.run_id, "info", "upload", "upload", None).await?;
    let target_summary =
        store_artifacts_to_resolved_target(ctx.job_id, ctx.run_id, &target, &build.artifacts)
            .await?;
    let _ = tokio::fs::remove_dir_all(&build.artifacts.run_dir).await;

    Ok(serde_json::json!({
        "target": target_summary,
        "entries_count": build.artifacts.entries_count,
        "parts": build.artifacts.parts.len(),
        "sqlite": {
            "path": sqlite_path,
            "snapshot_name": build.snapshot_name,
            "snapshot_size": build.snapshot_size,
            "integrity_check": build.integrity_check.map(|check| serde_json::json!({
                "ok": check.ok,
                "truncated": check.truncated,
                "lines": check.lines,
            })),
        },
    }))
}
