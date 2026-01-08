use std::path::Path;

use super::io::{append_offline_event_line, write_offline_run_file_atomic};
use super::types::{OfflineRunEventV1, OfflineRunFileV1, OfflineRunStatusV1};

#[derive(Debug)]
enum OfflineWriterCommand {
    AppendEvent {
        level: String,
        kind: String,
        message: String,
        fields: Option<serde_json::Value>,
    },
    Finish {
        status: OfflineRunStatusV1,
        summary: Option<serde_json::Value>,
        error: Option<String>,
        respond_to: tokio::sync::oneshot::Sender<Result<(), anyhow::Error>>,
    },
}

pub(in super::super) struct OfflineRunWriterHandle {
    tx: tokio::sync::mpsc::UnboundedSender<OfflineWriterCommand>,
}

impl OfflineRunWriterHandle {
    pub(in super::super) async fn start(
        data_dir: &Path,
        run_id: &str,
        job_id: &str,
        job_name: &str,
        started_at: i64,
    ) -> Result<Self, anyhow::Error> {
        let run_dir = super::offline_run_dir(data_dir, run_id);
        tokio::fs::create_dir_all(&run_dir).await?;

        let run_path = run_dir.join("run.json");
        let events_path = run_dir.join("events.jsonl");

        write_offline_run_file_atomic(
            &run_path,
            OfflineRunFileV1 {
                v: 1,
                id: run_id.to_string(),
                job_id: job_id.to_string(),
                job_name: job_name.to_string(),
                status: OfflineRunStatusV1::Running,
                started_at,
                ended_at: None,
                summary: None,
                error: None,
            },
        )
        .await?;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<OfflineWriterCommand>();

        let run_id = run_id.to_string();
        let job_id = job_id.to_string();
        let job_name = job_name.to_string();
        tokio::spawn(async move {
            let mut next_seq: i64 = 1;
            while let Some(cmd) = rx.recv().await {
                match cmd {
                    OfflineWriterCommand::AppendEvent {
                        level,
                        kind,
                        message,
                        fields,
                    } => {
                        let event = OfflineRunEventV1 {
                            seq: next_seq,
                            ts: time::OffsetDateTime::now_utc().unix_timestamp(),
                            level,
                            kind,
                            message,
                            fields,
                        };
                        next_seq = next_seq.saturating_add(1);
                        if let Err(error) = append_offline_event_line(&events_path, &event).await {
                            tracing::warn!(
                                run_id = %run_id,
                                job_id = %job_id,
                                error = %error,
                                "failed to persist offline run event"
                            );
                        }
                    }
                    OfflineWriterCommand::Finish {
                        status,
                        summary,
                        error,
                        respond_to,
                    } => {
                        let ended_at = time::OffsetDateTime::now_utc().unix_timestamp();
                        let result = write_offline_run_file_atomic(
                            &run_path,
                            OfflineRunFileV1 {
                                v: 1,
                                id: run_id,
                                job_id,
                                job_name,
                                status,
                                started_at,
                                ended_at: Some(ended_at),
                                summary,
                                error,
                            },
                        )
                        .await;
                        let _ = respond_to.send(result);
                        break;
                    }
                }
            }
        });

        Ok(Self { tx })
    }

    pub(in super::super) fn append_event(
        &self,
        level: &str,
        kind: &str,
        message: &str,
        fields: Option<serde_json::Value>,
    ) -> Result<(), anyhow::Error> {
        self.tx
            .send(OfflineWriterCommand::AppendEvent {
                level: level.to_string(),
                kind: kind.to_string(),
                message: message.to_string(),
                fields,
            })
            .map_err(|_| anyhow::anyhow!("offline writer closed"))?;
        Ok(())
    }

    async fn finish(
        self,
        status: OfflineRunStatusV1,
        summary: Option<serde_json::Value>,
        error: Option<String>,
    ) -> Result<(), anyhow::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel::<Result<(), anyhow::Error>>();
        self.tx
            .send(OfflineWriterCommand::Finish {
                status,
                summary,
                error,
                respond_to: tx,
            })
            .map_err(|_| anyhow::anyhow!("offline writer closed"))?;

        rx.await
            .map_err(|_| anyhow::anyhow!("offline writer closed"))??;
        Ok(())
    }

    pub(in super::super) async fn finish_success(
        self,
        summary: serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        self.finish(OfflineRunStatusV1::Success, Some(summary), None)
            .await
    }

    pub(in super::super) async fn finish_failed(
        self,
        error_code: &str,
        summary: serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        self.finish(
            OfflineRunStatusV1::Failed,
            Some(summary),
            Some(error_code.to_string()),
        )
        .await
    }

    pub(in super::super) async fn finish_rejected(self) -> Result<(), anyhow::Error> {
        self.finish(
            OfflineRunStatusV1::Rejected,
            Some(serde_json::json!({ "executed_offline": true })),
            Some("overlap_rejected".to_string()),
        )
        .await
    }
}
