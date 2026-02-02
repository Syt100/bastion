use std::path::{Path, PathBuf};
use std::time::Duration;

use futures_util::{Sink, SinkExt};
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;
use tracing::warn;
use uuid::Uuid;

use bastion_backup::restore::{self, PayloadDecryption};
use bastion_core::agent_protocol::{
    AgentToHubMessageV1, ArtifactStreamOpenV1, OperationEventV1, OperationResultV1,
    PROTOCOL_VERSION, RestoreDestinationV1, RestoreTaskV1,
};
use bastion_core::backup_format::MANIFEST_NAME;
use bastion_core::manifest::ManifestV1;
use bastion_core::progress::{
    PROGRESS_SNAPSHOT_EVENT_KIND_V1, ProgressKindV1, ProgressSnapshotV1, ProgressUnitsV1,
};

use super::hub_stream::{HubStreamManager, HubStreamReader};
use super::managed::save_task_result;

const HUB_STREAM_OPEN_TIMEOUT: Duration = Duration::from_secs(30);
const HUB_STREAM_PULL_TIMEOUT: Duration = Duration::from_secs(30);
const HUB_STREAM_MAX_BYTES: u32 = 1024 * 1024;

struct OpProgressBuilder {
    last_ts: Option<i64>,
    last_done_bytes: u64,
}

impl OpProgressBuilder {
    fn new() -> Self {
        Self {
            last_ts: None,
            last_done_bytes: 0,
        }
    }

    fn snapshot(&mut self, done: ProgressUnitsV1) -> ProgressSnapshotV1 {
        let stage: &'static str = "restore";
        let now_ts = time::OffsetDateTime::now_utc().unix_timestamp();

        let (rate_bps, eta_seconds) = match self.last_ts {
            None => (None, None),
            Some(prev_ts) => {
                let dt = now_ts.saturating_sub(prev_ts);
                let delta = done.bytes.saturating_sub(self.last_done_bytes);
                let rate = if dt > 0 && delta > 0 {
                    Some(delta.saturating_div(dt as u64).max(1))
                } else {
                    None
                };
                (rate, None)
            }
        };

        self.last_ts = Some(now_ts);
        self.last_done_bytes = done.bytes;

        ProgressSnapshotV1 {
            v: 1,
            kind: ProgressKindV1::Restore,
            stage: stage.to_string(),
            ts: now_ts,
            done,
            total: None,
            rate_bps,
            eta_seconds,
            detail: None,
        }
    }
}

pub(super) async fn handle_restore_task(
    data_dir: &Path,
    tx: &mut (impl Sink<Message, Error = tungstenite::Error> + Unpin),
    hub_streams: &HubStreamManager,
    task_id: &str,
    task: RestoreTaskV1,
) -> Result<(), anyhow::Error> {
    let op_id = task.op_id.trim().to_string();
    let run_id = task.run_id.trim().to_string();
    if op_id.is_empty() {
        anyhow::bail!("restore task op_id is required");
    }
    if run_id.is_empty() {
        anyhow::bail!("restore task run_id is required");
    }

    let destination = task
        .destination
        .clone()
        .unwrap_or_else(|| RestoreDestinationV1::LocalFs {
            directory: task.destination_dir.clone(),
        });

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
        "age" => {
            let key_name = manifest
                .pipeline
                .encryption_key
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| anyhow::anyhow!("missing manifest.pipeline.encryption_key"))?;

            let identity = super::managed::load_managed_backup_age_identity(data_dir, key_name)?
                .ok_or_else(|| anyhow::anyhow!("missing backup age identity: {}", key_name))?;

            send_op_event(
                tx,
                &op_id,
                "info",
                "age_identity",
                "age_identity",
                Some(serde_json::json!({ "key_name": key_name })),
            )
            .await?;

            PayloadDecryption::AgeX25519 { identity }
        }
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

    let data_dir_owned = data_dir.to_path_buf();
    let restore_staging_root = data_dir_owned.join("restore_staging").join(op_id.clone());
    let restore_staging_dir = restore_staging_root.join("webdav_sink");
    let restore_staging_root_cleanup = restore_staging_root.clone();
    let op_id_for_restore = op_id.clone();
    let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel::<ProgressUnitsV1>(8);
    let mut progress = OpProgressBuilder::new();

    let mut restore_handle = tokio::task::spawn_blocking(move || {
        let on_progress = |done: ProgressUnitsV1| {
            // RestoreEngine progress is already throttled; blocking send is OK here.
            let _ = progress_tx.blocking_send(done);
        };

        match destination {
            RestoreDestinationV1::LocalFs { directory } => {
                let directory = directory.trim().to_string();
                if directory.is_empty() {
                    anyhow::bail!("restore task destination.directory is required");
                }
                let dest = PathBuf::from(&directory);
                restore::restore_to_local_fs(
                    Box::new(reader),
                    dest.clone(),
                    conflict,
                    decryption,
                    selection.as_ref(),
                    Some(&on_progress),
                )?;
                Ok::<_, anyhow::Error>(serde_json::json!({
                    "destination": { "type": "local_fs", "directory": dest.to_string_lossy().to_string() },
                    "conflict_policy": conflict.as_str(),
                }))
            }
            RestoreDestinationV1::Webdav {
                base_url,
                secret_name,
                prefix,
            } => {
                let base_url = base_url.trim().to_string();
                let secret_name = secret_name.trim().to_string();
                let prefix = prefix.trim().to_string();
                if base_url.is_empty() {
                    anyhow::bail!("restore task destination.base_url is required");
                }
                if secret_name.is_empty() {
                    anyhow::bail!("restore task destination.secret_name is required");
                }
                if prefix.is_empty() {
                    anyhow::bail!("restore task destination.prefix is required");
                }

                let credentials =
                    super::managed::load_managed_webdav_credentials(&data_dir_owned, &secret_name)?
                        .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {}", secret_name))?;

                restore::restore_to_webdav(
                    Box::new(reader),
                    restore::WebdavRestoreTarget {
                        base_url: &base_url,
                        credentials,
                        prefix: &prefix,
                    },
                    &op_id_for_restore,
                    conflict,
                    decryption,
                    selection.as_ref(),
                    restore_staging_dir,
                    Some(&on_progress),
                )?;
                Ok::<_, anyhow::Error>(serde_json::json!({
                    "destination": { "type": "webdav", "base_url": base_url, "prefix": prefix },
                    "conflict_policy": conflict.as_str(),
                }))
            }
        }
    });

    let summary = loop {
        tokio::select! {
            res = &mut restore_handle => break res??,
            maybe_done = progress_rx.recv() => {
                if let Some(done) = maybe_done {
                    send_op_progress_snapshot(tx, &op_id, progress.snapshot(done)).await?;
                }
            }
        }
    };

    // Best-effort cleanup for any staging created by the restore.
    let _ = tokio::fs::remove_dir_all(restore_staging_root_cleanup).await;

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

async fn send_op_progress_snapshot(
    tx: &mut (impl Sink<Message, Error = tungstenite::Error> + Unpin),
    op_id: &str,
    snapshot: ProgressSnapshotV1,
) -> Result<(), anyhow::Error> {
    let stage = snapshot.stage.clone();
    let fields = serde_json::to_value(snapshot)?;
    send_op_event(
        tx,
        op_id,
        "info",
        PROGRESS_SNAPSHOT_EVENT_KIND_V1,
        &stage,
        Some(fields),
    )
    .await
}

#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use futures_util::Sink;
    use tokio_tungstenite::tungstenite;
    use tokio_tungstenite::tungstenite::Message;

    use bastion_core::agent_protocol::PROTOCOL_VERSION;
    use bastion_core::progress::{ProgressKindV1, ProgressSnapshotV1, ProgressUnitsV1};

    use super::{send_op_event, send_op_progress_snapshot};

    #[derive(Default)]
    struct VecSink {
        messages: Vec<Message>,
    }

    impl Sink<Message> for VecSink {
        type Error = tungstenite::Error;

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
    async fn send_op_event_serializes_operation_event() {
        let mut sink = VecSink::default();
        send_op_event(
            &mut sink,
            "op1",
            "info",
            "start",
            "start",
            Some(serde_json::json!({ "k": 1 })),
        )
        .await
        .unwrap();

        assert_eq!(sink.messages.len(), 1);
        let Message::Text(text) = &sink.messages[0] else {
            panic!("expected text message");
        };

        let v: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(v["type"], "operation_event");
        assert_eq!(v["v"], PROTOCOL_VERSION);
        assert_eq!(v["event"]["op_id"], "op1");
        assert_eq!(v["event"]["level"], "info");
        assert_eq!(v["event"]["kind"], "start");
        assert_eq!(v["event"]["message"], "start");
        assert_eq!(v["event"]["fields"]["k"], 1);
    }

    #[tokio::test]
    async fn send_op_progress_snapshot_wraps_snapshot_as_operation_event() {
        let mut sink = VecSink::default();
        let snapshot = ProgressSnapshotV1 {
            v: 1,
            kind: ProgressKindV1::Restore,
            stage: "restore".to_string(),
            ts: 123,
            done: ProgressUnitsV1 {
                bytes: 10,
                files: 2,
                dirs: 0,
            },
            total: None,
            rate_bps: None,
            eta_seconds: None,
            detail: Some(serde_json::json!({ "x": true })),
        };

        send_op_progress_snapshot(&mut sink, "op1", snapshot)
            .await
            .unwrap();

        assert_eq!(sink.messages.len(), 1);
        let Message::Text(text) = &sink.messages[0] else {
            panic!("expected text message");
        };

        let v: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(v["type"], "operation_event");
        assert_eq!(v["event"]["op_id"], "op1");
        assert_eq!(
            v["event"]["kind"],
            bastion_core::progress::PROGRESS_SNAPSHOT_EVENT_KIND_V1
        );
        assert_eq!(v["event"]["message"], "restore");
        assert_eq!(v["event"]["fields"]["kind"], "restore");
        assert_eq!(v["event"]["fields"]["done"]["bytes"], 10);
        assert_eq!(v["event"]["fields"]["done"]["files"], 2);
        assert_eq!(v["event"]["fields"]["detail"]["x"], true);
    }
}
