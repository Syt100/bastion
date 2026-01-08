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

pub(super) struct ExecuteRunArgs<'a> {
    pub(super) db: &'a SqlitePool,
    pub(super) secrets: &'a SecretsCrypto,
    pub(super) run_events_bus: &'a RunEventsBus,
    pub(super) data_dir: &'a std::path::Path,
    pub(super) job: &'a jobs_repo::Job,
    pub(super) run_id: &'a str,
    pub(super) started_at: OffsetDateTime,
    pub(super) spec: job_spec::JobSpecV1,
}

pub(super) async fn execute_run(
    args: ExecuteRunArgs<'_>,
) -> Result<serde_json::Value, anyhow::Error> {
    let ExecuteRunArgs {
        db,
        secrets,
        run_events_bus,
        data_dir,
        job,
        run_id,
        started_at,
        spec,
    } = args;
    match spec {
        job_spec::JobSpecV1::Filesystem {
            pipeline,
            source,
            target,
            ..
        } => {
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
            let encryption =
                backup_encryption::ensure_payload_encryption(db, secrets, &pipeline).await?;
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

            run_events::append_and_broadcast(
                db,
                run_events_bus,
                run_id,
                "info",
                "upload",
                "upload",
                None,
            )
            .await?;
            let target_summary = super::target_store::store_run_artifacts_to_target(
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
        job_spec::JobSpecV1::Sqlite {
            pipeline,
            source,
            target,
            ..
        } => {
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
            let encryption =
                backup_encryption::ensure_payload_encryption(db, secrets, &pipeline).await?;
            let build = tokio::task::spawn_blocking(move || {
                backup::sqlite::build_sqlite_run(
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

            run_events::append_and_broadcast(
                db,
                run_events_bus,
                run_id,
                "info",
                "upload",
                "upload",
                None,
            )
            .await?;
            let target_summary = super::target_store::store_run_artifacts_to_target(
                db,
                secrets,
                &job.id,
                run_id,
                &target,
                &build.artifacts,
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
        job_spec::JobSpecV1::Vaultwarden {
            pipeline,
            source,
            target,
            ..
        } => {
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
            let part_size = target.part_size_bytes();
            let encryption =
                backup_encryption::ensure_payload_encryption(db, secrets, &pipeline).await?;
            let artifacts = tokio::task::spawn_blocking(move || {
                backup::vaultwarden::build_vaultwarden_run(
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

            run_events::append_and_broadcast(
                db,
                run_events_bus,
                run_id,
                "info",
                "upload",
                "upload",
                None,
            )
            .await?;
            let target_summary = super::target_store::store_run_artifacts_to_target(
                db, secrets, &job.id, run_id, &target, &artifacts,
            )
            .await?;

            let _ = tokio::fs::remove_dir_all(&artifacts.run_dir).await;

            Ok(serde_json::json!({
                "target": target_summary,
                "entries_count": artifacts.entries_count,
                "parts": artifacts.parts.len(),
                "vaultwarden": {
                    "data_dir": vw_data_dir,
                    "db": "db.sqlite3",
                }
            }))
        }
    }
}
