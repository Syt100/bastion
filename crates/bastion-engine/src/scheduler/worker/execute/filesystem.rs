use sqlx::SqlitePool;
use time::OffsetDateTime;

use bastion_core::job_spec;
use bastion_core::run_failure::RunFailedWithSummary;
use bastion_storage::jobs_repo;
use bastion_storage::secrets::SecretsCrypto;

use crate::run_events;
use crate::run_events_bus::RunEventsBus;

use bastion_backup as backup;
use bastion_backup::backup_encryption;

#[allow(clippy::too_many_arguments)]
pub(super) async fn execute_filesystem_run(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    run_events_bus: &RunEventsBus,
    data_dir: &std::path::Path,
    job: &jobs_repo::Job,
    run_id: &str,
    started_at: OffsetDateTime,
    pipeline: job_spec::PipelineV1,
    source: job_spec::FilesystemSource,
    target: job_spec::TargetV1,
) -> Result<serde_json::Value, anyhow::Error> {
    run_events::append_and_broadcast(
        db,
        run_events_bus,
        run_id,
        "info",
        "packaging",
        "packaging",
        None,
    )
    .await?;

    let data_dir = data_dir.to_path_buf();
    let job_id = job.id.clone();
    let run_id_owned = run_id.to_string();
    let part_size = target.part_size_bytes();
    let error_policy = source.error_policy;
    let encryption = backup_encryption::ensure_payload_encryption(db, secrets, &pipeline).await?;
    let artifacts = tokio::task::spawn_blocking(move || {
        backup::filesystem::build_filesystem_run(
            &data_dir,
            &job_id,
            &run_id_owned,
            started_at,
            &source,
            &encryption,
            part_size,
        )
    })
    .await??;

    if artifacts.issues.warnings_total > 0 || artifacts.issues.errors_total > 0 {
        let level = if artifacts.issues.errors_total > 0 {
            "error"
        } else {
            "warn"
        };
        let fields = serde_json::json!({
            "warnings_total": artifacts.issues.warnings_total,
            "errors_total": artifacts.issues.errors_total,
            "sample_warnings": &artifacts.issues.sample_warnings,
            "sample_errors": &artifacts.issues.sample_errors,
        });
        let _ = run_events::append_and_broadcast(
            db,
            run_events_bus,
            run_id,
            level,
            "fs_issues",
            "filesystem issues",
            Some(fields),
        )
        .await;
    }

    let issues = artifacts.issues;
    let artifacts = artifacts.artifacts;

    run_events::append_and_broadcast(db, run_events_bus, run_id, "info", "upload", "upload", None)
        .await?;
    let target_summary = super::super::target_store::store_run_artifacts_to_target(
        db, secrets, &job.id, run_id, &target, &artifacts,
    )
    .await?;

    let _ = tokio::fs::remove_dir_all(&artifacts.run_dir).await;

    let summary = serde_json::json!({
        "target": target_summary,
        "entries_count": artifacts.entries_count,
        "parts": artifacts.parts.len(),
        "filesystem": {
            "warnings_total": issues.warnings_total,
            "errors_total": issues.errors_total,
        },
    });

    if error_policy == job_spec::FsErrorPolicy::SkipFail && issues.errors_total > 0 {
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
