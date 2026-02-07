use std::time::{Duration, Instant};

use futures_util::Sink;
use tokio_tungstenite::tungstenite::Message;

use bastion_backup as backup;
use bastion_core::agent_protocol::{PipelineResolvedV1, TargetResolvedV1};
use bastion_core::job_spec::{
    ConsistencyPolicyV1, FilesystemSource, FsErrorPolicy, SnapshotModeV1,
};
use bastion_core::progress::{ProgressKindV1, ProgressSnapshotV1, ProgressUnitsV1};
use bastion_core::run_failure::RunFailedWithSummary;

use super::super::targets::{store_artifacts_to_resolved_target, target_part_size_bytes};

#[cfg(unix)]
fn create_dir_link(link: &std::path::Path, target: &std::path::Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_dir_link(link: &std::path::Path, target: &std::path::Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
}

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
        self.snapshot_at(time::OffsetDateTime::now_utc().unix_timestamp(), update)
    }

    fn snapshot_at(
        &mut self,
        now_ts: i64,
        update: backup::filesystem::FilesystemBuildProgressUpdate,
    ) -> ProgressSnapshotV1 {
        let stage = update.stage;

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
    fn snapshot_mode_str(mode: SnapshotModeV1) -> &'static str {
        match mode {
            SnapshotModeV1::Off => "off",
            SnapshotModeV1::Auto => "auto",
            SnapshotModeV1::Required => "required",
        }
    }

    let snapshot_mode = source.snapshot_mode;
    let snapshot_provider = source.snapshot_provider.clone();
    let mut snapshot_summary = if snapshot_mode == SnapshotModeV1::Off {
        serde_json::json!({ "mode": snapshot_mode_str(snapshot_mode), "status": "off" })
    } else {
        serde_json::json!({ "mode": snapshot_mode_str(snapshot_mode), "status": "unavailable" })
    };
    let mut snapshot_handle: Option<backup::filesystem::source_snapshot::SourceSnapshotHandle> =
        None;
    let mut read_mapping: Option<backup::filesystem::FilesystemReadMapping> = None;

    if snapshot_mode != SnapshotModeV1::Off {
        let using_paths = source.paths.iter().any(|p| !p.trim().is_empty());
        let snapshot_root = if using_paths {
            let paths = source
                .paths
                .iter()
                .map(|p| p.trim())
                .filter(|p| !p.is_empty())
                .collect::<Vec<_>>();
            if paths.len() == 1 {
                Ok(std::path::PathBuf::from(paths[0]))
            } else {
                Err(format!(
                    "snapshot requires exactly 1 source path (got {})",
                    paths.len()
                ))
            }
        } else {
            let root = source.root.trim();
            if root.is_empty() {
                Err("snapshot requires filesystem.source.root or exactly 1 filesystem.source.paths entry".to_string())
            } else {
                Ok(std::path::PathBuf::from(root))
            }
        };

        match snapshot_root {
            Ok(root) => {
                let provider_override = snapshot_provider.clone();
                let fields = serde_json::json!({
                    "mode": snapshot_mode_str(snapshot_mode),
                    "provider_override": provider_override.as_deref(),
                    "root": root.to_string_lossy().to_string(),
                });
                super::send_run_event(
                    tx,
                    ctx.run_id,
                    "info",
                    "snapshot_started",
                    "snapshot started",
                    Some(fields),
                )
                .await?;

                let run_dir = backup::run_dir(ctx.data_dir, ctx.run_id);
                let attempt = tokio::task::spawn_blocking(move || {
                    backup::filesystem::source_snapshot::attempt_source_snapshot(
                        &root,
                        &run_dir,
                        provider_override.as_deref(),
                    )
                })
                .await?;

                match attempt {
                    backup::filesystem::source_snapshot::SnapshotAttempt::Ready(handle) => {
                        read_mapping = Some(handle.read_mapping());
                        snapshot_handle = Some(handle.clone());
                        let provider = handle.provider.clone();
                        snapshot_summary = serde_json::json!({
                            "mode": snapshot_mode_str(snapshot_mode),
                            "provider": provider.clone(),
                            "status": "ready",
                        });

                        let fields = serde_json::json!({
                            "mode": snapshot_mode_str(snapshot_mode),
                            "provider": provider,
                            "root": handle.original_root.to_string_lossy().to_string(),
                            "snapshot_root": handle.snapshot_root.to_string_lossy().to_string(),
                        });
                        super::send_run_event(
                            tx,
                            ctx.run_id,
                            "info",
                            "snapshot_ready",
                            "snapshot ready",
                            Some(fields),
                        )
                        .await?;
                    }
                    backup::filesystem::source_snapshot::SnapshotAttempt::Unavailable(unavail) => {
                        snapshot_summary = serde_json::json!({
                            "mode": snapshot_mode_str(snapshot_mode),
                            "provider": unavail.provider,
                            "status": "unavailable",
                            "reason": unavail.reason,
                        });

                        super::send_run_event(
                            tx,
                            ctx.run_id,
                            "warn",
                            "snapshot_unavailable",
                            "snapshot unavailable",
                            Some(snapshot_summary.clone()),
                        )
                        .await?;

                        if snapshot_mode == SnapshotModeV1::Required {
                            let summary = serde_json::json!({
                                "artifact_format": pipeline.format,
                                "filesystem": {
                                    "snapshot": snapshot_summary,
                                }
                            });
                            return Err(anyhow::Error::new(RunFailedWithSummary::new(
                                "snapshot_unavailable",
                                "snapshot unavailable (required by policy)",
                                summary,
                            )));
                        }
                    }
                }
            }
            Err(reason) => {
                snapshot_summary = serde_json::json!({
                    "mode": snapshot_mode_str(snapshot_mode),
                    "provider": snapshot_provider,
                    "status": "unavailable",
                    "reason": reason,
                });
                super::send_run_event(
                    tx,
                    ctx.run_id,
                    "warn",
                    "snapshot_unavailable",
                    "snapshot unavailable",
                    Some(snapshot_summary.clone()),
                )
                .await?;

                if snapshot_mode == SnapshotModeV1::Required {
                    let summary = serde_json::json!({
                        "artifact_format": pipeline.format,
                        "filesystem": {
                            "snapshot": snapshot_summary,
                        }
                    });
                    return Err(anyhow::Error::new(RunFailedWithSummary::new(
                        "snapshot_unavailable",
                        "snapshot unavailable (invalid configuration)",
                        summary,
                    )));
                }
            }
        }
    }

    super::send_run_event(tx, ctx.run_id, "info", "packaging", "packaging", None).await?;
    let part_size = target_part_size_bytes(&target);
    let error_policy = source.error_policy;
    let consistency_policy = source.consistency_policy;
    let consistency_fail_threshold = source.consistency_fail_threshold.unwrap_or(0);
    let upload_on_consistency_failure = source.upload_on_consistency_failure.unwrap_or(false);
    let webdav_direct = pipeline.webdav.raw_tree_direct.clone();
    let encryption = super::payload_encryption(pipeline.encryption);
    let artifact_format = pipeline.format;
    let artifact_format_for_summary = artifact_format.clone();
    let started_at = ctx.started_at;

    let allow_rolling_upload = !matches!(
        (consistency_policy, upload_on_consistency_failure),
        (ConsistencyPolicyV1::Fail, false)
    );

    let (on_part_finished, parts_uploader) = if allow_rolling_upload {
        super::prepare_archive_part_uploader(
            &target,
            ctx.job_id,
            ctx.run_id,
            artifact_format.clone(),
        )
    } else {
        (None, None)
    };

    let mut raw_tree_webdav_direct_upload: Option<
        backup::filesystem::RawTreeWebdavDirectUploadConfig,
    > = None;
    if webdav_direct.mode != bastion_core::job_spec::WebdavRawTreeDirectModeV1::Off {
        let supported = allow_rolling_upload
            && artifact_format == bastion_core::manifest::ArtifactFormatV1::RawTreeV1
            && matches!(target, TargetResolvedV1::Webdav { .. });

        if !supported && webdav_direct.mode == bastion_core::job_spec::WebdavRawTreeDirectModeV1::On
        {
            anyhow::bail!(
                "webdav raw-tree direct upload is required by config but not supported by this run (format/target/policy)"
            );
        }

        if supported
            && let TargetResolvedV1::Webdav {
                base_url,
                username,
                password,
                ..
            } = &target
        {
            raw_tree_webdav_direct_upload =
                Some(backup::filesystem::RawTreeWebdavDirectUploadConfig {
                    handle: tokio::runtime::Handle::current(),
                    base_url: base_url.clone(),
                    credentials: bastion_targets::WebdavCredentials {
                        username: username.clone(),
                        password: password.clone(),
                    },
                    max_attempts: 3,
                    resume_by_size: webdav_direct.resume_by_size,
                });
        }
    }
    let using_webdav_raw_tree_direct_upload = raw_tree_webdav_direct_upload.is_some();

    // For raw_tree_v1 + local_dir targets, avoid duplicating the staged `data/` tree by linking the
    // staging `data/` dir to the final target run dir (best-effort; falls back to normal staging).
    let mut direct_target_run_dir: Option<std::path::PathBuf> = None;
    if artifact_format == bastion_core::manifest::ArtifactFormatV1::RawTreeV1
        && let TargetResolvedV1::LocalDir { base_dir, .. } = &target
    {
        let target_run_dir = std::path::Path::new(base_dir)
            .join(ctx.job_id)
            .join(ctx.run_id);
        let target_data_dir = target_run_dir.join("data");
        let stage_data_dir = backup::stage_dir(ctx.data_dir, ctx.run_id).join("data");

        if let Err(error) = std::fs::create_dir_all(&target_data_dir) {
            let fields = serde_json::json!({
                "error": error.to_string(),
                "target_data_dir": target_data_dir.to_string_lossy().to_string(),
            });
            super::send_run_event(
                tx,
                ctx.run_id,
                "warn",
                "direct_data_path_unavailable",
                "raw-tree direct data path unavailable",
                Some(fields),
            )
            .await?;
        } else {
            let _ = std::fs::create_dir_all(backup::stage_dir(ctx.data_dir, ctx.run_id));
            if let Ok(meta) = std::fs::symlink_metadata(&stage_data_dir) {
                let _ = if meta.is_dir() {
                    std::fs::remove_dir_all(&stage_data_dir)
                } else {
                    std::fs::remove_file(&stage_data_dir)
                };
            }

            match create_dir_link(&stage_data_dir, &target_data_dir) {
                Ok(()) => {
                    direct_target_run_dir = Some(target_run_dir);
                    let fields = serde_json::json!({
                        "stage_data_dir": stage_data_dir.to_string_lossy().to_string(),
                        "target_data_dir": target_data_dir.to_string_lossy().to_string(),
                    });
                    super::send_run_event(
                        tx,
                        ctx.run_id,
                        "info",
                        "direct_data_path_ready",
                        "raw-tree direct data path ready",
                        Some(fields),
                    )
                    .await?;
                }
                Err(error) => {
                    let fields = serde_json::json!({
                        "error": error.to_string(),
                        "stage_data_dir": stage_data_dir.to_string_lossy().to_string(),
                        "target_data_dir": target_data_dir.to_string_lossy().to_string(),
                    });
                    super::send_run_event(
                        tx,
                        ctx.run_id,
                        "warn",
                        "direct_data_path_unavailable",
                        "raw-tree direct data path unavailable",
                        Some(fields),
                    )
                    .await?;
                }
            }
        }
    }

    let (progress_tx, mut progress_rx) =
        tokio::sync::mpsc::channel::<backup::filesystem::FilesystemBuildProgressUpdate>(8);
    let mut progress = BackupProgressBuilder::new();

    let data_dir_buf = ctx.data_dir.to_path_buf();
    let job_id_clone = ctx.job_id.to_string();
    let run_id_clone = ctx.run_id.to_string();
    let read_mapping_for_build = read_mapping.clone();
    let progress_tx_build = progress_tx.clone();
    let raw_tree_webdav_direct_upload_for_build = raw_tree_webdav_direct_upload.clone();
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
            read_mapping_for_build.as_ref(),
            Some(&on_progress),
            on_part_finished,
            raw_tree_webdav_direct_upload_for_build,
        )
    });

    let build_join = loop {
        tokio::select! {
            res = &mut build_handle => break res,
            maybe_update = progress_rx.recv() => {
                if let Some(update) = maybe_update {
                    super::send_run_progress_snapshot(tx, ctx.run_id, progress.snapshot(update)).await?;
                }
            }
        }
    };

    let build_res: Result<backup::filesystem::FilesystemRunBuild, anyhow::Error> = match build_join
    {
        Ok(res) => res,
        Err(error) => Err(anyhow::Error::new(error)),
    };

    if let Some(handle) = snapshot_handle.take() {
        let provider = handle.provider.clone();
        let cleanup = tokio::task::spawn_blocking(move || handle.cleanup()).await?;
        if let Err(error) = cleanup {
            if let Some(obj) = snapshot_summary.as_object_mut() {
                obj.insert(
                    "status".to_string(),
                    serde_json::Value::String("cleanup_failed".to_string()),
                );
                obj.insert(
                    "reason".to_string(),
                    serde_json::Value::String(error.to_string()),
                );
            }

            let fields = serde_json::json!({
                "provider": provider,
                "error": error.to_string(),
            });
            super::send_run_event(
                tx,
                ctx.run_id,
                "warn",
                "snapshot_cleanup_failed",
                "snapshot cleanup failed",
                Some(fields),
            )
            .await?;
        }
    }

    if build_res.is_err() {
        if using_webdav_raw_tree_direct_upload
            && let TargetResolvedV1::Webdav {
                base_url,
                username,
                password,
                ..
            } = &target
        {
            let creds = bastion_targets::WebdavCredentials {
                username: username.clone(),
                password: password.clone(),
            };
            let _ = bastion_targets::webdav::cleanup_incomplete_run(
                base_url, creds, ctx.job_id, ctx.run_id,
            )
            .await;
        }

        if let Some(dir) = direct_target_run_dir.as_ref() {
            let _ = tokio::fs::remove_dir_all(dir).await;
        }
    }

    let build = build_res?;

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

    let raw_consistency = build.consistency;
    let consistency_total = raw_consistency.total();
    let consistency_failed =
        consistency_policy.should_fail(consistency_total, consistency_fail_threshold);

    if consistency_policy.should_emit_warnings() && consistency_total > 0 {
        let fields = serde_json::to_value(&raw_consistency)?;
        super::send_run_event(
            tx,
            ctx.run_id,
            "warn",
            "source_consistency",
            "source consistency warnings",
            Some(fields),
        )
        .await?;
    }

    let source_total = build.source_total;
    let issues = build.issues;
    let consistency = if consistency_policy == ConsistencyPolicyV1::Ignore {
        Default::default()
    } else {
        raw_consistency
    };
    let artifacts = build.artifacts;
    let raw_tree_data_bytes = build.raw_tree_stats.map(|s| s.data_bytes).unwrap_or(0);
    let raw_tree_data_bytes_for_transfer = if using_webdav_raw_tree_direct_upload {
        0
    } else {
        raw_tree_data_bytes
    };

    let parts_bytes: u64 = artifacts.parts.iter().map(|p| p.size).sum();
    let entries_size = std::fs::metadata(&artifacts.entries_index_path)?.len();
    let manifest_size = std::fs::metadata(&artifacts.manifest_path)?.len();
    let complete_size = std::fs::metadata(&artifacts.complete_path)?.len();
    let transfer_total_bytes = parts_bytes
        .saturating_add(entries_size)
        .saturating_add(manifest_size)
        .saturating_add(complete_size)
        .saturating_add(raw_tree_data_bytes_for_transfer);
    let mut last_upload_done_bytes: u64 = 0;

    if consistency_failed && !upload_on_consistency_failure {
        let target_summary = serde_json::json!({
            "type": match target {
                TargetResolvedV1::Webdav { .. } => "webdav",
                TargetResolvedV1::LocalDir { .. } => "local_dir",
            }
        });

        let metrics = {
            let mut m = serde_json::Map::new();
            if let Some(t) = source_total {
                m.insert("source_total".to_string(), serde_json::json!(t));
            }
            m.insert(
                "transfer_total_bytes".to_string(),
                serde_json::json!(transfer_total_bytes),
            );
            serde_json::Value::Object(m)
        };

        let summary = serde_json::json!({
            "target": target_summary,
            "artifact_format": artifact_format_for_summary,
            "entries_count": artifacts.entries_count,
            "parts": artifacts.parts.len(),
            "metrics": metrics,
            "filesystem": {
                "warnings_total": issues.warnings_total,
                "errors_total": issues.errors_total,
                "snapshot": snapshot_summary.clone(),
                "consistency": consistency,
            }
        });

        let _ = tokio::fs::remove_dir_all(&artifacts.run_dir).await;
        if let Some(dir) = direct_target_run_dir.as_ref() {
            let _ = tokio::fs::remove_dir_all(dir).await;
        }

        return Err(anyhow::Error::new(RunFailedWithSummary::new(
            "source_consistency",
            format!(
                "source changed during backup (failed by policy): total {consistency_total} > threshold {consistency_fail_threshold}"
            ),
            summary,
        )));
    }

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
            res = &mut upload_fut => {
                match res {
                    Ok(v) => break v,
                    Err(error) => {
                        if using_webdav_raw_tree_direct_upload
                            && let TargetResolvedV1::Webdav { base_url, username, password, .. } = &target
                        {
                            let creds = bastion_targets::WebdavCredentials {
                                username: username.clone(),
                                password: password.clone(),
                            };
                            let _ = bastion_targets::webdav::cleanup_incomplete_run(
                                base_url,
                                creds,
                                ctx.job_id,
                                ctx.run_id,
                            )
                            .await;
                        }

                        return Err(error);
                    }
                }
            },
            maybe_update = progress_rx.recv() => {
                if let Some(update) = maybe_update {
                    if update.stage == "upload" {
                        last_upload_done_bytes = update.done.bytes;
                    }
                    super::send_run_progress_snapshot(tx, ctx.run_id, progress.snapshot(update)).await?;
                }
            }
        }
    };

    if transfer_total_bytes > 0 && last_upload_done_bytes < transfer_total_bytes {
        let final_update = backup::filesystem::FilesystemBuildProgressUpdate {
            stage: "upload",
            done: ProgressUnitsV1 {
                files: 0,
                dirs: 0,
                bytes: transfer_total_bytes,
            },
            total: Some(ProgressUnitsV1 {
                files: 0,
                dirs: 0,
                bytes: transfer_total_bytes,
            }),
        };
        super::send_run_progress_snapshot(tx, ctx.run_id, progress.snapshot(final_update)).await?;
    }

    let _ = tokio::fs::remove_dir_all(&artifacts.run_dir).await;

    let metrics = {
        let mut m = serde_json::Map::new();
        if let Some(t) = source_total {
            m.insert("source_total".to_string(), serde_json::json!(t));
        }
        m.insert(
            "transfer_total_bytes".to_string(),
            serde_json::json!(transfer_total_bytes),
        );
        serde_json::Value::Object(m)
    };

    let mut summary = serde_json::json!({
        "target": target_summary,
        "artifact_format": artifact_format_for_summary,
        "entries_count": artifacts.entries_count,
        "parts": artifacts.parts.len(),
        "metrics": metrics,
        "filesystem": {
            "warnings_total": issues.warnings_total,
            "errors_total": issues.errors_total,
            "snapshot": snapshot_summary.clone(),
            "consistency": consistency,
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

    if consistency_failed {
        return Err(anyhow::Error::new(RunFailedWithSummary::new(
            "source_consistency",
            format!(
                "source changed during backup (failed by policy): total {consistency_total} > threshold {consistency_fail_threshold}"
            ),
            summary,
        )));
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use futures_util::Sink;
    use tokio_tungstenite::tungstenite::Message;

    use super::BackupProgressBuilder;
    use bastion_backup as backup;
    use bastion_core::agent_protocol::{PipelineResolvedV1, TargetResolvedV1};
    use bastion_core::progress::ProgressUnitsV1;
    use bastion_core::run_failure::RunFailedWithSummary;

    #[derive(Default)]
    struct RecordingSink {
        messages: Vec<Message>,
    }

    impl Sink<Message> for RecordingSink {
        type Error = tokio_tungstenite::tungstenite::Error;

        fn poll_ready(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
            self.messages.push(item);
            Ok(())
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn snapshot_required_fails_when_snapshot_config_is_invalid() -> Result<(), anyhow::Error>
    {
        let tmp = tempfile::tempdir()?;
        let dest_dir = tmp.path().join("dest");

        let ctx = super::super::TaskContext {
            data_dir: tmp.path(),
            run_id: "run_id",
            job_id: "job_id",
            started_at: time::OffsetDateTime::now_utc(),
        };

        let pipeline = PipelineResolvedV1::default();
        let source = bastion_core::job_spec::FilesystemSource {
            pre_scan: false,
            // Invalid for snapshots: requires exactly 1 source path in paths-mode.
            paths: vec!["/a".to_string(), "/b".to_string()],
            root: String::new(),
            include: Vec::new(),
            exclude: Vec::new(),
            symlink_policy: bastion_core::job_spec::FsSymlinkPolicy::Keep,
            hardlink_policy: bastion_core::job_spec::FsHardlinkPolicy::Copy,
            error_policy: bastion_core::job_spec::FsErrorPolicy::FailFast,
            snapshot_mode: bastion_core::job_spec::SnapshotModeV1::Required,
            snapshot_provider: None,
            consistency_policy: bastion_core::job_spec::ConsistencyPolicyV1::Warn,
            consistency_fail_threshold: None,
            upload_on_consistency_failure: None,
        };

        let target = TargetResolvedV1::LocalDir {
            base_dir: dest_dir.to_string_lossy().to_string(),
            part_size_bytes: 1024,
        };

        let mut sink = RecordingSink::default();
        let err = super::run_filesystem_backup(&mut sink, &ctx, pipeline, source, target)
            .await
            .expect_err("expected snapshot failure");

        let failed = err
            .downcast_ref::<RunFailedWithSummary>()
            .expect("downcast RunFailedWithSummary");
        assert_eq!(failed.code, "snapshot_unavailable");
        assert!(failed.message.contains("invalid configuration"));
        assert!(
            failed.summary["filesystem"]["snapshot"]["reason"]
                .as_str()
                .unwrap_or_default()
                .contains("exactly 1 source path")
        );
        Ok(())
    }

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

    #[test]
    fn backup_progress_snapshot_resets_rate_and_eta_when_stage_changes() {
        let mut builder = BackupProgressBuilder::new();
        let total = ProgressUnitsV1 {
            files: 0,
            dirs: 0,
            bytes: 100,
        };

        let scan_0 = builder.snapshot_at(
            1000,
            backup::filesystem::FilesystemBuildProgressUpdate {
                stage: "scan",
                done: ProgressUnitsV1::default(),
                total: Some(total),
            },
        );
        assert_eq!(scan_0.rate_bps, None);
        assert_eq!(scan_0.eta_seconds, None);

        let scan_1 = builder.snapshot_at(
            1010,
            backup::filesystem::FilesystemBuildProgressUpdate {
                stage: "scan",
                done: ProgressUnitsV1 {
                    files: 0,
                    dirs: 0,
                    bytes: 50,
                },
                total: Some(total),
            },
        );
        assert_eq!(scan_1.rate_bps, Some(5));
        assert_eq!(scan_1.eta_seconds, Some(10));

        // Stage change resets rate/eta even if bytes increased.
        let packaging_0 = builder.snapshot_at(
            1020,
            backup::filesystem::FilesystemBuildProgressUpdate {
                stage: "packaging",
                done: ProgressUnitsV1 {
                    files: 0,
                    dirs: 0,
                    bytes: 60,
                },
                total: Some(total),
            },
        );
        assert_eq!(packaging_0.rate_bps, None);
        assert_eq!(packaging_0.eta_seconds, None);

        let packaging_1 = builder.snapshot_at(
            1025,
            backup::filesystem::FilesystemBuildProgressUpdate {
                stage: "packaging",
                done: ProgressUnitsV1 {
                    files: 0,
                    dirs: 0,
                    bytes: 70,
                },
                total: Some(total),
            },
        );
        assert_eq!(packaging_1.rate_bps, Some(2));
        assert_eq!(packaging_1.eta_seconds, Some(15));
    }

    #[test]
    fn backup_progress_snapshot_rate_is_never_zero_when_progress_is_made() {
        let mut builder = BackupProgressBuilder::new();
        let total = ProgressUnitsV1 {
            files: 0,
            dirs: 0,
            bytes: 10,
        };

        let _ = builder.snapshot_at(
            1000,
            backup::filesystem::FilesystemBuildProgressUpdate {
                stage: "scan",
                done: ProgressUnitsV1::default(),
                total: Some(total),
            },
        );
        let s = builder.snapshot_at(
            1010,
            backup::filesystem::FilesystemBuildProgressUpdate {
                stage: "scan",
                done: ProgressUnitsV1 {
                    files: 0,
                    dirs: 0,
                    bytes: 1,
                },
                total: Some(total),
            },
        );
        assert_eq!(s.rate_bps, Some(1));
    }
}
