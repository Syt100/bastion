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

type ArchivePartFinishedHook = Box<dyn Fn(backup::LocalArtifact) -> std::io::Result<()> + Send>;
type ArchivePartUploadHandle = tokio::task::JoinHandle<Result<(), anyhow::Error>>;
type ArchivePartUploader = (
    Option<ArchivePartFinishedHook>,
    Option<ArchivePartUploadHandle>,
);

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

fn prepare_archive_part_uploader(
    target: &bastion_core::agent_protocol::TargetResolvedV1,
    job_id: &str,
    run_id: &str,
    artifact_format: bastion_core::manifest::ArtifactFormatV1,
) -> ArchivePartUploader {
    if artifact_format != bastion_core::manifest::ArtifactFormatV1::ArchiveV1 {
        return (None, None);
    }

    let (tx, rx) = tokio::sync::mpsc::channel::<backup::LocalArtifact>(1);

    let handle: ArchivePartUploadHandle = match target {
        bastion_core::agent_protocol::TargetResolvedV1::Webdav {
            base_url,
            username,
            password,
            ..
        } => {
            let credentials = bastion_targets::WebdavCredentials {
                username: username.clone(),
                password: password.clone(),
            };
            let base_url = base_url.to_string();
            let job_id = job_id.to_string();
            let run_id = run_id.to_string();
            tokio::spawn(async move {
                bastion_targets::webdav::store_run_parts_rolling(
                    &base_url,
                    credentials,
                    &job_id,
                    &run_id,
                    rx,
                )
                .await
                .map(|_| ())
            })
        }
        bastion_core::agent_protocol::TargetResolvedV1::LocalDir { base_dir, .. } => {
            let base_dir = base_dir.to_string();
            let job_id = job_id.to_string();
            let run_id = run_id.to_string();
            tokio::task::spawn_blocking(move || {
                bastion_targets::local_dir::store_run_parts_rolling(
                    std::path::Path::new(&base_dir),
                    &job_id,
                    &run_id,
                    rx,
                )
                .map(|_| ())
            })
        }
    };

    let on_part_finished: ArchivePartFinishedHook = Box::new(move |part| {
        tx.blocking_send(part)
            .map_err(|_| std::io::Error::other("rolling uploader dropped"))?;
        Ok(())
    });

    (Some(on_part_finished), Some(handle))
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

#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use futures_util::Sink;

    use super::*;

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

    #[test]
    fn payload_encryption_maps_protocol_variants() {
        assert!(matches!(
            payload_encryption(EncryptionResolvedV1::None),
            backup::PayloadEncryption::None
        ));

        let enc = payload_encryption(EncryptionResolvedV1::AgeX25519 {
            recipient: "recipient".to_string(),
            key_name: "key_name".to_string(),
        });
        let backup::PayloadEncryption::AgeX25519 {
            recipient,
            key_name,
        } = enc
        else {
            panic!("expected AgeX25519 payload encryption");
        };
        assert_eq!(recipient, "recipient");
        assert_eq!(key_name, "key_name");
    }

    #[test]
    fn prepare_archive_part_uploader_skips_non_archive_formats() {
        let target = bastion_core::agent_protocol::TargetResolvedV1::LocalDir {
            base_dir: "/tmp".to_string(),
            part_size_bytes: 1024,
        };
        let (hook, handle) = prepare_archive_part_uploader(
            &target,
            "job_id",
            "run_id",
            bastion_core::manifest::ArtifactFormatV1::RawTreeV1,
        );
        assert!(hook.is_none());
        assert!(handle.is_none());
    }

    #[tokio::test]
    async fn prepare_archive_part_uploader_local_dir_uploads_and_deletes_parts()
    -> Result<(), anyhow::Error> {
        let base_dir = tempfile::tempdir()?;
        let stage_dir = tempfile::tempdir()?;

        let local_part_path = stage_dir.path().join("payload.part000001");
        let local_part_bytes = b"hello world";
        std::fs::write(&local_part_path, local_part_bytes)?;

        let target = bastion_core::agent_protocol::TargetResolvedV1::LocalDir {
            base_dir: base_dir.path().to_string_lossy().to_string(),
            part_size_bytes: 1024,
        };

        let (hook, handle) = prepare_archive_part_uploader(
            &target,
            "job_id",
            "run_id",
            bastion_core::manifest::ArtifactFormatV1::ArchiveV1,
        );
        let hook = hook.expect("hook");
        let handle = handle.expect("handle");

        let artifact = backup::LocalArtifact {
            name: "payload.part000001".to_string(),
            path: local_part_path.clone(),
            size: local_part_bytes.len() as u64,
            hash_alg: bastion_core::manifest::HashAlgorithm::Blake3,
            hash: "dummy".to_string(),
        };

        // The hook uses blocking_send(), so run it from a blocking thread.
        tokio::task::spawn_blocking(move || hook(artifact)).await??;
        handle.await??;

        let target_part_path = base_dir
            .path()
            .join("job_id")
            .join("run_id")
            .join("payload.part000001");

        assert_eq!(std::fs::read(target_part_path)?, local_part_bytes);
        assert!(!local_part_path.exists());
        Ok(())
    }

    #[tokio::test]
    async fn send_run_event_emits_agent_protocol_run_event() -> Result<(), anyhow::Error> {
        let mut sink = RecordingSink::default();
        send_run_event(
            &mut sink,
            "run_id",
            "info",
            "kind",
            "message",
            Some(serde_json::json!({"hello": "world"})),
        )
        .await?;

        assert_eq!(sink.messages.len(), 1);
        let Message::Text(text) = &sink.messages[0] else {
            anyhow::bail!("expected text frame");
        };

        let msg: AgentToHubMessageV1 = serde_json::from_str(text.as_ref())?;
        match msg {
            AgentToHubMessageV1::RunEvent {
                v,
                run_id,
                level,
                kind,
                message,
                fields,
            } => {
                assert_eq!(v, PROTOCOL_VERSION);
                assert_eq!(run_id, "run_id");
                assert_eq!(level, "info");
                assert_eq!(kind, "kind");
                assert_eq!(message, "message");
                assert_eq!(fields, Some(serde_json::json!({"hello": "world"})));
            }
            _ => anyhow::bail!("expected RunEvent"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn send_run_progress_snapshot_emits_progress_snapshot_run_event()
    -> Result<(), anyhow::Error> {
        let mut sink = RecordingSink::default();
        let snapshot = ProgressSnapshotV1 {
            v: 1,
            kind: bastion_core::progress::ProgressKindV1::Backup,
            stage: "packaging".to_string(),
            ts: 123,
            done: bastion_core::progress::ProgressUnitsV1::default(),
            total: None,
            rate_bps: None,
            eta_seconds: None,
            detail: Some(serde_json::json!({"x": 1})),
        };
        let expected_fields = serde_json::to_value(&snapshot)?;

        send_run_progress_snapshot(&mut sink, "run_id", snapshot).await?;

        assert_eq!(sink.messages.len(), 1);
        let Message::Text(text) = &sink.messages[0] else {
            anyhow::bail!("expected text frame");
        };

        let msg: AgentToHubMessageV1 = serde_json::from_str(text.as_ref())?;
        match msg {
            AgentToHubMessageV1::RunEvent {
                v,
                run_id,
                level,
                kind,
                message,
                fields,
            } => {
                assert_eq!(v, PROTOCOL_VERSION);
                assert_eq!(run_id, "run_id");
                assert_eq!(level, "info");
                assert_eq!(kind, PROGRESS_SNAPSHOT_EVENT_KIND_V1);
                assert_eq!(message, "packaging");
                assert_eq!(fields, Some(expected_fields));
            }
            _ => anyhow::bail!("expected RunEvent"),
        }
        Ok(())
    }
}
