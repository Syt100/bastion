use std::path::{Path, PathBuf};
use std::time::Duration;

use futures_util::{Sink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tracing::{debug, info, warn};
use url::Url;

use base64::Engine as _;
use bastion_core::agent_protocol::{
    AgentToHubMessageV1, EncryptionResolvedV1, HubToAgentMessageV1, JobConfigV1, JobSpecResolvedV1,
    PROTOCOL_VERSION, TargetResolvedV1,
};
use bastion_core::run_failure::RunFailedWithSummary;
use bastion_storage::secrets::{EncryptedSecret, SecretsCrypto};
use bastion_targets::WebdavCredentials;

use crate::config::AgentArgs;
use bastion_backup as backup;
use bastion_targets as targets;

const IDENTITY_FILE_NAME: &str = "agent.json";
const MANAGED_SECRETS_FILE_NAME: &str = "secrets.json";
const MANAGED_CONFIG_FILE_NAME: &str = "config.json";
const MANAGED_CONFIG_KIND: &str = "agent_config_snapshot";
const MANAGED_CONFIG_NAME: &str = "config";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AgentIdentityV1 {
    v: u32,
    hub_url: String,
    agent_id: String,
    agent_key: String,
    name: Option<String>,
    enrolled_at: i64,
}

#[derive(Debug, Serialize)]
struct EnrollRequest<'a> {
    token: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct EnrollResponse {
    agent_id: String,
    agent_key: String,
}

pub async fn run(args: AgentArgs) -> Result<(), anyhow::Error> {
    if args.heartbeat_seconds == 0 {
        anyhow::bail!("heartbeat_seconds must be > 0");
    }

    let data_dir = bastion_config::data_dir::resolve_data_dir(args.data_dir)?;
    let base_url = normalize_base_url(&args.hub_url)?;

    let identity_path = identity_path(&data_dir);
    let identity = match load_identity(&identity_path)? {
        Some(v) => {
            let stored_url = normalize_base_url(&v.hub_url)?;
            if stored_url != base_url {
                anyhow::bail!(
                    "agent is already enrolled for hub_url={}, delete {} to re-enroll",
                    stored_url,
                    identity_path.display()
                );
            }
            v
        }
        None => {
            let Some(token) = args.enroll_token.as_deref() else {
                anyhow::bail!(
                    "agent is not enrolled yet; provide --enroll-token or set BASTION_AGENT_ENROLL_TOKEN"
                );
            };

            info!(hub_url = %base_url, "enrolling agent");
            let resp = enroll(&base_url, token, args.name.as_deref()).await?;
            let now = time::OffsetDateTime::now_utc().unix_timestamp();
            let identity = AgentIdentityV1 {
                v: 1,
                hub_url: base_url.to_string(),
                agent_id: resp.agent_id,
                agent_key: resp.agent_key,
                name: args.name.clone(),
                enrolled_at: now,
            };
            save_identity(&identity_path, &identity)?;
            identity
        }
    };

    let ws_url = agent_ws_url(&base_url)?;
    let heartbeat = Duration::from_secs(args.heartbeat_seconds);
    let pong_timeout = Duration::from_secs(args.heartbeat_seconds.saturating_mul(3));
    let mut backoff = Duration::from_secs(1);
    let mut attempt = 0u32;

    let run_lock = std::sync::Arc::new(tokio::sync::Mutex::new(()));
    let (connected_tx, connected_rx) = tokio::sync::watch::channel(false);

    tokio::spawn(offline_scheduler_loop(
        data_dir.clone(),
        identity.agent_id.clone(),
        run_lock.clone(),
        connected_rx,
    ));

    loop {
        let action = connect_and_run(
            &ws_url,
            &identity,
            &data_dir,
            heartbeat,
            pong_timeout,
            run_lock.clone(),
            &connected_tx,
        )
        .await;
        match action {
            Ok(LoopAction::Exit) => return Ok(()),
            Ok(LoopAction::Reconnect) => {
                attempt = attempt.saturating_add(1);
                tokio::time::sleep(jittered_backoff(backoff, &identity.agent_id, attempt)).await;
                backoff = std::cmp::min(backoff * 2, Duration::from_secs(30));
            }
            Err(error) => {
                warn!(error = %error, "agent connection failed; retrying");
                attempt = attempt.saturating_add(1);
                tokio::time::sleep(jittered_backoff(backoff, &identity.agent_id, attempt)).await;
                backoff = std::cmp::min(backoff * 2, Duration::from_secs(30));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoopAction {
    Reconnect,
    Exit,
}

async fn connect_and_run(
    ws_url: &Url,
    identity: &AgentIdentityV1,
    data_dir: &Path,
    heartbeat: Duration,
    pong_timeout: Duration,
    run_lock: std::sync::Arc<tokio::sync::Mutex<()>>,
    connected_tx: &tokio::sync::watch::Sender<bool>,
) -> Result<LoopAction, anyhow::Error> {
    let mut req = ws_url.as_str().into_client_request()?;
    req.headers_mut().insert(
        AUTHORIZATION,
        format!("Bearer {}", identity.agent_key).parse()?,
    );

    let (socket, _) = tokio_tungstenite::connect_async(req).await?;
    let (mut tx, mut rx) = socket.split();

    let _ = connected_tx.send(true);
    struct ConnectedGuard(tokio::sync::watch::Sender<bool>);
    impl Drop for ConnectedGuard {
        fn drop(&mut self) {
            let _ = self.0.send(false);
        }
    }
    let _connected_guard = ConnectedGuard(connected_tx.clone());

    let hello = AgentToHubMessageV1::Hello {
        v: PROTOCOL_VERSION,
        agent_id: identity.agent_id.clone(),
        name: identity.name.clone(),
        info: serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        }),
        capabilities: serde_json::json!({
            "backup": ["filesystem", "sqlite", "vaultwarden"],
        }),
    };
    tx.send(Message::Text(serde_json::to_string(&hello)?.into()))
        .await?;

    if let Ok(base_url) = normalize_base_url(&identity.hub_url)
        && let Err(error) = sync_offline_runs(&base_url, &identity.agent_key, data_dir).await
    {
        warn!(
            agent_id = %identity.agent_id,
            error = %error,
            "failed to sync offline runs"
        );
    }

    let mut tick = tokio::time::interval(heartbeat);
    let mut last_pong = tokio::time::Instant::now();
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            _ = &mut shutdown => {
                let _ = tx.send(Message::Close(None)).await;
                return Ok(LoopAction::Exit);
            }
            _ = tick.tick() => {
                if last_pong.elapsed() > pong_timeout {
                    warn!(
                        agent_id = %identity.agent_id,
                        timeout_seconds = pong_timeout.as_secs(),
                        "pong timeout; reconnecting"
                    );
                    return Ok(LoopAction::Reconnect);
                }
                let ping = AgentToHubMessageV1::Ping { v: PROTOCOL_VERSION };
                if tx.send(Message::Text(serde_json::to_string(&ping)?.into())).await.is_err() {
                    return Ok(LoopAction::Reconnect);
                }
            }
            msg = rx.next() => {
                let Some(msg) = msg else {
                    return Ok(LoopAction::Reconnect);
                };
                match msg {
                    Ok(Message::Text(text)) => {
                        let text = text.to_string();
                        match serde_json::from_str::<HubToAgentMessageV1>(&text) {
                            Ok(HubToAgentMessageV1::Pong { .. }) => {
                                last_pong = tokio::time::Instant::now();
                            }
                            Ok(HubToAgentMessageV1::SecretsSnapshot {
                                v,
                                node_id,
                                issued_at,
                                webdav,
                            }) if v == PROTOCOL_VERSION => {
                                if node_id != identity.agent_id {
                                    warn!(
                                        agent_id = %identity.agent_id,
                                        node_id = %node_id,
                                        "received secrets snapshot for unexpected node_id; ignoring"
                                    );
                                    continue;
                                }

                                if let Err(error) =
                                    save_managed_secrets_snapshot(data_dir, &node_id, issued_at, &webdav)
                                {
                                    warn!(agent_id = %identity.agent_id, error = %error, "failed to persist secrets snapshot");
                                } else {
                                    debug!(
                                        agent_id = %identity.agent_id,
                                        webdav = webdav.len(),
                                        "persisted secrets snapshot"
                                    );
                                }
                            }
                            Ok(HubToAgentMessageV1::ConfigSnapshot {
                                v,
                                node_id,
                                snapshot_id,
                                issued_at,
                                jobs,
                            }) if v == PROTOCOL_VERSION => {
                                if node_id != identity.agent_id {
                                    warn!(
                                        agent_id = %identity.agent_id,
                                        node_id = %node_id,
                                        "received config snapshot for unexpected node_id; ignoring"
                                    );
                                    continue;
                                }

                                if let Err(error) = save_managed_config_snapshot(
                                    data_dir,
                                    &node_id,
                                    &snapshot_id,
                                    issued_at,
                                    &jobs,
                                ) {
                                    warn!(
                                        agent_id = %identity.agent_id,
                                        snapshot_id = %snapshot_id,
                                        error = %error,
                                        "failed to persist config snapshot"
                                    );
                                } else {
                                    debug!(
                                        agent_id = %identity.agent_id,
                                        snapshot_id = %snapshot_id,
                                        jobs = jobs.len(),
                                        "persisted config snapshot"
                                    );
                                }

                                let ack = AgentToHubMessageV1::ConfigAck {
                                    v: PROTOCOL_VERSION,
                                    snapshot_id,
                                };
                                if tx.send(Message::Text(serde_json::to_string(&ack)?.into())).await.is_err() {
                                    return Ok(LoopAction::Reconnect);
                                }
                            }
                            Ok(HubToAgentMessageV1::Task { v, task_id, task }) if v == PROTOCOL_VERSION => {
                                let run_id = task.run_id.clone();
                                debug!(task_id = %task_id, run_id = %run_id, "received task");

                                if let Some(cached) = load_cached_task_result(data_dir, &task_id, &run_id) {
                                    debug!(task_id = %task_id, run_id = %run_id, "replaying cached task result");
                                    let ack = AgentToHubMessageV1::Ack { v: PROTOCOL_VERSION, task_id: task_id.clone() };
                                    if tx.send(Message::Text(serde_json::to_string(&ack)?.into())).await.is_err() {
                                        return Ok(LoopAction::Reconnect);
                                    }

                                    if tx.send(Message::Text(serde_json::to_string(&cached)?.into())).await.is_err() {
                                        return Ok(LoopAction::Reconnect);
                                    }
                                    continue;
                                }

                                let ack = AgentToHubMessageV1::Ack { v: PROTOCOL_VERSION, task_id: task_id.clone() };
                                if tx.send(Message::Text(serde_json::to_string(&ack)?.into())).await.is_err() {
                                    return Ok(LoopAction::Reconnect);
                                }

                                let _guard = run_lock.lock().await;
                                match handle_backup_task(data_dir, &mut tx, &task_id, *task).await {
                                    Ok(()) => {}
                                    Err(error) => {
                                        if is_ws_error(&error) {
                                            warn!(
                                                task_id = %task_id,
                                                run_id = %run_id,
                                                error = %error,
                                                "task aborted due to websocket error; reconnecting"
                                            );
                                            return Ok(LoopAction::Reconnect);
                                        }

                                        warn!(task_id = %task_id, run_id = %run_id, error = %error, "task failed");
                                        let summary = error
                                            .downcast_ref::<RunFailedWithSummary>()
                                            .map(|e| e.summary.clone());
                                        let result = AgentToHubMessageV1::TaskResult {
                                            v: PROTOCOL_VERSION,
                                            task_id: task_id.clone(),
                                            run_id,
                                            status: "failed".to_string(),
                                            summary,
                                            error: Some(format!("{error:#}")),
                                        };

                                        if let Err(error) = save_task_result(data_dir, &result) {
                                            warn!(task_id = %task_id, error = %error, "failed to persist task result");
                                        }

                                        if tx.send(Message::Text(serde_json::to_string(&result)?.into())).await.is_err() {
                                            return Ok(LoopAction::Reconnect);
                                        }
                                    }
                                };
                            }
                            _ => {}
                        }
                    }
                    Ok(Message::Close(_)) => return Ok(LoopAction::Reconnect),
                    Ok(_) => {}
                    Err(_) => return Ok(LoopAction::Reconnect),
                }
            }
        }
    }
}

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

async fn offline_scheduler_loop(
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
            match load_managed_config_snapshot(&data_dir, &agent_id) {
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
                                    ) {
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

struct OfflineRunWriter {
    run_id: String,
    job_id: String,
    job_name: String,
    started_at: i64,
    run_path: PathBuf,
    events_path: PathBuf,
    next_seq: i64,
}

impl OfflineRunWriter {
    fn start(
        data_dir: &Path,
        run_id: &str,
        job_id: &str,
        job_name: &str,
        started_at: i64,
    ) -> Result<Self, anyhow::Error> {
        let run_dir = offline_run_dir(data_dir, run_id);
        std::fs::create_dir_all(&run_dir)?;

        let run_path = run_dir.join("run.json");
        let events_path = run_dir.join("events.jsonl");

        let writer = Self {
            run_id: run_id.to_string(),
            job_id: job_id.to_string(),
            job_name: job_name.to_string(),
            started_at,
            run_path,
            events_path,
            next_seq: 1,
        };

        writer.write_run(OfflineRunStatusV1::Running, None, None, None)?;
        Ok(writer)
    }

    fn append_event(
        &mut self,
        level: &str,
        kind: &str,
        message: &str,
        fields: Option<serde_json::Value>,
    ) -> Result<(), anyhow::Error> {
        let event = OfflineRunEventV1 {
            seq: self.next_seq,
            ts: time::OffsetDateTime::now_utc().unix_timestamp(),
            level: level.to_string(),
            kind: kind.to_string(),
            message: message.to_string(),
            fields,
        };
        self.next_seq = self.next_seq.saturating_add(1);

        let line = serde_json::to_vec(&event)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.events_path)?;
        use std::io::Write as _;
        file.write_all(&line)?;
        file.write_all(b"\n")?;
        Ok(())
    }

    fn finish_success(&mut self, summary: serde_json::Value) -> Result<(), anyhow::Error> {
        let ended_at = time::OffsetDateTime::now_utc().unix_timestamp();
        self.write_run(
            OfflineRunStatusV1::Success,
            Some(ended_at),
            Some(summary),
            None,
        )?;
        Ok(())
    }

    fn finish_failed(
        &mut self,
        error_code: &str,
        summary: serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        let ended_at = time::OffsetDateTime::now_utc().unix_timestamp();
        self.write_run(
            OfflineRunStatusV1::Failed,
            Some(ended_at),
            Some(summary),
            Some(error_code.to_string()),
        )?;
        Ok(())
    }

    fn finish_rejected(&mut self) -> Result<(), anyhow::Error> {
        let ended_at = time::OffsetDateTime::now_utc().unix_timestamp();
        self.write_run(
            OfflineRunStatusV1::Rejected,
            Some(ended_at),
            Some(serde_json::json!({ "executed_offline": true })),
            Some("overlap_rejected".to_string()),
        )?;
        Ok(())
    }

    fn write_run(
        &self,
        status: OfflineRunStatusV1,
        ended_at: Option<i64>,
        summary: Option<serde_json::Value>,
        error: Option<String>,
    ) -> Result<(), anyhow::Error> {
        let doc = OfflineRunFileV1 {
            v: 1,
            id: self.run_id.clone(),
            job_id: self.job_id.clone(),
            job_name: self.job_name.clone(),
            status,
            started_at: self.started_at,
            ended_at,
            summary,
            error,
        };

        let bytes = serde_json::to_vec_pretty(&doc)?;
        let tmp = self.run_path.with_extension("json.partial");
        let _ = std::fs::remove_file(&tmp);
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(&tmp, &self.run_path)?;
        Ok(())
    }
}

struct OfflineSink {
    writer: OfflineRunWriter,
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

fn persist_offline_rejected_run(
    data_dir: &Path,
    job_id: &str,
    job_name: &str,
) -> Result<(), anyhow::Error> {
    let run_id = uuid::Uuid::new_v4().to_string();
    let started_at = time::OffsetDateTime::now_utc().unix_timestamp();
    let mut writer = OfflineRunWriter::start(data_dir, &run_id, job_id, job_name, started_at)?;
    let _ = writer.append_event(
        "info",
        "rejected",
        "rejected",
        Some(serde_json::json!({ "source": "schedule", "executed_offline": true })),
    );
    writer.finish_rejected()?;
    Ok(())
}

async fn execute_offline_run_task(
    data_dir: &Path,
    agent_id: &str,
    task: &OfflineRunTask,
) -> Result<(), anyhow::Error> {
    let started_at = time::OffsetDateTime::now_utc();
    let mut sink = OfflineSink {
        writer: OfflineRunWriter::start(
            data_dir,
            &task.run_id,
            &task.job_id,
            &task.job_name,
            started_at.unix_timestamp(),
        )?,
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

    let outcome = handle_backup_task(data_dir, &mut sink, &task.run_id, run_task).await;
    match outcome {
        Ok(()) => {
            let mut summary = sink.task_summary.unwrap_or_else(|| serde_json::json!({}));
            mark_summary_executed_offline(&mut summary);
            sink.writer.finish_success(summary)?;
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
            let _ = sink.writer.append_event(
                "error",
                "failed",
                &message,
                Some(serde_json::json!({ "agent_id": agent_id })),
            );
            sink.writer.finish_failed(error_code, summary)?;
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

async fn sync_offline_runs(
    base_url: &Url,
    agent_key: &str,
    data_dir: &Path,
) -> Result<(), anyhow::Error> {
    let root = offline_runs_dir(data_dir);
    let entries = match std::fs::read_dir(&root) {
        Ok(v) => v,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };

    let mut run_dirs = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            run_dirs.push(path);
        }
    }
    run_dirs.sort();

    let ingest_url = base_url.join("agent/runs/ingest")?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    for dir in run_dirs {
        let run_path = dir.join("run.json");
        let bytes = std::fs::read(&run_path)?;
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
        if let Ok(text) = std::fs::read_to_string(&events_path) {
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let ev: OfflineRunEventV1 = serde_json::from_str(line)?;
                events.push(ev);
            }
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
            .header(AUTHORIZATION, format!("Bearer {agent_key}"))
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

async fn handle_backup_task(
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

fn target_part_size_bytes(target: &TargetResolvedV1) -> u64 {
    match target {
        TargetResolvedV1::Webdav {
            part_size_bytes, ..
        } => *part_size_bytes,
        TargetResolvedV1::LocalDir {
            part_size_bytes, ..
        } => *part_size_bytes,
    }
}

async fn store_artifacts_to_resolved_target(
    job_id: &str,
    run_id: &str,
    target: &TargetResolvedV1,
    artifacts: &backup::LocalRunArtifacts,
) -> Result<serde_json::Value, anyhow::Error> {
    match target {
        TargetResolvedV1::Webdav {
            base_url,
            username,
            password,
            ..
        } => {
            let creds = WebdavCredentials {
                username: username.clone(),
                password: password.clone(),
            };
            let run_url =
                targets::webdav::store_run(base_url, creds, job_id, run_id, artifacts).await?;
            Ok(serde_json::json!({ "type": "webdav", "run_url": run_url.as_str() }))
        }
        TargetResolvedV1::LocalDir { base_dir, .. } => {
            let base_dir = base_dir.to_string();
            let job_id = job_id.to_string();
            let run_id = run_id.to_string();
            let artifacts = artifacts.clone();
            let run_dir = tokio::task::spawn_blocking(move || {
                targets::local_dir::store_run(
                    std::path::Path::new(&base_dir),
                    &job_id,
                    &run_id,
                    &artifacts,
                )
            })
            .await??;
            Ok(serde_json::json!({
                "type": "local_dir",
                "run_dir": run_dir.to_string_lossy().to_string()
            }))
        }
    }
}

fn identity_path(data_dir: &Path) -> PathBuf {
    data_dir.join(IDENTITY_FILE_NAME)
}

fn normalize_base_url(raw: &str) -> Result<Url, anyhow::Error> {
    let mut url = Url::parse(raw)?;
    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/", url.path()));
    }
    Ok(url)
}

fn agent_ws_url(base_url: &Url) -> Result<Url, anyhow::Error> {
    let mut url = base_url.clone();
    match url.scheme() {
        "http" => url.set_scheme("ws").ok(),
        "https" => url.set_scheme("wss").ok(),
        other => anyhow::bail!("unsupported hub_url scheme: {other}"),
    };
    Ok(url.join("agent/ws")?)
}

async fn enroll(
    base_url: &Url,
    token: &str,
    name: Option<&str>,
) -> Result<EnrollResponse, anyhow::Error> {
    let enroll_url = base_url.join("agent/enroll")?;
    let res = reqwest::Client::new()
        .post(enroll_url)
        .json(&EnrollRequest { token, name })
        .send()
        .await?;

    if res.status() != reqwest::StatusCode::OK {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        anyhow::bail!("enroll failed: HTTP {status}: {text}");
    }

    Ok(res.json::<EnrollResponse>().await?)
}

fn load_identity(path: &Path) -> Result<Option<AgentIdentityV1>, anyhow::Error> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(serde_json::from_slice::<AgentIdentityV1>(&bytes)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn save_identity(path: &Path, identity: &AgentIdentityV1) -> Result<(), anyhow::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let bytes = serde_json::to_vec_pretty(identity)?;
    let tmp = path.with_extension("json.partial");
    let _ = std::fs::remove_file(&tmp);
    std::fs::write(&tmp, bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }

    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn jittered_backoff(base: Duration, agent_id: &str, attempt: u32) -> Duration {
    if base.is_zero() {
        return base;
    }

    // Equal-jitter backoff: [base/2, base], deterministic per agent+attempt.
    let half = base / 2;
    let half_ms = half.as_millis().min(u128::from(u64::MAX)) as u64;
    if half_ms == 0 {
        return base;
    }

    let seed = fnv1a64(agent_id.as_bytes())
        .wrapping_add(u64::from(attempt).wrapping_mul(0x9e3779b97f4a7c15));
    let jitter_ms = seed % (half_ms + 1);
    half + Duration::from_millis(jitter_ms)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn is_ws_error(error: &anyhow::Error) -> bool {
    error
        .chain()
        .any(|e| e.is::<tokio_tungstenite::tungstenite::Error>())
}

fn is_safe_task_id(task_id: &str) -> bool {
    !task_id.is_empty()
        && task_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn task_results_dir(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join("agent").join("task_results")
}

fn managed_secrets_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir
        .join("agent")
        .join("managed")
        .join(MANAGED_SECRETS_FILE_NAME)
}

fn managed_config_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir
        .join("agent")
        .join("managed")
        .join(MANAGED_CONFIG_FILE_NAME)
}

fn task_result_path(data_dir: &Path, task_id: &str) -> Option<std::path::PathBuf> {
    if !is_safe_task_id(task_id) {
        return None;
    }
    Some(task_results_dir(data_dir).join(format!("{task_id}.json")))
}

#[derive(Debug, Serialize, Deserialize)]
struct ManagedSecretsFileV1 {
    v: u32,
    node_id: String,
    issued_at: i64,
    saved_at: i64,
    #[serde(default)]
    webdav: Vec<ManagedWebdavSecretV1>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManagedWebdavSecretV1 {
    name: String,
    updated_at: i64,
    kid: u32,
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebdavSecretPayload {
    username: String,
    password: String,
}

fn save_managed_secrets_snapshot(
    data_dir: &Path,
    node_id: &str,
    issued_at: i64,
    webdav: &[bastion_core::agent_protocol::WebdavSecretV1],
) -> Result<(), anyhow::Error> {
    let crypto = SecretsCrypto::load_or_create(data_dir)?;

    let saved_at = time::OffsetDateTime::now_utc().unix_timestamp();
    let mut entries = Vec::with_capacity(webdav.len());
    for secret in webdav {
        let payload = WebdavSecretPayload {
            username: secret.username.clone(),
            password: secret.password.clone(),
        };
        let bytes = serde_json::to_vec(&payload)?;
        let encrypted = crypto.encrypt(node_id, "webdav", &secret.name, &bytes)?;
        entries.push(ManagedWebdavSecretV1 {
            name: secret.name.clone(),
            updated_at: secret.updated_at,
            kid: encrypted.kid,
            nonce: encrypted.nonce.to_vec(),
            ciphertext: encrypted.ciphertext,
        });
    }

    let doc = ManagedSecretsFileV1 {
        v: 1,
        node_id: node_id.to_string(),
        issued_at,
        saved_at,
        webdav: entries,
    };

    let path = managed_secrets_path(data_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let bytes = serde_json::to_vec_pretty(&doc)?;
    let tmp = path.with_extension("json.partial");
    let _ = std::fs::remove_file(&tmp);
    std::fs::write(&tmp, bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }

    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct ManagedConfigFileV1 {
    v: u32,
    node_id: String,
    snapshot_id: String,
    issued_at: i64,
    saved_at: i64,
    encrypted: EncryptedBlobV1,
}

#[derive(Debug, Serialize, Deserialize)]
struct EncryptedBlobV1 {
    kid: u32,
    nonce_b64: String,
    ciphertext_b64: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ManagedConfigPlainV1 {
    v: u32,
    snapshot_id: String,
    issued_at: i64,
    jobs: Vec<JobConfigV1>,
}

fn save_managed_config_snapshot(
    data_dir: &Path,
    node_id: &str,
    snapshot_id: &str,
    issued_at: i64,
    jobs: &[JobConfigV1],
) -> Result<(), anyhow::Error> {
    let crypto = SecretsCrypto::load_or_create(data_dir)?;

    let plain = ManagedConfigPlainV1 {
        v: 1,
        snapshot_id: snapshot_id.to_string(),
        issued_at,
        jobs: jobs.to_vec(),
    };
    let bytes = serde_json::to_vec(&plain)?;

    let encrypted = crypto.encrypt(node_id, MANAGED_CONFIG_KIND, MANAGED_CONFIG_NAME, &bytes)?;
    let encrypted = EncryptedBlobV1 {
        kid: encrypted.kid,
        nonce_b64: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(encrypted.nonce),
        ciphertext_b64: base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(encrypted.ciphertext),
    };

    let saved_at = time::OffsetDateTime::now_utc().unix_timestamp();
    let doc = ManagedConfigFileV1 {
        v: 1,
        node_id: node_id.to_string(),
        snapshot_id: snapshot_id.to_string(),
        issued_at,
        saved_at,
        encrypted,
    };

    let path = managed_config_path(data_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let bytes = serde_json::to_vec_pretty(&doc)?;
    let tmp = path.with_extension("json.partial");
    let _ = std::fs::remove_file(&tmp);
    std::fs::write(&tmp, bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }

    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn load_managed_config_snapshot(
    data_dir: &Path,
    node_id: &str,
) -> Result<Option<ManagedConfigPlainV1>, anyhow::Error> {
    let path = managed_config_path(data_dir);
    let bytes = match std::fs::read(&path) {
        Ok(v) => v,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };

    let doc: ManagedConfigFileV1 = serde_json::from_slice(&bytes)?;
    if doc.v != 1 {
        anyhow::bail!("unsupported managed config snapshot version: {}", doc.v);
    }

    let nonce = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(doc.encrypted.nonce_b64.as_bytes())?;
    let ciphertext = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(doc.encrypted.ciphertext_b64.as_bytes())?;

    let nonce: [u8; 24] = nonce
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid nonce length"))?;

    let crypto = SecretsCrypto::load_or_create(data_dir)?;
    let secret = EncryptedSecret {
        kid: doc.encrypted.kid,
        nonce,
        ciphertext,
    };
    let plaintext = crypto.decrypt(node_id, MANAGED_CONFIG_KIND, MANAGED_CONFIG_NAME, &secret)?;
    let plain: ManagedConfigPlainV1 = serde_json::from_slice(&plaintext)?;
    Ok(Some(plain))
}

fn load_cached_task_result(
    data_dir: &Path,
    task_id: &str,
    run_id: &str,
) -> Option<AgentToHubMessageV1> {
    let path = task_result_path(data_dir, task_id)?;
    let bytes = std::fs::read(path).ok()?;
    let msg = serde_json::from_slice::<AgentToHubMessageV1>(&bytes).ok()?;
    match &msg {
        AgentToHubMessageV1::TaskResult {
            v,
            task_id: saved_task_id,
            run_id: saved_run_id,
            ..
        } if *v == PROTOCOL_VERSION && saved_task_id == task_id && saved_run_id == run_id => {
            Some(msg)
        }
        _ => None,
    }
}

fn save_task_result(data_dir: &Path, msg: &AgentToHubMessageV1) -> Result<(), anyhow::Error> {
    let AgentToHubMessageV1::TaskResult { task_id, .. } = msg else {
        return Ok(());
    };

    let Some(path) = task_result_path(data_dir, task_id) else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let bytes = serde_json::to_vec_pretty(msg)?;
    let tmp = path.with_extension("json.partial");
    let _ = std::fs::remove_file(&tmp);
    std::fs::write(&tmp, bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }

    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ManagedConfigFileV1, ManagedSecretsFileV1, agent_ws_url, managed_config_path,
        managed_secrets_path, normalize_base_url, save_identity, save_managed_config_snapshot,
        save_managed_secrets_snapshot,
    };

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

    #[test]
    fn normalize_base_url_appends_slash() {
        let url = normalize_base_url("http://localhost:9876").unwrap();
        assert_eq!(url.as_str(), "http://localhost:9876/");
    }

    #[test]
    fn agent_ws_url_converts_scheme() {
        let base = normalize_base_url("https://hub.example.com/bastion").unwrap();
        let ws = agent_ws_url(&base).unwrap();
        assert_eq!(ws.as_str(), "wss://hub.example.com/bastion/agent/ws");
    }

    #[test]
    fn identity_is_written_atomically() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("agent.json");
        let id = super::AgentIdentityV1 {
            v: 1,
            hub_url: "http://localhost:9876/".to_string(),
            agent_id: "a".to_string(),
            agent_key: "k".to_string(),
            name: Some("n".to_string()),
            enrolled_at: 1,
        };

        save_identity(&path, &id).unwrap();
        let saved = std::fs::read_to_string(&path).unwrap();
        assert!(saved.contains("\"agent_id\""));
    }

    #[test]
    fn managed_secrets_snapshot_is_persisted_encrypted() {
        let tmp = tempfile::tempdir().unwrap();
        let webdav = vec![bastion_core::agent_protocol::WebdavSecretV1 {
            name: "primary".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
            updated_at: 10,
        }];

        save_managed_secrets_snapshot(tmp.path(), "a", 123, &webdav).unwrap();

        let path = managed_secrets_path(tmp.path());
        assert!(path.exists());

        let bytes = std::fs::read(&path).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(!text.contains("pass"));

        let saved: ManagedSecretsFileV1 = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(saved.v, 1);
        assert_eq!(saved.node_id, "a");
        assert_eq!(saved.issued_at, 123);
        assert_eq!(saved.webdav.len(), 1);
        assert_eq!(saved.webdav[0].name, "primary");
        assert_eq!(saved.webdav[0].updated_at, 10);
        assert_eq!(saved.webdav[0].nonce.len(), 24);
        assert!(!saved.webdav[0].ciphertext.is_empty());

        assert!(tmp.path().join("master.key").exists());
    }

    #[test]
    fn managed_config_snapshot_is_persisted_encrypted() {
        let tmp = tempfile::tempdir().unwrap();
        let jobs = vec![bastion_core::agent_protocol::JobConfigV1 {
            job_id: "job1".to_string(),
            name: "job1".to_string(),
            schedule: Some("0 */6 * * *".to_string()),
            overlap_policy: bastion_core::agent_protocol::OverlapPolicyV1::Queue,
            updated_at: 10,
            spec: bastion_core::agent_protocol::JobSpecResolvedV1::Filesystem {
                v: 1,
                pipeline: Default::default(),
                source: bastion_core::job_spec::FilesystemSource {
                    root: "/".to_string(),
                    include: vec![],
                    exclude: vec![],
                    symlink_policy: Default::default(),
                    hardlink_policy: Default::default(),
                    error_policy: bastion_core::job_spec::FsErrorPolicy::FailFast,
                },
                target: bastion_core::agent_protocol::TargetResolvedV1::LocalDir {
                    base_dir: "/tmp".to_string(),
                    part_size_bytes: 1024,
                },
            },
        }];

        save_managed_config_snapshot(tmp.path(), "a", "snap1", 123, &jobs).unwrap();

        let path = managed_config_path(tmp.path());
        assert!(path.exists());

        let bytes = std::fs::read(&path).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        // Ensure the on-disk doc doesn't contain obvious plaintext fields.
        assert!(!text.contains("\"base_dir\""));
        assert!(!text.contains("0 */6 * * *"));

        let saved: ManagedConfigFileV1 = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(saved.v, 1);
        assert_eq!(saved.node_id, "a");
        assert_eq!(saved.snapshot_id, "snap1");
        assert_eq!(saved.issued_at, 123);
        assert!(!saved.encrypted.nonce_b64.is_empty());
        assert!(!saved.encrypted.ciphertext_b64.is_empty());

        assert!(tmp.path().join("master.key").exists());
    }
}
