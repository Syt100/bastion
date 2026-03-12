use std::time::Instant;

use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;

use bastion_core::job_spec;
use bastion_core::progress::{ProgressKindV1, ProgressUnitsV1};
use bastion_core::run_failure::RunFailedWithSummary;
use bastion_storage::jobs_repo;
use bastion_storage::secrets::SecretsCrypto;

use crate::error_envelope::{insert_error_envelope, with_context_param};
use crate::run_events;
use crate::run_events_bus::RunEventsBus;

use bastion_backup as backup;
use bastion_backup::backup_encryption;

use super::planner::plan_vaultwarden_execution;
use super::progress::{RUN_PROGRESS_MIN_INTERVAL, RunProgressUpdate, spawn_run_progress_writer};
use super::rolling_archive;
use super::{check_run_canceled, execute_stage_envelope};

fn vaultwarden_source_consistency_event_fields(
    consistency: &serde_json::Value,
) -> serde_json::Value {
    let mut fields = consistency.as_object().cloned().unwrap_or_default();
    let total = fields
        .values()
        .filter_map(|value| value.as_u64())
        .sum::<u64>();
    let envelope = with_context_param(
        execute_stage_envelope(
            "vaultwarden",
            "consistency_check",
            "packaging",
            "scheduler.execute.vaultwarden.source_consistency",
            "consistency",
            "diagnostics.hint.execute.source_consistency",
            "diagnostics.message.execute.source_consistency",
            "internal",
            false,
        ),
        "signal_total",
        total,
    );
    insert_error_envelope(&mut fields, envelope);
    serde_json::Value::Object(fields)
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn execute_vaultwarden_run(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    run_events_bus: &RunEventsBus,
    data_dir: &std::path::Path,
    job: &jobs_repo::Job,
    run_id: &str,
    started_at: OffsetDateTime,
    cancel_token: &CancellationToken,
    pipeline: job_spec::PipelineV1,
    source: job_spec::VaultwardenSource,
    target: job_spec::TargetV1,
) -> Result<serde_json::Value, anyhow::Error> {
    check_run_canceled(run_id, cancel_token)?;
    let progress_tx =
        spawn_run_progress_writer(db.clone(), run_id.to_string(), ProgressKindV1::Backup);
    let _ = progress_tx.send(Some(RunProgressUpdate {
        stage: "packaging",
        done: ProgressUnitsV1::default(),
        total: None,
        detail: None,
    }));

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
    let vw_data_dir = source.data_dir.clone();
    let consistency_policy = source.consistency_policy;
    let consistency_fail_threshold = source.consistency_fail_threshold.unwrap_or(0);
    let upload_on_consistency_failure = source.upload_on_consistency_failure.unwrap_or(false);
    let part_size = target.part_size_bytes();
    let artifact_format = pipeline.format.clone();
    let encryption = backup_encryption::ensure_payload_encryption(db, secrets, &pipeline).await?;

    let planned = plan_vaultwarden_execution(&pipeline, &source, &target)
        .map_err(|error| anyhow::anyhow!("execution planning failed: {error}"))?;
    let planner_fields = planned
        .plan
        .observability_fields(&planned.source_driver, &planned.target_driver);
    let planner_summary = planned
        .plan
        .summary_payload(&planned.source_driver, &planned.target_driver);
    run_events::append_and_broadcast(
        db,
        run_events_bus,
        run_id,
        "info",
        "planning",
        "planning",
        Some(planner_fields),
    )
    .await?;

    let (on_part_finished, parts_uploader) = if planned.plan.allow_rolling_upload {
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

    let build_res = tokio::task::spawn_blocking(move || {
        backup::vaultwarden::build_vaultwarden_run(
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
            on_part_finished,
        )
    })
    .await?;
    check_run_canceled(run_id, cancel_token)?;
    let uploader_res = rolling_archive::join_parts_uploader(parts_uploader).await;
    let build = match (build_res, uploader_res) {
        (Ok(build), Ok(())) => build,
        (Err(build_error), Ok(())) => return Err(build_error),
        (Ok(_), Err(upload_error)) => return Err(upload_error),
        (Err(build_error), Err(upload_error)) => {
            return Err(rolling_archive::merge_packaging_and_uploader_errors(
                build_error,
                upload_error,
            ));
        }
    };
    let consistency_total = build.consistency.total();
    let consistency_failed =
        consistency_policy.should_fail(consistency_total, consistency_fail_threshold);
    let consistency = if consistency_policy == job_spec::ConsistencyPolicyV1::Ignore {
        Default::default()
    } else {
        build.consistency
    };
    let artifacts = build.artifacts;

    if consistency_policy.should_emit_warnings() && consistency_total > 0 {
        let consistency_fields = serde_json::to_value(&consistency)?;
        let _ = run_events::append_and_broadcast(
            db,
            run_events_bus,
            run_id,
            "warn",
            "source_consistency",
            "source consistency warnings",
            Some(vaultwarden_source_consistency_event_fields(
                &consistency_fields,
            )),
        )
        .await;
    }

    let parts_bytes: u64 = artifacts.parts.iter().map(|p| p.size).sum();
    let entries_size = std::fs::metadata(&artifacts.entries_index_path)?.len();
    let manifest_size = std::fs::metadata(&artifacts.manifest_path)?.len();
    let complete_size = std::fs::metadata(&artifacts.complete_path)?.len();
    let transfer_total_bytes = parts_bytes
        .saturating_add(entries_size)
        .saturating_add(manifest_size)
        .saturating_add(complete_size);

    if consistency_failed && !upload_on_consistency_failure {
        let target_summary = serde_json::json!({
            "type": match target {
                job_spec::TargetV1::Webdav { .. } => "webdav",
                job_spec::TargetV1::LocalDir { .. } => "local_dir",
            }
        });

        let summary = serde_json::json!({
            "target": target_summary,
            "artifact_format": pipeline.format,
            "entries_count": artifacts.entries_count,
            "parts": artifacts.parts.len(),
            "metrics": {
                "transfer_total_bytes": transfer_total_bytes,
            },
            "vaultwarden": {
                "data_dir": vw_data_dir,
                "db": "db.sqlite3",
                "consistency": consistency,
            },
            "planner": planner_summary.clone(),
        });

        let _ = tokio::fs::remove_dir_all(&artifacts.run_dir).await;

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
        &artifacts,
        None,
        Some(upload_cb),
    )
    .await?;
    check_run_canceled(run_id, cancel_token)?;

    let _ = tokio::fs::remove_dir_all(&artifacts.run_dir).await;

    let summary = serde_json::json!({
        "target": target_summary,
        "artifact_format": pipeline.format,
        "entries_count": artifacts.entries_count,
        "parts": artifacts.parts.len(),
        "metrics": {
            "transfer_total_bytes": transfer_total_bytes,
        },
        "vaultwarden": {
            "data_dir": vw_data_dir,
            "db": "db.sqlite3",
            "consistency": consistency,
        },
        "planner": planner_summary,
    });

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
    use super::vaultwarden_source_consistency_event_fields;

    #[test]
    fn source_consistency_fields_include_error_envelope() {
        let fields = vaultwarden_source_consistency_event_fields(&serde_json::json!({
            "changed_files": 3,
            "changed_dirs": 1,
        }));
        let obj = fields.as_object().expect("object");
        assert_eq!(
            obj.get("error_envelope")
                .and_then(|value| value.get("code"))
                .and_then(|value| value.as_str()),
            Some("scheduler.execute.vaultwarden.source_consistency")
        );
    }
}
