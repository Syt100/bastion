use std::time::{Duration, Instant};

use futures_util::Sink;
use tokio_tungstenite::tungstenite::Message;

use bastion_backup as backup;
use bastion_core::agent_protocol::{PipelineResolvedV1, TargetResolvedV1};
use bastion_core::job_spec::{FilesystemSource, FsErrorPolicy};
use bastion_core::progress::{ProgressKindV1, ProgressSnapshotV1, ProgressUnitsV1};
use bastion_core::run_failure::RunFailedWithSummary;

use super::super::targets::{store_artifacts_to_resolved_target, target_part_size_bytes};

struct BackupProgressBuilder {
    last_stage: Option<&'static str>,
    last_ts: Option<i64>,
    last_done_bytes: u64,
    source_total: Option<ProgressUnitsV1>,
}

impl BackupProgressBuilder {
    fn new() -> Self {
        Self {
            last_stage: None,
            last_ts: None,
            last_done_bytes: 0,
            source_total: None,
        }
    }

    fn snapshot(
        &mut self,
        update: backup::filesystem::FilesystemBuildProgressUpdate,
    ) -> ProgressSnapshotV1 {
        let stage = update.stage;
        let now_ts = time::OffsetDateTime::now_utc().unix_timestamp();

        if stage != "upload"
            && let Some(total) = update.total
        {
            self.source_total = Some(total);
        }

        let stage_changed = self.last_stage != Some(stage);
        let (rate_bps, eta_seconds) = if stage_changed {
            (None, None)
        } else {
            let dt = self
                .last_ts
                .map(|ts| now_ts.saturating_sub(ts))
                .unwrap_or(0);
            let delta = update.done.bytes.saturating_sub(self.last_done_bytes);
            let rate = if dt > 0 && delta > 0 {
                Some(delta.saturating_div(dt as u64).max(1))
            } else {
                None
            };

            let eta = match (rate, update.total.as_ref()) {
                (Some(rate), Some(total)) if rate > 0 && total.bytes > update.done.bytes => Some(
                    total
                        .bytes
                        .saturating_sub(update.done.bytes)
                        .saturating_div(rate),
                ),
                _ => None,
            };
            (rate, eta)
        };

        self.last_stage = Some(stage);
        self.last_ts = Some(now_ts);
        self.last_done_bytes = update.done.bytes;

        let detail = match stage {
            "upload" => {
                let mut backup = serde_json::Map::new();
                if let Some(total) = self.source_total {
                    backup.insert("source_total".to_string(), serde_json::json!(total));
                }
                backup.insert(
                    "transfer_done_bytes".to_string(),
                    serde_json::json!(update.done.bytes),
                );
                if let Some(total) = update.total {
                    backup.insert(
                        "transfer_total_bytes".to_string(),
                        serde_json::json!(total.bytes),
                    );
                }

                let mut root = serde_json::Map::new();
                root.insert("backup".to_string(), serde_json::Value::Object(backup));
                Some(serde_json::Value::Object(root))
            }
            "scan" | "packaging" => self.source_total.map(|t| {
                serde_json::json!({
                    "backup": {
                        "source_total": t,
                    }
                })
            }),
            _ => None,
        };

        ProgressSnapshotV1 {
            v: 1,
            kind: ProgressKindV1::Backup,
            stage: stage.to_string(),
            ts: now_ts,
            done: update.done,
            total: update.total,
            rate_bps,
            eta_seconds,
            detail,
        }
    }
}

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
    let artifact_format = pipeline.format;
    let started_at = ctx.started_at;

    let (progress_tx, mut progress_rx) =
        tokio::sync::mpsc::channel::<backup::filesystem::FilesystemBuildProgressUpdate>(8);
    let mut progress = BackupProgressBuilder::new();

    let data_dir_buf = ctx.data_dir.to_path_buf();
    let job_id_clone = ctx.job_id.to_string();
    let run_id_clone = ctx.run_id.to_string();
    let progress_tx_build = progress_tx.clone();
    let mut build_handle = tokio::task::spawn_blocking(move || {
        let on_progress = |update: backup::filesystem::FilesystemBuildProgressUpdate| {
            // Pre-scan/packaging is already throttled; blocking send is OK here.
            let _ = progress_tx_build.blocking_send(update);
        };
        backup::filesystem::build_filesystem_run(
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
            Some(&on_progress),
            None,
        )
    });

    let build = loop {
        tokio::select! {
            res = &mut build_handle => break res??,
            maybe_update = progress_rx.recv() => {
                if let Some(update) = maybe_update {
                    super::send_run_progress_snapshot(tx, ctx.run_id, progress.snapshot(update)).await?;
                }
            }
        }
    };

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
    let raw_tree_data_bytes = build.raw_tree_stats.map(|s| s.data_bytes).unwrap_or(0);

    let parts_bytes: u64 = artifacts.parts.iter().map(|p| p.size).sum();
    let entries_size = std::fs::metadata(&artifacts.entries_index_path)?.len();
    let manifest_size = std::fs::metadata(&artifacts.manifest_path)?.len();
    let complete_size = std::fs::metadata(&artifacts.complete_path)?.len();
    let transfer_total_bytes = parts_bytes
        .saturating_add(entries_size)
        .saturating_add(manifest_size)
        .saturating_add(complete_size)
        .saturating_add(raw_tree_data_bytes);

    super::send_run_event(tx, ctx.run_id, "info", "upload", "upload", None).await?;
    struct UploadThrottle {
        last_emit: Instant,
        last_done: u64,
        last_total: Option<u64>,
    }
    const UPLOAD_PROGRESS_MIN_INTERVAL: Duration = Duration::from_secs(1);

    let upload_throttle = std::sync::Arc::new(std::sync::Mutex::new(UploadThrottle {
        last_emit: Instant::now()
            .checked_sub(UPLOAD_PROGRESS_MIN_INTERVAL)
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

            let total_bytes = Some(transfer_total_bytes);
            let done_bytes = p.bytes_done;
            let finished = total_bytes.is_some_and(|t| done_bytes >= t);
            let should_emit =
                finished || now.duration_since(guard.last_emit) >= UPLOAD_PROGRESS_MIN_INTERVAL;
            if !should_emit {
                return;
            }
            if done_bytes == guard.last_done && total_bytes == guard.last_total {
                return;
            }

            guard.last_emit = now;
            guard.last_done = done_bytes;
            guard.last_total = total_bytes;

            let update = backup::filesystem::FilesystemBuildProgressUpdate {
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
            };
            let _ = progress_tx_upload.try_send(update);
        })
    };

    let mut upload_fut = std::pin::pin!(store_artifacts_to_resolved_target(
        ctx.job_id,
        ctx.run_id,
        &target,
        &artifacts,
        Some(upload_cb),
    ));
    let target_summary = loop {
        tokio::select! {
            res = &mut upload_fut => break res?,
            maybe_update = progress_rx.recv() => {
                if let Some(update) = maybe_update {
                    super::send_run_progress_snapshot(tx, ctx.run_id, progress.snapshot(update)).await?;
                }
            }
        }
    };

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

#[cfg(test)]
mod tests {
    use super::BackupProgressBuilder;
    use bastion_backup as backup;
    use bastion_core::progress::ProgressUnitsV1;

    #[test]
    fn upload_snapshot_includes_source_and_transfer_detail() {
        let mut builder = BackupProgressBuilder::new();

        let source_total = ProgressUnitsV1 {
            files: 10,
            dirs: 2,
            bytes: 123,
        };
        let packaging = builder.snapshot(backup::filesystem::FilesystemBuildProgressUpdate {
            stage: "packaging",
            done: ProgressUnitsV1::default(),
            total: Some(source_total),
        });
        let packaging_detail = packaging.detail.expect("packaging detail");
        assert_eq!(
            packaging_detail["backup"]["source_total"]["bytes"]
                .as_u64()
                .unwrap_or_default(),
            source_total.bytes
        );

        let transfer_total_bytes = 999u64;
        let transfer_done_bytes = 111u64;
        let upload = builder.snapshot(backup::filesystem::FilesystemBuildProgressUpdate {
            stage: "upload",
            done: ProgressUnitsV1 {
                files: 0,
                dirs: 0,
                bytes: transfer_done_bytes,
            },
            total: Some(ProgressUnitsV1 {
                files: 0,
                dirs: 0,
                bytes: transfer_total_bytes,
            }),
        });
        let upload_detail = upload.detail.expect("upload detail");

        assert_eq!(
            upload_detail["backup"]["source_total"]["files"]
                .as_u64()
                .unwrap_or_default(),
            source_total.files
        );
        assert_eq!(
            upload_detail["backup"]["transfer_total_bytes"]
                .as_u64()
                .unwrap_or_default(),
            transfer_total_bytes
        );
        assert_eq!(
            upload_detail["backup"]["transfer_done_bytes"]
                .as_u64()
                .unwrap_or_default(),
            transfer_done_bytes
        );
    }
}
