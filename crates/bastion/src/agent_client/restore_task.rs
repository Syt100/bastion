use std::path::{Path, PathBuf};
use std::time::Duration;

use futures_util::{Sink, SinkExt};
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;
use tracing::warn;
use uuid::Uuid;

use bastion_backup::restore::{self, PayloadDecryption};
use bastion_core::backup_format::MANIFEST_NAME;
use bastion_core::manifest::ManifestV1;
use bastion_core::agent_protocol::{
    AgentToHubMessageV1, ArtifactStreamOpenV1, OperationEventV1, OperationResultV1, RestoreTaskV1,
    PROTOCOL_VERSION,
};

use super::hub_stream::{HubStreamManager, HubStreamReader};
use super::managed::save_task_result;

const HUB_STREAM_OPEN_TIMEOUT: Duration = Duration::from_secs(30);
const HUB_STREAM_PULL_TIMEOUT: Duration = Duration::from_secs(30);
const HUB_STREAM_MAX_BYTES: u32 = 1024 * 1024;

pub(super) async fn handle_restore_task(
    data_dir: &Path,
    tx: &mut (impl Sink<Message, Error = tungstenite::Error> + Unpin),
    hub_streams: &HubStreamManager,
    task_id: &str,
    task: RestoreTaskV1,
) -> Result<(), anyhow::Error> {
    let op_id = task.op_id.trim().to_string();
    let run_id = task.run_id.trim().to_string();
    let destination_dir = task.destination_dir.trim().to_string();
    if op_id.is_empty() {
        anyhow::bail!("restore task op_id is required");
    }
    if run_id.is_empty() {
        anyhow::bail!("restore task run_id is required");
    }
    if destination_dir.is_empty() {
        anyhow::bail!("restore task destination_dir is required");
    }

    send_op_event(tx, &op_id, "info", "start", "start", None).await?;

    let manifest_bytes = hub_streams
        .read_bytes(
            &op_id,
            &run_id,
            MANIFEST_NAME,
            HUB_STREAM_OPEN_TIMEOUT,
            HUB_STREAM_PULL_TIMEOUT,
            HUB_STREAM_MAX_BYTES,
        )
        .await?;
    let manifest = serde_json::from_slice::<ManifestV1>(&manifest_bytes)?;

    send_op_event(
        tx,
        &op_id,
        "info",
        "manifest",
        "manifest",
        Some(serde_json::json!({
            "artifacts": manifest.artifacts.len(),
            "entries_count": manifest.entry_index.count,
            "encryption": manifest.pipeline.encryption,
        })),
    )
    .await?;

    let decryption = match manifest.pipeline.encryption.as_str() {
        "none" => PayloadDecryption::None,
        // TODO(spec): implement backup age identity distribution + local agent key storage.
        "age" => anyhow::bail!("encrypted restore on agent is not supported yet"),
        other => anyhow::bail!("unsupported manifest.pipeline.encryption: {other}"),
    };

    let conflict = task
        .conflict_policy
        .parse::<restore::ConflictPolicy>()
        .map_err(|_| anyhow::anyhow!("invalid conflict policy"))?;
    let selection = task.selection.map(|s| restore::RestoreSelection {
        files: s.files,
        dirs: s.dirs,
    });

    // Open a payload stream and run the restore in a blocking task (tar/zstd + filesystem writes).
    let payload_stream_id = Uuid::new_v4();
    let res = hub_streams
        .open(
            ArtifactStreamOpenV1 {
                stream_id: payload_stream_id.to_string(),
                op_id: op_id.clone(),
                run_id: run_id.clone(),
                artifact: "payload".to_string(),
                path: None,
            },
            HUB_STREAM_OPEN_TIMEOUT,
        )
        .await?;
    if let Some(error) = res.error.as_deref()
        && !error.trim().is_empty()
    {
        anyhow::bail!("hub stream open failed: {error}");
    }

    let handle = tokio::runtime::Handle::current();
    let reader = HubStreamReader::new(
        handle.clone(),
        hub_streams.clone(),
        payload_stream_id,
        HUB_STREAM_MAX_BYTES,
        HUB_STREAM_PULL_TIMEOUT,
    );

    let dest = PathBuf::from(&destination_dir);
    let summary = tokio::task::spawn_blocking(move || {
        restore::restore_to_local_fs(
            Box::new(reader),
            dest.clone(),
            conflict,
            decryption,
            selection.as_ref(),
        )?;
        Ok::<_, anyhow::Error>(serde_json::json!({
            "destination_dir": dest.to_string_lossy().to_string(),
            "conflict_policy": conflict.as_str(),
        }))
    })
    .await??;

    send_op_event(tx, &op_id, "info", "complete", "complete", None).await?;

    let result = AgentToHubMessageV1::OperationResult {
        v: PROTOCOL_VERSION,
        result: OperationResultV1 {
            op_id: op_id.clone(),
            status: "success".to_string(),
            summary: Some(summary),
            error: None,
        },
    };
    if let Err(error) = save_task_result(data_dir, &result) {
        warn!(task_id = %task_id, error = %error, "failed to persist restore result");
    }
    tx.send(Message::Text(serde_json::to_string(&result)?.into()))
        .await?;

    Ok(())
}

async fn send_op_event(
    tx: &mut (impl Sink<Message, Error = tungstenite::Error> + Unpin),
    op_id: &str,
    level: &str,
    kind: &str,
    message: &str,
    fields: Option<serde_json::Value>,
) -> Result<(), anyhow::Error> {
    let msg = AgentToHubMessageV1::OperationEvent {
        v: PROTOCOL_VERSION,
        event: OperationEventV1 {
            op_id: op_id.to_string(),
            level: level.to_string(),
            kind: kind.to_string(),
            message: message.to_string(),
            fields,
        },
    };
    tx.send(Message::Text(serde_json::to_string(&msg)?.into()))
        .await?;
    Ok(())
}

