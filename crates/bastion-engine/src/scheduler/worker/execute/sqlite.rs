use std::time::Instant;

use sqlx::SqlitePool;
use time::OffsetDateTime;

use bastion_core::job_spec;
use bastion_core::progress::{ProgressKindV1, ProgressUnitsV1};
use bastion_storage::jobs_repo;
use bastion_storage::secrets::SecretsCrypto;

use crate::run_events;
use crate::run_events_bus::RunEventsBus;

use bastion_backup as backup;
use bastion_backup::backup_encryption;

use super::progress::{RUN_PROGRESS_MIN_INTERVAL, RunProgressUpdate, spawn_run_progress_writer};

#[allow(clippy::too_many_arguments)]
pub(super) async fn execute_sqlite_run(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    run_events_bus: &RunEventsBus,
    data_dir: &std::path::Path,
    job: &jobs_repo::Job,
    run_id: &str,
    started_at: OffsetDateTime,
    pipeline: job_spec::PipelineV1,
    source: job_spec::SqliteSource,
    target: job_spec::TargetV1,
) -> Result<serde_json::Value, anyhow::Error> {
    let progress_tx =
        spawn_run_progress_writer(db.clone(), run_id.to_string(), ProgressKindV1::Backup);
    let _ = progress_tx.send(Some(RunProgressUpdate {
        stage: "snapshot",
        done: ProgressUnitsV1::default(),
        total: None,
        detail: None,
    }));

    run_events::append_and_broadcast(
        db,
        run_events_bus,
        run_id,
        "info",
        "snapshot",
        "snapshot",
        None,
    )
    .await?;

    let sqlite_path = source.path.clone();
    let data_dir = data_dir.to_path_buf();
    let job_id = job.id.clone();
    let run_id_owned = run_id.to_string();
    let part_size = target.part_size_bytes();
    let artifact_format = pipeline.format.clone();
    let encryption = backup_encryption::ensure_payload_encryption(db, secrets, &pipeline).await?;
    let build = tokio::task::spawn_blocking(move || {
        backup::sqlite::build_sqlite_run(
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
        )
    })
    .await??;

    if let Some(check) = build.integrity_check.as_ref() {
        let data = serde_json::json!({
            "ok": check.ok,
            "truncated": check.truncated,
            "lines": check.lines,
        });
        let _ = run_events::append_and_broadcast(
            db,
            run_events_bus,
            run_id,
            if check.ok { "info" } else { "error" },
            "integrity_check",
            "integrity_check",
            Some(data),
        )
        .await;

        if !check.ok {
            let first = check.lines.first().cloned().unwrap_or_default();
            anyhow::bail!("sqlite integrity_check failed: {}", first);
        }
    }

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
                detail: None,
            }));
        })
    };

    let target_summary = super::super::target_store::store_run_artifacts_to_target(
        db,
        secrets,
        &job.id,
        run_id,
        &target,
        &build.artifacts,
        Some(upload_cb),
    )
    .await?;

    let _ = tokio::fs::remove_dir_all(&build.artifacts.run_dir).await;

    Ok(serde_json::json!({
        "target": target_summary,
        "entries_count": build.artifacts.entries_count,
        "parts": build.artifacts.parts.len(),
        "sqlite": {
            "path": sqlite_path,
            "snapshot_name": build.snapshot_name,
        }
    }))
}
