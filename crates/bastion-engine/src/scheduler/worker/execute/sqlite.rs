use sqlx::SqlitePool;
use time::OffsetDateTime;

use bastion_core::job_spec;
use bastion_storage::jobs_repo;
use bastion_storage::secrets::SecretsCrypto;

use crate::run_events;
use crate::run_events_bus::RunEventsBus;

use bastion_backup as backup;
use bastion_backup::backup_encryption;

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
    let encryption = backup_encryption::ensure_payload_encryption(db, secrets, &pipeline).await?;
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

    run_events::append_and_broadcast(db, run_events_bus, run_id, "info", "upload", "upload", None)
        .await?;
    let target_summary = super::super::target_store::store_run_artifacts_to_target(
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
