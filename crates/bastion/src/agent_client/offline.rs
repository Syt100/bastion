use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;
use tracing::warn;
use url::Url;

use bastion_core::agent_protocol::{AgentToHubMessageV1, JobSpecResolvedV1};
use bastion_core::run_failure::RunFailedWithSummary;

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
            match super::managed::load_managed_config_snapshot(&data_dir, &agent_id) {
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

fn normalize_cron(expr: &str) -> Result<String, anyhow::Error> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    match parts.len() {
        5 => Ok(format!("0 {expr}")),
        6 => Ok(expr.to_string()),
        _ => Err(anyhow::anyhow!("invalid cron expression")),
    }
}

fn parse_cron_cached<'a>(
    expr: &str,
    schedule_cache: &'a mut std::collections::HashMap<String, cron::Schedule>,
) -> Result<&'a cron::Schedule, anyhow::Error> {
    use std::str::FromStr as _;

    let expr = normalize_cron(expr)?;
    if !schedule_cache.contains_key(&expr) {
        let schedule = cron::Schedule::from_str(&expr)?;
        schedule_cache.insert(expr.clone(), schedule);
    }
    Ok(schedule_cache
        .get(&expr)
        .expect("schedule_cache contains key we just inserted"))
}

fn cron_matches_minute_cached(
    expr: &str,
    minute_start: chrono::DateTime<chrono::Utc>,
    schedule_cache: &mut std::collections::HashMap<String, cron::Schedule>,
) -> Result<bool, anyhow::Error> {
    use chrono::Duration as ChronoDuration;

    let schedule = parse_cron_cached(expr, schedule_cache)?;
    let prev = minute_start - ChronoDuration::seconds(1);
    let mut iter = schedule.after(&prev);
    let Some(next) = iter.next() else {
        return Ok(false);
    };
    Ok(next == minute_start)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum OfflineRunStatusV1 {
    Running,
    Success,
    Failed,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
struct OfflineRunFileV1 {
    v: u32,
    id: String,
    job_id: String,
    job_name: String,
    status: OfflineRunStatusV1,
    started_at: i64,
    ended_at: Option<i64>,
    summary: Option<serde_json::Value>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OfflineRunEventV1 {
    seq: i64,
    ts: i64,
    level: String,
    kind: String,
    message: String,
    fields: Option<serde_json::Value>,
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

struct OfflineRunWriterHandle {
    tx: tokio::sync::mpsc::UnboundedSender<OfflineWriterCommand>,
}

impl OfflineRunWriterHandle {
    async fn start(
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

    fn append_event(
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

    async fn finish_success(self, summary: serde_json::Value) -> Result<(), anyhow::Error> {
        self.finish(OfflineRunStatusV1::Success, Some(summary), None)
            .await
    }

    async fn finish_failed(
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

    async fn finish_rejected(self) -> Result<(), anyhow::Error> {
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

fn offline_runs_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("agent").join("offline_runs")
}

fn offline_run_dir(data_dir: &Path, run_id: &str) -> PathBuf {
    offline_runs_dir(data_dir).join(run_id)
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

    let outcome = super::handle_backup_task(data_dir, &mut sink, &task.run_id, run_task).await;
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

#[derive(Debug, Serialize)]
struct AgentIngestRunRequestV1 {
    run: AgentIngestRunV1,
}

#[derive(Debug, Serialize)]
struct AgentIngestRunV1 {
    id: String,
    job_id: String,
    status: String,
    started_at: i64,
    ended_at: i64,
    summary: Option<serde_json::Value>,
    error: Option<String>,
    events: Vec<OfflineRunEventV1>,
}

pub(super) async fn sync_offline_runs(
    base_url: &Url,
    agent_key: &str,
    data_dir: &Path,
) -> Result<(), anyhow::Error> {
    let root = offline_runs_dir(data_dir);
    let mut entries = match tokio::fs::read_dir(&root).await {
        Ok(v) => v,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };

    let mut run_dirs = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            run_dirs.push(entry.path());
        }
    }
    run_dirs.sort();

    let ingest_url = base_url.join("agent/runs/ingest")?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    for dir in run_dirs {
        let run_path = dir.join("run.json");
        let bytes = tokio::fs::read(&run_path).await?;
        let run: OfflineRunFileV1 = serde_json::from_slice(&bytes)?;

        if run.status == OfflineRunStatusV1::Running {
            continue;
        }
        let ended_at = run.ended_at.unwrap_or(run.started_at);
        let status = match run.status {
            OfflineRunStatusV1::Success => "success",
            OfflineRunStatusV1::Failed => "failed",
            OfflineRunStatusV1::Rejected => "rejected",
            OfflineRunStatusV1::Running => continue,
        };

        let events_path = dir.join("events.jsonl");
        let mut events = Vec::new();
        match tokio::fs::File::open(&events_path).await {
            Ok(file) => {
                use tokio::io::AsyncBufReadExt as _;

                let mut lines = tokio::io::BufReader::new(file).lines();
                while let Some(line) = lines.next_line().await? {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    let ev: OfflineRunEventV1 = serde_json::from_str(line)?;
                    events.push(ev);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        let req = AgentIngestRunRequestV1 {
            run: AgentIngestRunV1 {
                id: run.id,
                job_id: run.job_id,
                status: status.to_string(),
                started_at: run.started_at,
                ended_at,
                summary: run.summary,
                error: run.error,
                events,
            },
        };

        let res = client
            .post(ingest_url.clone())
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {agent_key}"),
            )
            .json(&req)
            .send()
            .await?;

        if res.status() != reqwest::StatusCode::NO_CONTENT {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            anyhow::bail!("ingest failed: HTTP {status}: {text}");
        }

        tokio::fs::remove_dir_all(&dir).await?;
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::sync_offline_runs;

    #[test]
    fn normalize_cron_accepts_5_or_6_fields() {
        assert_eq!(
            super::normalize_cron("*/5 * * * *").unwrap(),
            "0 */5 * * * *"
        );
        assert_eq!(
            super::normalize_cron("0 */6 * * * *").unwrap(),
            "0 */6 * * * *"
        );
    }

    #[test]
    fn normalize_cron_rejects_other_field_counts() {
        assert!(super::normalize_cron("").is_err());
        assert!(super::normalize_cron("* * * *").is_err());
        assert!(super::normalize_cron("* * * * * * *").is_err());
    }

    #[test]
    fn cron_matches_minute_cached_matches_expected_minutes() {
        use chrono::TimeZone as _;

        let minute_start = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 10, 0).unwrap();
        let mut cache = std::collections::HashMap::new();

        assert!(
            super::cron_matches_minute_cached("*/5 * * * *", minute_start, &mut cache).unwrap()
        );
        assert!(
            super::cron_matches_minute_cached("0 */5 * * * *", minute_start, &mut cache).unwrap()
        );
        assert!(
            !super::cron_matches_minute_cached("*/7 * * * *", minute_start, &mut cache).unwrap()
        );
    }

    #[tokio::test]
    async fn sync_offline_runs_ingests_and_removes_dir() {
        use axum::routing::post;
        use axum::{Json, Router};
        use serde::Deserialize;

        #[derive(Debug, Deserialize)]
        struct IngestReq {
            run: IngestRun,
        }

        #[derive(Debug, Deserialize)]
        struct IngestRun {
            id: String,
            job_id: String,
            status: String,
            started_at: i64,
            ended_at: i64,
            summary: Option<serde_json::Value>,
            error: Option<String>,
            events: Vec<super::OfflineRunEventV1>,
        }

        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path();

        let run_id = "run1";
        let run_dir = super::offline_run_dir(data_dir, run_id);
        tokio::fs::create_dir_all(&run_dir).await.unwrap();

        let run_file = super::OfflineRunFileV1 {
            v: 1,
            id: run_id.to_string(),
            job_id: "job1".to_string(),
            job_name: "job1".to_string(),
            status: super::OfflineRunStatusV1::Success,
            started_at: 1,
            ended_at: Some(2),
            summary: Some(serde_json::json!({ "k": "v" })),
            error: None,
        };
        tokio::fs::write(
            run_dir.join("run.json"),
            serde_json::to_vec(&run_file).unwrap(),
        )
        .await
        .unwrap();

        let ev1 = super::OfflineRunEventV1 {
            seq: 1,
            ts: 1,
            level: "info".to_string(),
            kind: "start".to_string(),
            message: "start".to_string(),
            fields: None,
        };
        let ev2 = super::OfflineRunEventV1 {
            seq: 2,
            ts: 2,
            level: "info".to_string(),
            kind: "done".to_string(),
            message: "done".to_string(),
            fields: Some(serde_json::json!({ "n": 1 })),
        };
        let events_jsonl = format!(
            "{}\n{}\n",
            serde_json::to_string(&ev1).unwrap(),
            serde_json::to_string(&ev2).unwrap()
        );
        tokio::fs::write(run_dir.join("events.jsonl"), events_jsonl)
            .await
            .unwrap();

        let captured = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<IngestReq>::new()));
        let captured_clone = captured.clone();
        let agent_key = "agent-key";

        let app = Router::new().route(
            "/agent/runs/ingest",
            post(
                move |headers: axum::http::HeaderMap, Json(payload): Json<IngestReq>| {
                    let captured = captured_clone.clone();
                    async move {
                        let auth = headers
                            .get(axum::http::header::AUTHORIZATION)
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or_default();
                        assert_eq!(auth, format!("Bearer {agent_key}"));
                        captured.lock().await.push(payload);
                        axum::http::StatusCode::NO_CONTENT
                    }
                },
            ),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await;
        });

        let base_url = url::Url::parse(&format!("http://{addr}/")).unwrap();
        sync_offline_runs(&base_url, agent_key, data_dir)
            .await
            .unwrap();
        let _ = shutdown_tx.send(());

        assert!(!run_dir.exists());
        let captured = captured.lock().await;
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].run.id, "run1");
        assert_eq!(captured[0].run.job_id, "job1");
        assert_eq!(captured[0].run.status, "success");
        assert_eq!(captured[0].run.started_at, 1);
        assert_eq!(captured[0].run.ended_at, 2);
        assert_eq!(
            captured[0].run.summary.as_ref().and_then(|v| v.get("k")),
            Some(&serde_json::Value::String("v".to_string()))
        );
        assert!(captured[0].run.error.is_none());
        assert_eq!(captured[0].run.events.len(), 2);
    }
}
