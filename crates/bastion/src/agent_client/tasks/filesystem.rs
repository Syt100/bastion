use futures_util::Sink;
use tokio_tungstenite::tungstenite::Message;

use bastion_backup as backup;
use bastion_core::agent_protocol::{PipelineResolvedV1, TargetResolvedV1};
use bastion_core::job_spec::{FilesystemSource, FsErrorPolicy};
use bastion_core::run_failure::RunFailedWithSummary;

use super::super::targets::{store_artifacts_to_resolved_target, target_part_size_bytes};

pub(super) async fn run_filesystem_backup(
    tx: &mut (impl Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    ctx: &super::TaskContext<'_>,
    pipeline: PipelineResolvedV1,
    source: FilesystemSource,
    target: TargetResolvedV1,
) -> Result<serde_json::Value, anyhow::Error> {
    super::send_run_event(tx, ctx.run_id, "info", "packaging", "packaging", None).await?;
    let part_size = target_part_size_bytes(&target);
    let error_policy = source.error_policy;
    let encryption = super::payload_encryption(pipeline.encryption);
    let started_at = ctx.started_at;

    let data_dir_buf = ctx.data_dir.to_path_buf();
    let job_id_clone = ctx.job_id.to_string();
    let run_id_clone = ctx.run_id.to_string();
    let build = tokio::task::spawn_blocking(move || {
        backup::filesystem::build_filesystem_run(
            &data_dir_buf,
            &job_id_clone,
            &run_id_clone,
            started_at,
            &source,
            &encryption,
            part_size,
        )
    })
    .await??;

    if build.issues.warnings_total > 0 || build.issues.errors_total > 0 {
        let level = if build.issues.errors_total > 0 {
            "error"
        } else {
            "warn"
        };
        let fields = serde_json::json!({
            "warnings_total": build.issues.warnings_total,
            "errors_total": build.issues.errors_total,
            "sample_warnings": &build.issues.sample_warnings,
            "sample_errors": &build.issues.sample_errors,
        });
        super::send_run_event(
            tx,
            ctx.run_id,
            level,
            "fs_issues",
            "filesystem issues",
            Some(fields),
        )
        .await?;
    }

    let issues = build.issues;
    let artifacts = build.artifacts;

    super::send_run_event(tx, ctx.run_id, "info", "upload", "upload", None).await?;
    let target_summary =
        store_artifacts_to_resolved_target(ctx.job_id, ctx.run_id, &target, &artifacts).await?;

    let _ = tokio::fs::remove_dir_all(&artifacts.run_dir).await;

    let mut summary = serde_json::json!({
        "target": target_summary,
        "entries_count": artifacts.entries_count,
        "parts": artifacts.parts.len(),
        "filesystem": {
            "warnings_total": issues.warnings_total,
            "errors_total": issues.errors_total,
        }
    });

    if error_policy == FsErrorPolicy::SkipFail && issues.errors_total > 0 {
        if let Some(obj) = summary.as_object_mut() {
            obj.insert(
                "error_code".to_string(),
                serde_json::Value::String("fs_issues".to_string()),
            );
        }
        return Err(anyhow::Error::new(RunFailedWithSummary::new(
            "fs_issues",
            format!(
                "filesystem backup completed with {} errors",
                issues.errors_total
            ),
            summary,
        )));
    }

    Ok(summary)
}
