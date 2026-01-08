use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio_tungstenite::tungstenite::Message;
use tracing::warn;

use bastion_core::agent_protocol::{AgentToHubMessageV1, JobSpecResolvedV1};
use bastion_core::run_failure::RunFailedWithSummary;

use super::cron::cron_matches_minute_cached;
use super::storage::OfflineRunWriterHandle;

#[derive(Debug, Clone)]
struct OfflineRunTask {
    run_id: String,
    job_id: String,
    job_name: String,
    spec: JobSpecResolvedV1,
}

#[derive(Debug, Default)]
struct InFlightCounts {
    per_job: std::collections::HashMap<String, usize>,
}

impl InFlightCounts {
    fn inflight_for_job(&self, job_id: &str) -> usize {
        self.per_job.get(job_id).copied().unwrap_or(0)
    }

    fn inc_job(&mut self, job_id: &str) {
        *self.per_job.entry(job_id.to_string()).or_insert(0) += 1;
    }

    fn dec_job(&mut self, job_id: &str) {
        let Some(v) = self.per_job.get_mut(job_id) else {
            return;
        };
        *v = v.saturating_sub(1);
        if *v == 0 {
            self.per_job.remove(job_id);
        }
    }
}

pub(super) async fn offline_scheduler_loop(
    data_dir: PathBuf,
    agent_id: String,
    run_lock: std::sync::Arc<tokio::sync::Mutex<()>>,
    connected_rx: tokio::sync::watch::Receiver<bool>,
) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<OfflineRunTask>();
    let inflight = std::sync::Arc::new(tokio::sync::Mutex::new(InFlightCounts::default()));

    tokio::spawn(offline_cron_loop(
        data_dir.clone(),
        agent_id.clone(),
        connected_rx,
        tx,
        inflight.clone(),
    ));

    offline_worker_loop(data_dir, agent_id, run_lock, rx, inflight).await;
}

async fn offline_cron_loop(
    data_dir: PathBuf,
    agent_id: String,
    mut connected_rx: tokio::sync::watch::Receiver<bool>,
    tx: tokio::sync::mpsc::UnboundedSender<OfflineRunTask>,
    inflight: std::sync::Arc<tokio::sync::Mutex<InFlightCounts>>,
) {
    use chrono::{DateTime, Duration as ChronoDuration, Utc};
    use cron::Schedule;

    let mut schedule_cache: std::collections::HashMap<String, Schedule> =
        std::collections::HashMap::new();
    let mut last_minute = time::OffsetDateTime::now_utc().unix_timestamp() / 60 - 1;

    loop {
        if *connected_rx.borrow() {
            if connected_rx.changed().await.is_err() {
                break;
            }
            continue;
        }

        let now = time::OffsetDateTime::now_utc();
        let now_ts = now.unix_timestamp();
        let now_dt = match DateTime::<Utc>::from_timestamp(now_ts, now.nanosecond()) {
            Some(ts) => ts,
            None => {
                tokio::select! {
                    _ = connected_rx.changed() => {}
                    _ = tokio::time::sleep(Duration::from_secs(5)) => {}
                }
                continue;
            }
        };

        let minute = now_ts / 60;
        let minute_start = match DateTime::<Utc>::from_timestamp(minute * 60, 0) {
            Some(ts) => ts,
            None => {
                tokio::select! {
                    _ = connected_rx.changed() => {}
                    _ = tokio::time::sleep(Duration::from_secs(5)) => {}
                }
                continue;
            }
        };

        if minute != last_minute {
            last_minute = minute;
            match super::super::managed::load_managed_config_snapshot(&data_dir, &agent_id) {
                Ok(Some(snapshot)) => {
                    for job in snapshot.jobs {
                        let Some(expr) = job
                            .schedule
                            .as_deref()
                            .map(str::trim)
                            .filter(|v| !v.is_empty())
                        else {
                            continue;
                        };
                        match cron_matches_minute_cached(expr, minute_start, &mut schedule_cache) {
                            Ok(true) => {
                                let should_reject = {
                                    let state = inflight.lock().await;
                                    matches!(
                                        job.overlap_policy,
                                        bastion_core::agent_protocol::OverlapPolicyV1::Reject
                                    ) && state.inflight_for_job(&job.job_id) > 0
                                };

                                if should_reject {
                                    if let Err(error) = persist_offline_rejected_run(
                                        &data_dir,
                                        &job.job_id,
                                        &job.name,
                                    )
                                    .await
                                    {
                                        warn!(
                                            agent_id = %agent_id,
                                            job_id = %job.job_id,
                                            error = %error,
                                            "failed to persist offline rejected run"
                                        );
                                    }
                                    continue;
                                }

                                let run_id = uuid::Uuid::new_v4().to_string();
                                let task = OfflineRunTask {
                                    run_id,
                                    job_id: job.job_id,
                                    job_name: job.name,
                                    spec: job.spec,
                                };

                                {
                                    let mut state = inflight.lock().await;
                                    state.inc_job(&task.job_id);
                                }

                                if tx.send(task).is_err() {
                                    break;
                                }
                            }
                            Ok(false) => {}
                            Err(error) => {
                                warn!(agent_id = %agent_id, error = %error, "invalid cron schedule; skipping");
                            }
                        }
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    warn!(agent_id = %agent_id, error = %error, "failed to load managed config snapshot");
                }
            }
        }

        let next_minute = now_dt + ChronoDuration::seconds(60 - (now_dt.timestamp() % 60));
        let sleep_dur = match next_minute.signed_duration_since(now_dt).to_std() {
            Ok(v) => v,
            Err(_) => std::time::Duration::from_secs(1),
        };

        tokio::select! {
            _ = connected_rx.changed() => {}
            _ = tokio::time::sleep(std::time::Duration::from_secs(1).min(sleep_dur)) => {}
        }
    }
}

async fn offline_worker_loop(
    data_dir: PathBuf,
    agent_id: String,
    run_lock: std::sync::Arc<tokio::sync::Mutex<()>>,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<OfflineRunTask>,
    inflight: std::sync::Arc<tokio::sync::Mutex<InFlightCounts>>,
) {
    while let Some(task) = rx.recv().await {
        let job_id = task.job_id.clone();
        let run_id = task.run_id.clone();

        let _guard = run_lock.lock().await;
        if let Err(error) = execute_offline_run_task(&data_dir, &agent_id, &task).await {
            warn!(
                agent_id = %agent_id,
                job_id = %job_id,
                run_id = %run_id,
                error = %error,
                "offline run failed"
            );
        }

        let mut state = inflight.lock().await;
        state.dec_job(&job_id);
    }
}

async fn persist_offline_rejected_run(
    data_dir: &Path,
    job_id: &str,
    job_name: &str,
) -> Result<(), anyhow::Error> {
    let run_id = uuid::Uuid::new_v4().to_string();
    let started_at = time::OffsetDateTime::now_utc().unix_timestamp();
    let writer =
        OfflineRunWriterHandle::start(data_dir, &run_id, job_id, job_name, started_at).await?;
    let _ = writer.append_event(
        "info",
        "rejected",
        "rejected",
        Some(serde_json::json!({ "source": "schedule", "executed_offline": true })),
    );
    writer.finish_rejected().await?;
    Ok(())
}

async fn execute_offline_run_task(
    data_dir: &Path,
    agent_id: &str,
    task: &OfflineRunTask,
) -> Result<(), anyhow::Error> {
    let started_at = time::OffsetDateTime::now_utc();
    let writer = OfflineRunWriterHandle::start(
        data_dir,
        &task.run_id,
        &task.job_id,
        &task.job_name,
        started_at.unix_timestamp(),
    )
    .await?;

    let mut sink = OfflineSink {
        writer,
        task_summary: None,
    };

    let _ = sink.writer.append_event(
        "info",
        "queued",
        "queued",
        Some(serde_json::json!({ "source": "schedule", "executed_offline": true })),
    );

    let run_task = bastion_core::agent_protocol::BackupRunTaskV1 {
        run_id: task.run_id.clone(),
        job_id: task.job_id.clone(),
        started_at: started_at.unix_timestamp(),
        spec: task.spec.clone(),
    };

    let outcome =
        super::super::handle_backup_task(data_dir, &mut sink, &task.run_id, run_task).await;
    let OfflineSink {
        writer,
        task_summary,
    } = sink;
    match outcome {
        Ok(()) => {
            let mut summary = task_summary.unwrap_or_else(|| serde_json::json!({}));
            mark_summary_executed_offline(&mut summary);
            writer.finish_success(summary).await?;
        }
        Err(error) => {
            let soft = error.downcast_ref::<RunFailedWithSummary>();
            let error_code = soft.map(|e| e.code).unwrap_or("run_failed");

            let mut summary = soft
                .map(|e| e.summary.clone())
                .unwrap_or_else(|| serde_json::json!({}));
            summary.as_object_mut().map(|o| {
                o.insert(
                    "error_code".to_string(),
                    serde_json::Value::String(error_code.to_string()),
                )
            });
            mark_summary_executed_offline(&mut summary);

            let message = format!("failed: {error}");
            let _ = writer.append_event(
                "error",
                "failed",
                &message,
                Some(serde_json::json!({ "agent_id": agent_id })),
            );
            writer.finish_failed(error_code, summary).await?;
        }
    }

    Ok(())
}

struct OfflineSink {
    writer: OfflineRunWriterHandle,
    task_summary: Option<serde_json::Value>,
}

impl futures_util::Sink<Message> for OfflineSink {
    type Error = tokio_tungstenite::tungstenite::Error;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn start_send(mut self: std::pin::Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        if let Message::Text(text) = item {
            let text = text.to_string();
            if let Ok(msg) = serde_json::from_str::<AgentToHubMessageV1>(&text) {
                match msg {
                    AgentToHubMessageV1::RunEvent {
                        level,
                        kind,
                        message,
                        fields,
                        ..
                    } => {
                        self.writer
                            .append_event(&level, &kind, &message, fields)
                            .map_err(to_ws_error)?;
                    }
                    AgentToHubMessageV1::TaskResult { summary, .. } => {
                        self.task_summary = summary;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

fn to_ws_error(error: anyhow::Error) -> tokio_tungstenite::tungstenite::Error {
    tokio_tungstenite::tungstenite::Error::Io(std::io::Error::other(error.to_string()))
}

fn mark_summary_executed_offline(summary: &mut serde_json::Value) {
    if let Some(obj) = summary.as_object_mut() {
        obj.insert(
            "executed_offline".to_string(),
            serde_json::Value::Bool(true),
        );
    } else {
        *summary = serde_json::json!({ "executed_offline": true });
    }
}
