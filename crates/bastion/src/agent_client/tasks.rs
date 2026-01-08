use std::path::Path;

use futures_util::{Sink, SinkExt};
use tokio_tungstenite::tungstenite::Message;
use tracing::warn;

use bastion_core::agent_protocol::{
    AgentToHubMessageV1, EncryptionResolvedV1, JobSpecResolvedV1, PROTOCOL_VERSION,
};
use bastion_core::run_failure::RunFailedWithSummary;

use bastion_backup as backup;

use super::managed::save_task_result;
use super::targets::{store_artifacts_to_resolved_target, target_part_size_bytes};

pub(super) async fn handle_backup_task(
    data_dir: &Path,
    tx: &mut (impl Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    task_id: &str,
    task: bastion_core::agent_protocol::BackupRunTaskV1,
) -> Result<(), anyhow::Error> {
    let run_id = task.run_id.clone();
    let job_id = task.job_id.clone();
    let started_at = time::OffsetDateTime::from_unix_timestamp(task.started_at)
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc());

    send_run_event(tx, &run_id, "info", "start", "start", None).await?;

    let summary = match task.spec {
        JobSpecResolvedV1::Filesystem {
            pipeline,
            source,
            target,
            ..
        } => {
            send_run_event(tx, &run_id, "info", "packaging", "packaging", None).await?;
            let part_size = target_part_size_bytes(&target);
            let error_policy = source.error_policy;
            let encryption = match pipeline.encryption {
                EncryptionResolvedV1::None => backup::PayloadEncryption::None,
                EncryptionResolvedV1::AgeX25519 {
                    recipient,
                    key_name,
                } => backup::PayloadEncryption::AgeX25519 {
                    recipient,
                    key_name,
                },
            };
            let data_dir_buf = data_dir.to_path_buf();
            let job_id_clone = job_id.clone();
            let run_id_clone = run_id.clone();
            let build = tokio::task::spawn_blocking(move || {
                backup::filesystem::build_filesystem_run(
                    &data_dir_buf,
                    &job_id_clone,
                    &run_id_clone,
                    started_at,
                    &source,
                    &encryption,
                    part_size,
                )
            })
            .await??;

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
                send_run_event(
                    tx,
                    &run_id,
                    level,
                    "fs_issues",
                    "filesystem issues",
                    Some(fields),
                )
                .await?;
            }

            let issues = build.issues;
            let artifacts = build.artifacts;

            send_run_event(tx, &run_id, "info", "upload", "upload", None).await?;
            let target_summary =
                store_artifacts_to_resolved_target(&job_id, &run_id, &target, &artifacts).await?;

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

            if error_policy == bastion_core::job_spec::FsErrorPolicy::SkipFail
                && issues.errors_total > 0
            {
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

            summary
        }
        JobSpecResolvedV1::Sqlite {
            pipeline,
            source,
            target,
            ..
        } => {
            send_run_event(tx, &run_id, "info", "snapshot", "snapshot", None).await?;
            let sqlite_path = source.path.clone();
            let part_size = target_part_size_bytes(&target);

            let encryption = match pipeline.encryption {
                EncryptionResolvedV1::None => backup::PayloadEncryption::None,
                EncryptionResolvedV1::AgeX25519 {
                    recipient,
                    key_name,
                } => backup::PayloadEncryption::AgeX25519 {
                    recipient,
                    key_name,
                },
            };
            let data_dir_buf = data_dir.to_path_buf();
            let job_id_clone = job_id.clone();
            let run_id_clone = run_id.clone();
            let build = tokio::task::spawn_blocking(move || {
                backup::sqlite::build_sqlite_run(
                    &data_dir_buf,
                    &job_id_clone,
                    &run_id_clone,
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
                send_run_event(
                    tx,
                    &run_id,
                    if check.ok { "info" } else { "error" },
                    "integrity_check",
                    "integrity_check",
                    Some(data),
                )
                .await?;
                if !check.ok {
                    let first = check.lines.first().cloned().unwrap_or_default();
                    anyhow::bail!("sqlite integrity_check failed: {}", first);
                }
            }

            send_run_event(tx, &run_id, "info", "upload", "upload", None).await?;
            let target_summary =
                store_artifacts_to_resolved_target(&job_id, &run_id, &target, &build.artifacts)
                    .await?;
            let _ = tokio::fs::remove_dir_all(&build.artifacts.run_dir).await;

            serde_json::json!({
                "target": target_summary,
                "entries_count": build.artifacts.entries_count,
                "parts": build.artifacts.parts.len(),
                "sqlite": {
                    "path": sqlite_path,
                    "snapshot_name": build.snapshot_name,
                    "snapshot_size": build.snapshot_size,
                    "integrity_check": build.integrity_check.map(|check| serde_json::json!({
                        "ok": check.ok,
                        "truncated": check.truncated,
                        "lines": check.lines,
                    })),
                },
            })
        }
        JobSpecResolvedV1::Vaultwarden {
            pipeline,
            source,
            target,
            ..
        } => {
            send_run_event(tx, &run_id, "info", "snapshot", "snapshot", None).await?;
            let vw_data_dir = source.data_dir.clone();
            let part_size = target_part_size_bytes(&target);

            let encryption = match pipeline.encryption {
                EncryptionResolvedV1::None => backup::PayloadEncryption::None,
                EncryptionResolvedV1::AgeX25519 {
                    recipient,
                    key_name,
                } => backup::PayloadEncryption::AgeX25519 {
                    recipient,
                    key_name,
                },
            };
            let data_dir_buf = data_dir.to_path_buf();
            let job_id_clone = job_id.clone();
            let run_id_clone = run_id.clone();
            let artifacts = tokio::task::spawn_blocking(move || {
                backup::vaultwarden::build_vaultwarden_run(
                    &data_dir_buf,
                    &job_id_clone,
                    &run_id_clone,
                    started_at,
                    &source,
                    &encryption,
                    part_size,
                )
            })
            .await??;

            send_run_event(tx, &run_id, "info", "upload", "upload", None).await?;
            let target_summary =
                store_artifacts_to_resolved_target(&job_id, &run_id, &target, &artifacts).await?;
            let _ = tokio::fs::remove_dir_all(&artifacts.run_dir).await;

            serde_json::json!({
                "target": target_summary,
                "entries_count": artifacts.entries_count,
                "parts": artifacts.parts.len(),
                "vaultwarden": {
                    "data_dir": vw_data_dir,
                    "db": "db.sqlite3",
                }
            })
        }
    };

    send_run_event(tx, &run_id, "info", "complete", "complete", None).await?;

    let result = AgentToHubMessageV1::TaskResult {
        v: PROTOCOL_VERSION,
        task_id: task_id.to_string(),
        run_id: run_id.clone(),
        status: "success".to_string(),
        summary: Some(summary),
        error: None,
    };
    if let Err(error) = save_task_result(data_dir, &result) {
        warn!(task_id = %task_id, error = %error, "failed to persist task result");
    }
    tx.send(Message::Text(serde_json::to_string(&result)?.into()))
        .await?;
    Ok(())
}

async fn send_run_event(
    tx: &mut (impl Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    run_id: &str,
    level: &str,
    kind: &str,
    message: &str,
    fields: Option<serde_json::Value>,
) -> Result<(), anyhow::Error> {
    let msg = AgentToHubMessageV1::RunEvent {
        v: PROTOCOL_VERSION,
        run_id: run_id.to_string(),
        level: level.to_string(),
        kind: kind.to_string(),
        message: message.to_string(),
        fields,
    };
    tx.send(Message::Text(serde_json::to_string(&msg)?.into()))
        .await?;
    Ok(())
}
