use std::time::Instant;

use sqlx::SqlitePool;
use time::OffsetDateTime;

use bastion_core::job_spec;
use bastion_core::progress::{ProgressKindV1, ProgressUnitsV1};
use bastion_core::run_failure::RunFailedWithSummary;
use bastion_storage::jobs_repo;
use bastion_storage::secrets::SecretsCrypto;

use crate::run_events;
use crate::run_events_bus::RunEventsBus;

use bastion_backup as backup;
use bastion_backup::backup_encryption;

use super::progress::{RUN_PROGRESS_MIN_INTERVAL, RunProgressUpdate, spawn_run_progress_writer};

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
    let progress_tx =
        spawn_run_progress_writer(db.clone(), run_id.to_string(), ProgressKindV1::Backup);

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
    let artifact_format = pipeline.format.clone();
    let encryption = backup_encryption::ensure_payload_encryption(db, secrets, &pipeline).await?;
    let progress_tx_build = progress_tx.clone();
    let artifacts = tokio::task::spawn_blocking(move || {
        let on_progress = |update: backup::filesystem::FilesystemBuildProgressUpdate| {
            let _ = progress_tx_build.send(Some(RunProgressUpdate {
                stage: update.stage,
                done: update.done,
                total: update.total,
            }));
        };
        backup::filesystem::build_filesystem_run(
            &data_dir,
            &job_id,
            &run_id_owned,
            started_at,
            &source,
            backup::BuildPipelineOptions {
                artifact_format,
                encryption: &encryption,
                part_size_bytes: part_size,
            },
            Some(&on_progress),
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
    struct UploadThrottle {
        last_emit: Instant,
        last_done: u64,
        last_total: Option<u64>,
    }

    let upload_throttle = std::sync::Arc::new(std::sync::Mutex::new(UploadThrottle {
        last_emit: Instant::now()
            .checked_sub(RUN_PROGRESS_MIN_INTERVAL)
            .unwrap_or_else(Instant::now),
        last_done: 0,
        last_total: None,
    }));
    let progress_tx_upload = progress_tx.clone();
    let upload_cb: std::sync::Arc<dyn Fn(bastion_targets::StoreRunProgress) + Send + Sync> = {
        let upload_throttle = upload_throttle.clone();
        std::sync::Arc::new(move |p: bastion_targets::StoreRunProgress| {
            let now = Instant::now();
            let mut guard = match upload_throttle.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };

            let total_bytes = p.bytes_total;
            let done_bytes = p.bytes_done;
            let finished = total_bytes.is_some_and(|t| done_bytes >= t);
            let should_emit =
                finished || now.duration_since(guard.last_emit) >= RUN_PROGRESS_MIN_INTERVAL;
            if !should_emit {
                return;
            }
            if done_bytes == guard.last_done && total_bytes == guard.last_total {
                return;
            }

            guard.last_emit = now;
            guard.last_done = done_bytes;
            guard.last_total = total_bytes;

            let _ = progress_tx_upload.send(Some(RunProgressUpdate {
                stage: "upload",
                done: ProgressUnitsV1 {
                    files: 0,
                    dirs: 0,
                    bytes: done_bytes,
                },
                total: total_bytes.map(|bytes| ProgressUnitsV1 {
                    files: 0,
                    dirs: 0,
                    bytes,
                }),
            }));
        })
    };
    let target_summary = super::super::target_store::store_run_artifacts_to_target(
        db,
        secrets,
        &job.id,
        run_id,
        &target,
        &artifacts,
        Some(upload_cb),
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
