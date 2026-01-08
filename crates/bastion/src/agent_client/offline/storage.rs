use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub(super) fn offline_runs_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("agent").join("offline_runs")
}

pub(super) fn offline_run_dir(data_dir: &Path, run_id: &str) -> PathBuf {
    offline_runs_dir(data_dir).join(run_id)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum OfflineRunStatusV1 {
    Running,
    Success,
    Failed,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct OfflineRunFileV1 {
    pub(super) v: u32,
    pub(super) id: String,
    pub(super) job_id: String,
    pub(super) job_name: String,
    pub(super) status: OfflineRunStatusV1,
    pub(super) started_at: i64,
    pub(super) ended_at: Option<i64>,
    pub(super) summary: Option<serde_json::Value>,
    pub(super) error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct OfflineRunEventV1 {
    pub(super) seq: i64,
    pub(super) ts: i64,
    pub(super) level: String,
    pub(super) kind: String,
    pub(super) message: String,
    pub(super) fields: Option<serde_json::Value>,
}

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

pub(super) struct OfflineRunWriterHandle {
    tx: tokio::sync::mpsc::UnboundedSender<OfflineWriterCommand>,
}

impl OfflineRunWriterHandle {
    pub(super) async fn start(
        data_dir: &Path,
        run_id: &str,
        job_id: &str,
        job_name: &str,
        started_at: i64,
    ) -> Result<Self, anyhow::Error> {
        let run_dir = offline_run_dir(data_dir, run_id);
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

    pub(super) fn append_event(
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

    pub(super) async fn finish_success(
        self,
        summary: serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        self.finish(OfflineRunStatusV1::Success, Some(summary), None)
            .await
    }

    pub(super) async fn finish_failed(
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

    pub(super) async fn finish_rejected(self) -> Result<(), anyhow::Error> {
        self.finish(
            OfflineRunStatusV1::Rejected,
            Some(serde_json::json!({ "executed_offline": true })),
            Some("overlap_rejected".to_string()),
        )
        .await
    }
}

async fn write_offline_run_file_atomic(
    path: &Path,
    doc: OfflineRunFileV1,
) -> Result<(), anyhow::Error> {
    let bytes = serde_json::to_vec_pretty(&doc)?;
    let tmp = path.with_extension("json.partial");
    let _ = tokio::fs::remove_file(&tmp).await;
    tokio::fs::write(&tmp, bytes).await?;
    tokio::fs::rename(&tmp, path).await?;
    Ok(())
}

async fn append_offline_event_line(
    path: &Path,
    event: &OfflineRunEventV1,
) -> Result<(), anyhow::Error> {
    use tokio::io::AsyncWriteExt as _;

    let line = serde_json::to_vec(event)?;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;
    file.write_all(&line).await?;
    file.write_all(b"\n").await?;
    Ok(())
}
