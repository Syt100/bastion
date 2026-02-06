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

#[cfg(unix)]
fn create_dir_link(link: &std::path::Path, target: &std::path::Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_dir_link(link: &std::path::Path, target: &std::path::Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
}

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

    let data_dir = data_dir.to_path_buf();
    let job_id = job.id.clone();
    let run_id_owned = run_id.to_string();

    fn snapshot_mode_str(mode: job_spec::SnapshotModeV1) -> &'static str {
        match mode {
            job_spec::SnapshotModeV1::Off => "off",
            job_spec::SnapshotModeV1::Auto => "auto",
            job_spec::SnapshotModeV1::Required => "required",
        }
    }

    let snapshot_mode = source.snapshot_mode;
    let snapshot_provider = source.snapshot_provider.clone();
    let mut snapshot_summary = if snapshot_mode == job_spec::SnapshotModeV1::Off {
        serde_json::json!({ "mode": snapshot_mode_str(snapshot_mode), "status": "off" })
    } else {
        serde_json::json!({ "mode": snapshot_mode_str(snapshot_mode), "status": "unavailable" })
    };
    let mut snapshot_handle: Option<backup::filesystem::source_snapshot::SourceSnapshotHandle> =
        None;
    let mut read_mapping: Option<backup::filesystem::FilesystemReadMapping> = None;

    if snapshot_mode != job_spec::SnapshotModeV1::Off {
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
                let _ = run_events::append_and_broadcast(
                    db,
                    run_events_bus,
                    run_id,
                    "info",
                    "snapshot_started",
                    "snapshot started",
                    Some(fields),
                )
                .await;

                let run_dir = backup::run_dir(&data_dir, &run_id_owned);
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
                        let provider = handle.provider.clone();
                        read_mapping = Some(handle.read_mapping());
                        snapshot_handle = Some(handle.clone());
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
                        let _ = run_events::append_and_broadcast(
                            db,
                            run_events_bus,
                            run_id,
                            "info",
                            "snapshot_ready",
                            "snapshot ready",
                            Some(fields),
                        )
                        .await;
                    }
                    backup::filesystem::source_snapshot::SnapshotAttempt::Unavailable(unavail) => {
                        snapshot_summary = serde_json::json!({
                            "mode": snapshot_mode_str(snapshot_mode),
                            "provider": unavail.provider,
                            "status": "unavailable",
                            "reason": unavail.reason,
                        });

                        let _ = run_events::append_and_broadcast(
                            db,
                            run_events_bus,
                            run_id,
                            "warn",
                            "snapshot_unavailable",
                            "snapshot unavailable",
                            Some(snapshot_summary.clone()),
                        )
                        .await;

                        if snapshot_mode == job_spec::SnapshotModeV1::Required {
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
                let _ = run_events::append_and_broadcast(
                    db,
                    run_events_bus,
                    run_id,
                    "warn",
                    "snapshot_unavailable",
                    "snapshot unavailable",
                    Some(snapshot_summary.clone()),
                )
                .await;

                if snapshot_mode == job_spec::SnapshotModeV1::Required {
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
    let part_size = target.part_size_bytes();
    let error_policy = source.error_policy;
    let consistency_policy = source.consistency_policy;
    let consistency_fail_threshold = source.consistency_fail_threshold.unwrap_or(0);
    let upload_on_consistency_failure = source.upload_on_consistency_failure.unwrap_or(false);
    let artifact_format = pipeline.format.clone();
    let encryption = backup_encryption::ensure_payload_encryption(db, secrets, &pipeline).await?;

    let allow_rolling_upload = !matches!(
        (consistency_policy, upload_on_consistency_failure),
        (job_spec::ConsistencyPolicyV1::Fail, false)
    );

    let (on_part_finished, parts_uploader) = if allow_rolling_upload {
        rolling_archive::prepare_archive_part_uploader(
            db,
            secrets,
            &target,
            &job.id,
            run_id,
            artifact_format.clone(),
        )
        .await?
    } else {
        (None, None)
    };

    let mut raw_tree_webdav_direct_upload: Option<
        backup::filesystem::RawTreeWebdavDirectUploadConfig,
    > = None;
    if allow_rolling_upload
        && artifact_format == bastion_core::manifest::ArtifactFormatV1::RawTreeV1
        && let job_spec::TargetV1::Webdav {
            base_url,
            secret_name,
            ..
        } = &target
    {
        let cred_bytes = bastion_storage::secrets_repo::get_secret(
            db,
            secrets,
            bastion_core::HUB_NODE_ID,
            "webdav",
            secret_name,
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
        let credentials = bastion_targets::WebdavCredentials::from_json(&cred_bytes)?;

        raw_tree_webdav_direct_upload = Some(backup::filesystem::RawTreeWebdavDirectUploadConfig {
            handle: tokio::runtime::Handle::current(),
            base_url: base_url.clone(),
            credentials,
            max_attempts: 3,
            resume_by_size: true,
        });
    }
    let using_webdav_raw_tree_direct_upload = raw_tree_webdav_direct_upload.is_some();

    // For raw_tree_v1 + local_dir targets, avoid duplicating the staged `data/` tree by linking the
    // staging `data/` dir to the final target run dir (best-effort; falls back to normal staging).
    let mut direct_target_run_dir: Option<std::path::PathBuf> = None;
    if artifact_format == bastion_core::manifest::ArtifactFormatV1::RawTreeV1
        && let job_spec::TargetV1::LocalDir { base_dir, .. } = &target
    {
        let target_run_dir = std::path::Path::new(base_dir)
            .join(&job_id)
            .join(&run_id_owned);
        let target_data_dir = target_run_dir.join("data");
        let stage_data_dir = backup::stage_dir(&data_dir, &run_id_owned).join("data");

        if let Err(error) = std::fs::create_dir_all(&target_data_dir) {
            let fields = serde_json::json!({
                "error": error.to_string(),
                "target_data_dir": target_data_dir.to_string_lossy().to_string(),
            });
            let _ = run_events::append_and_broadcast(
                db,
                run_events_bus,
                run_id,
                "warn",
                "direct_data_path_unavailable",
                "raw-tree direct data path unavailable",
                Some(fields),
            )
            .await;
        } else {
            let _ = std::fs::create_dir_all(backup::stage_dir(&data_dir, &run_id_owned));
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
                    let _ = run_events::append_and_broadcast(
                        db,
                        run_events_bus,
                        run_id,
                        "info",
                        "direct_data_path_ready",
                        "raw-tree direct data path ready",
                        Some(fields),
                    )
                    .await;
                }
                Err(error) => {
                    let fields = serde_json::json!({
                        "error": error.to_string(),
                        "stage_data_dir": stage_data_dir.to_string_lossy().to_string(),
                        "target_data_dir": target_data_dir.to_string_lossy().to_string(),
                    });
                    let _ = run_events::append_and_broadcast(
                        db,
                        run_events_bus,
                        run_id,
                        "warn",
                        "direct_data_path_unavailable",
                        "raw-tree direct data path unavailable",
                        Some(fields),
                    )
                    .await;
                }
            }
        }
    }

    let read_mapping_for_build = read_mapping.clone();
    let progress_tx_build = progress_tx.clone();
    let raw_tree_webdav_direct_upload_for_build = raw_tree_webdav_direct_upload.clone();
    let job_id_for_build = job_id.clone();
    let run_id_for_build = run_id_owned.clone();
    let build_res = tokio::task::spawn_blocking(move || {
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
            &job_id_for_build,
            &run_id_for_build,
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
    })
    .await?;

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
            let _ = run_events::append_and_broadcast(
                db,
                run_events_bus,
                run_id,
                "warn",
                "snapshot_cleanup_failed",
                "snapshot cleanup failed",
                Some(fields),
            )
            .await;
        }
    }

    if build_res.is_err() {
        if using_webdav_raw_tree_direct_upload
            && let Some(cfg) = raw_tree_webdav_direct_upload.as_ref()
        {
            let _ = bastion_targets::webdav::cleanup_incomplete_run(
                &cfg.base_url,
                cfg.credentials.clone(),
                &job_id,
                &run_id_owned,
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

    let consistency_total = build.consistency.total();
    let consistency_failed =
        consistency_policy.should_fail(consistency_total, consistency_fail_threshold);

    if consistency_policy.should_emit_warnings() && consistency_total > 0 {
        let fields = serde_json::to_value(&build.consistency)?;
        let _ = run_events::append_and_broadcast(
            db,
            run_events_bus,
            run_id,
            "warn",
            "source_consistency",
            "source consistency warnings",
            Some(fields),
        )
        .await;
    }

    let source_total = build.source_total;
    let raw_tree_stats = build.raw_tree_stats;
    let issues = build.issues;
    let consistency = if consistency_policy == job_spec::ConsistencyPolicyV1::Ignore {
        Default::default()
    } else {
        build.consistency
    };
    let artifacts = build.artifacts;

    let parts_bytes: u64 = artifacts.parts.iter().map(|p| p.size).sum();
    let entries_size = std::fs::metadata(&artifacts.entries_index_path)?.len();
    let manifest_size = std::fs::metadata(&artifacts.manifest_path)?.len();
    let complete_size = std::fs::metadata(&artifacts.complete_path)?.len();
    let raw_tree_data_bytes = raw_tree_stats.map(|s| s.data_bytes).unwrap_or(0);
    let raw_tree_data_bytes_for_transfer = if using_webdav_raw_tree_direct_upload {
        0
    } else {
        raw_tree_data_bytes
    };
    let transfer_total_bytes = parts_bytes
        .saturating_add(entries_size)
        .saturating_add(manifest_size)
        .saturating_add(complete_size)
        .saturating_add(raw_tree_data_bytes_for_transfer);

    if consistency_failed && !upload_on_consistency_failure {
        let target_summary = serde_json::json!({
            "type": match target {
                job_spec::TargetV1::Webdav { .. } => "webdav",
                job_spec::TargetV1::LocalDir { .. } => "local_dir",
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
            "artifact_format": pipeline.format,
            "entries_count": artifacts.entries_count,
            "parts": artifacts.parts.len(),
            "metrics": metrics,
            "filesystem": {
                "warnings_total": issues.warnings_total,
                "errors_total": issues.errors_total,
                "snapshot": snapshot_summary.clone(),
                "consistency": consistency,
            },
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
    let target_summary = match super::super::target_store::store_run_artifacts_to_target(
        db,
        secrets,
        &job.id,
        run_id,
        &target,
        &artifacts,
        Some(upload_cb),
    )
    .await
    {
        Ok(v) => v,
        Err(error) => {
            if using_webdav_raw_tree_direct_upload
                && let Some(cfg) = raw_tree_webdav_direct_upload.as_ref()
            {
                let _ = bastion_targets::webdav::cleanup_incomplete_run(
                    &cfg.base_url,
                    cfg.credentials.clone(),
                    &job_id,
                    &run_id_owned,
                )
                .await;
            }
            return Err(error);
        }
    };

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

    let summary = serde_json::json!({
        "target": target_summary,
        "artifact_format": pipeline.format,
        "entries_count": artifacts.entries_count,
        "parts": artifacts.parts.len(),
        "metrics": metrics,
        "filesystem": {
            "warnings_total": issues.warnings_total,
            "errors_total": issues.errors_total,
            "snapshot": snapshot_summary.clone(),
            "consistency": consistency,
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
