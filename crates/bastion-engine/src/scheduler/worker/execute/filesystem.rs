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
use super::rolling_archive;

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

    let (on_part_finished, parts_uploader) = rolling_archive::prepare_archive_part_uploader(
        db,
        secrets,
        &target,
        &job.id,
        run_id,
        artifact_format.clone(),
    )
    .await?;

    let progress_tx_build = progress_tx.clone();
    let build = tokio::task::spawn_blocking(move || {
        let on_progress = |update: backup::filesystem::FilesystemBuildProgressUpdate| {
            let _ = progress_tx_build.send(Some(RunProgressUpdate {
                stage: update.stage,
                done: update.done,
                total: update.total,
                detail: update
                    .total
                    .map(|t| serde_json::json!({ "backup": { "source_total": t } })),
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
            on_part_finished,
        )
    })
    .await??;

    if let Some(handle) = parts_uploader {
        handle.await??;
    }

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

    let source_total = build.source_total;
    let raw_tree_stats = build.raw_tree_stats;
    let issues = build.issues;
    let artifacts = build.artifacts;

    let parts_bytes: u64 = artifacts.parts.iter().map(|p| p.size).sum();
    let entries_size = std::fs::metadata(&artifacts.entries_index_path)?.len();
    let manifest_size = std::fs::metadata(&artifacts.manifest_path)?.len();
    let complete_size = std::fs::metadata(&artifacts.complete_path)?.len();
    let raw_tree_data_bytes = raw_tree_stats.map(|s| s.data_bytes).unwrap_or(0);
    let transfer_total_bytes = parts_bytes
        .saturating_add(entries_size)
        .saturating_add(manifest_size)
        .saturating_add(complete_size)
        .saturating_add(raw_tree_data_bytes);

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
        let source_total_for_detail = source_total;
        std::sync::Arc::new(move |p: bastion_targets::StoreRunProgress| {
            let now = Instant::now();
            let mut guard = match upload_throttle.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };

            let total_bytes = Some(transfer_total_bytes);
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

            let detail = {
                let mut backup = serde_json::Map::new();
                if let Some(t) = source_total_for_detail {
                    backup.insert("source_total".to_string(), serde_json::json!(t));
                }
                backup.insert(
                    "transfer_total_bytes".to_string(),
                    serde_json::json!(transfer_total_bytes),
                );
                backup.insert(
                    "transfer_done_bytes".to_string(),
                    serde_json::json!(done_bytes),
                );

                let mut root = serde_json::Map::new();
                root.insert("backup".to_string(), serde_json::Value::Object(backup));
                serde_json::Value::Object(root)
            };

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
                detail: Some(detail),
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
        "artifact_format": pipeline.format,
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
