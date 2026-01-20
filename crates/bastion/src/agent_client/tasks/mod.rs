mod filesystem;
mod sqlite;
mod vaultwarden;

use std::path::Path;

use futures_util::{Sink, SinkExt};
use tokio_tungstenite::tungstenite::Message;
use tracing::warn;

use bastion_backup as backup;
use bastion_core::agent_protocol::{
    AgentToHubMessageV1, BackupRunTaskV1, EncryptionResolvedV1, JobSpecResolvedV1, PROTOCOL_VERSION,
};
use bastion_core::progress::{PROGRESS_SNAPSHOT_EVENT_KIND_V1, ProgressSnapshotV1};

use super::managed::save_task_result;

struct TaskContext<'a> {
    data_dir: &'a Path,
    run_id: &'a str,
    job_id: &'a str,
    started_at: time::OffsetDateTime,
}

pub(super) async fn handle_backup_task(
    data_dir: &Path,
    tx: &mut (impl Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    task_id: &str,
    task: BackupRunTaskV1,
) -> Result<(), anyhow::Error> {
    let run_id = task.run_id.clone();
    let job_id = task.job_id.clone();
    let started_at = time::OffsetDateTime::from_unix_timestamp(task.started_at)
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc());

    send_run_event(tx, &run_id, "info", "start", "start", None).await?;

    let ctx = TaskContext {
        data_dir,
        run_id: &run_id,
        job_id: &job_id,
        started_at,
    };

    let summary = match task.spec {
        JobSpecResolvedV1::Filesystem {
            pipeline,
            source,
            target,
            ..
        } => filesystem::run_filesystem_backup(tx, &ctx, pipeline, source, target).await?,
        JobSpecResolvedV1::Sqlite {
            pipeline,
            source,
            target,
            ..
        } => sqlite::run_sqlite_backup(tx, &ctx, pipeline, source, target).await?,
        JobSpecResolvedV1::Vaultwarden {
            pipeline,
            source,
            target,
            ..
        } => vaultwarden::run_vaultwarden_backup(tx, &ctx, pipeline, source, target).await?,
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

fn payload_encryption(encryption: EncryptionResolvedV1) -> backup::PayloadEncryption {
    match encryption {
        EncryptionResolvedV1::None => backup::PayloadEncryption::None,
        EncryptionResolvedV1::AgeX25519 {
            recipient,
            key_name,
        } => backup::PayloadEncryption::AgeX25519 {
            recipient,
            key_name,
        },
    }
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

async fn send_run_progress_snapshot(
    tx: &mut (impl Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    run_id: &str,
    snapshot: ProgressSnapshotV1,
) -> Result<(), anyhow::Error> {
    let stage = snapshot.stage.clone();
    let fields = serde_json::to_value(snapshot)?;
    send_run_event(
        tx,
        run_id,
        "info",
        PROGRESS_SNAPSHOT_EVENT_KIND_V1,
        &stage,
        Some(fields),
    )
    .await
}
