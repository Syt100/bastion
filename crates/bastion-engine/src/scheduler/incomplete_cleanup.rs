use std::sync::Arc;
use std::time::Duration;

use sqlx::SqlitePool;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use url::Url;

use bastion_core::HUB_NODE_ID;
use bastion_core::job_spec;
use bastion_storage::incomplete_cleanup_repo;
use bastion_storage::jobs_repo;
use bastion_storage::runs_repo;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;
use bastion_targets::WebdavClient;
use bastion_targets::WebdavCredentials;

use super::target_snapshot;

const RECONCILE_BATCH_LIMIT: u32 = 200;
const PROCESS_BATCH_LIMIT: u32 = 20;

const MAX_SLEEP_SECS: u64 = 60 * 60;
const SHORT_SLEEP_SECS: u64 = 5;

const RUNNING_TTL_SECS: i64 = 30 * 60;
const MAX_ATTEMPTS: i64 = 20;
const MAX_AGE_SECS: i64 = 30 * 24 * 60 * 60;

pub(super) async fn run_incomplete_cleanup_loop(
    db: SqlitePool,
    secrets: Arc<SecretsCrypto>,
    incomplete_cleanup_days: i64,
    notify: Arc<Notify>,
    shutdown: CancellationToken,
) {
    let cutoff_seconds = incomplete_cleanup_days.saturating_mul(24 * 60 * 60);
    if cutoff_seconds <= 0 {
        return;
    }

    loop {
        if shutdown.is_cancelled() {
            break;
        }

        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        let cutoff_started_at = now.saturating_sub(cutoff_seconds);

        let stats =
            match tick_incomplete_cleanup(&db, &secrets, cutoff_started_at, now).await {
                Ok(v) => v,
                Err(error) => {
                    warn!(error = %error, "incomplete cleanup tick failed");
                    TickStats::default()
                }
            };

        if stats.any_activity() {
            info!(
                reconciled = stats.reconciled,
                processed = stats.processed,
                recovered_running = stats.recovered_running,
                deleted = stats.deleted,
                done = stats.done,
                retrying = stats.retrying,
                blocked = stats.blocked,
                abandoned = stats.abandoned,
                "incomplete cleanup tick"
            );
        }

        let sleep = match compute_sleep(&db, now, &stats).await {
            Ok(v) => v,
            Err(error) => {
                warn!(error = %error, "failed to compute incomplete cleanup sleep");
                Duration::from_secs(MAX_SLEEP_SECS)
            }
        };

        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = notify.notified() => {},
            _ = tokio::time::sleep(sleep) => {}
        }
    }
}

#[derive(Debug, Default)]
struct TickStats {
    reconciled: u64,
    processed: u64,
    recovered_running: u64,
    deleted: u64,
    done: u64,
    retrying: u64,
    blocked: u64,
    abandoned: u64,
    hit_reconcile_limit: bool,
    hit_process_limit: bool,
}

impl TickStats {
    fn any_activity(&self) -> bool {
        self.reconciled > 0 || self.processed > 0 || self.recovered_running > 0
    }
}

#[derive(Debug, serde::Deserialize)]
struct RunTargetSnapshot {
    node_id: String,
    target: RunTarget,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RunTarget {
    Webdav { base_url: String, secret_name: String },
    LocalDir { base_dir: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorKind {
    Network,
    Http,
    Auth,
    Config,
    Unknown,
}

impl ErrorKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Network => "network",
            Self::Http => "http",
            Self::Auth => "auth",
            Self::Config => "config",
            Self::Unknown => "unknown",
        }
    }
}

async fn tick_incomplete_cleanup(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    cutoff_started_at: i64,
    now: i64,
) -> Result<TickStats, anyhow::Error> {
    let mut stats = TickStats::default();

    stats.recovered_running = recover_stuck_running(db, now).await?;

    let (reconciled, hit_limit) = reconcile_cleanup_tasks(db, cutoff_started_at, now).await?;
    stats.reconciled = reconciled;
    stats.hit_reconcile_limit = hit_limit;

    let (process_stats, hit_process_limit) = process_due_tasks(db, secrets, now).await?;
    stats.processed = process_stats.processed;
    stats.hit_process_limit = hit_process_limit;

    stats.deleted = process_stats.deleted;
    stats.done = process_stats.done;
    stats.retrying = process_stats.retrying;
    stats.blocked = process_stats.blocked;
    stats.abandoned = process_stats.abandoned;

    Ok(stats)
}

async fn compute_sleep(db: &SqlitePool, now: i64, stats: &TickStats) -> Result<Duration, anyhow::Error> {
    if stats.hit_process_limit || stats.hit_reconcile_limit {
        return Ok(Duration::from_secs(SHORT_SLEEP_SECS));
    }

    let next_due_at = incomplete_cleanup_repo::next_due_at(db).await?;
    let Some(next_due_at) = next_due_at else {
        return Ok(Duration::from_secs(MAX_SLEEP_SECS));
    };

    if next_due_at <= now {
        return Ok(Duration::from_secs(SHORT_SLEEP_SECS));
    }

    let delta = next_due_at.saturating_sub(now) as u64;
    Ok(Duration::from_secs(std::cmp::min(delta, MAX_SLEEP_SECS)))
}

async fn recover_stuck_running(db: &SqlitePool, now: i64) -> Result<u64, anyhow::Error> {
    use sqlx::Row;

    let cutoff = now.saturating_sub(RUNNING_TTL_SECS);
    let next_attempt_at = now.saturating_add(60);

    let rows = sqlx::query(
        r#"
        UPDATE incomplete_cleanup_tasks
        SET status = 'retrying',
            updated_at = ?,
            next_attempt_at = ?,
            last_error_kind = 'unknown',
            last_error = 'stuck running; recovered'
        WHERE status = 'running'
          AND last_attempt_at IS NOT NULL
          AND last_attempt_at < ?
        RETURNING run_id
        "#,
    )
    .bind(now)
    .bind(next_attempt_at)
    .bind(cutoff)
    .fetch_all(db)
    .await?;

    for row in &rows {
        let run_id = row.get::<String, _>("run_id");
        let _ = incomplete_cleanup_repo::append_event(
            db,
            &run_id,
            "warn",
            "failed",
            "stuck running; recovered",
            None,
            now,
        )
        .await;
    }

    Ok(rows.len() as u64)
}

async fn reconcile_cleanup_tasks(
    db: &SqlitePool,
    cutoff_started_at: i64,
    now: i64,
) -> Result<(u64, bool), anyhow::Error> {
    let candidates =
        runs_repo::list_incomplete_cleanup_candidates(db, cutoff_started_at, RECONCILE_BATCH_LIMIT)
            .await?;
    let hit_limit = candidates.len() >= RECONCILE_BATCH_LIMIT as usize;

    let mut inserted = 0_u64;
    for run in candidates {
        debug!(run_id = %run.id, job_id = %run.job_id, "incomplete cleanup candidate");

        let snapshot = match runs_repo::get_run_target_snapshot(db, &run.id).await? {
            Some(v) => v,
            None => {
                let job = match jobs_repo::get_job(db, &run.job_id).await {
                    Ok(Some(v)) => v,
                    Ok(None) => continue,
                    Err(error) => {
                        warn!(job_id = %run.job_id, run_id = %run.id, error = %error, "failed to load job while reconciling cleanup tasks");
                        continue;
                    }
                };

                let spec = match job_spec::parse_value(&job.spec) {
                    Ok(v) => v,
                    Err(error) => {
                        warn!(job_id = %job.id, run_id = %run.id, error = %error, "invalid job spec while reconciling cleanup tasks");
                        continue;
                    }
                };

                if let Err(error) = job_spec::validate(&spec) {
                    warn!(job_id = %job.id, run_id = %run.id, error = %error, "invalid job spec while reconciling cleanup tasks");
                    continue;
                }

                let node_id = job.agent_id.as_deref().unwrap_or(HUB_NODE_ID);
                let snapshot = target_snapshot::build_run_target_snapshot(node_id, &spec);
                let _ = runs_repo::set_run_target_snapshot(db, &run.id, snapshot.clone()).await;
                snapshot
            }
        };

        let snapshot_json = serde_json::to_string(&snapshot)?;
        let parsed = match serde_json::from_value::<RunTargetSnapshot>(snapshot) {
            Ok(v) => v,
            Err(error) => {
                warn!(run_id = %run.id, error = %error, "invalid target snapshot while reconciling cleanup tasks");
                continue;
            }
        };

        let target_type = match parsed.target {
            RunTarget::Webdav { .. } => "webdav",
            RunTarget::LocalDir { .. } => "local_dir",
        };

        if incomplete_cleanup_repo::upsert_task_if_missing(
            db,
            &run.id,
            &run.job_id,
            &parsed.node_id,
            target_type,
            &snapshot_json,
            now,
        )
        .await?
        {
            inserted = inserted.saturating_add(1);
        }
    }

    Ok((inserted, hit_limit))
}

#[derive(Debug, Default)]
struct ProcessStats {
    processed: u64,
    deleted: u64,
    done: u64,
    retrying: u64,
    blocked: u64,
    abandoned: u64,
}

async fn process_due_tasks(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    now: i64,
) -> Result<(ProcessStats, bool), anyhow::Error> {
    let mut stats = ProcessStats::default();

    for _ in 0..PROCESS_BATCH_LIMIT {
        let Some(task) = incomplete_cleanup_repo::claim_next_due(db, now).await? else {
            break;
        };

        stats.processed = stats.processed.saturating_add(1);
        if let Err(error) = process_task(db, secrets, &task, now, &mut stats).await {
            warn!(run_id = %task.run_id, error = %error, "incomplete cleanup task processing failed");
        }
    }

    let hit_limit = stats.processed >= PROCESS_BATCH_LIMIT as u64;
    Ok((stats, hit_limit))
}

async fn process_task(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    task: &incomplete_cleanup_repo::CleanupTaskRow,
    now: i64,
    stats: &mut ProcessStats,
) -> Result<(), anyhow::Error> {
    let attempt_started = std::time::Instant::now();

    let _ = incomplete_cleanup_repo::append_event(
        db,
        &task.run_id,
        "info",
        "attempt",
        &format!("attempt {}", task.attempts),
        Some(serde_json::json!({ "attempt": task.attempts })),
        now,
    )
    .await;

    let parsed = serde_json::from_value::<RunTargetSnapshot>(task.target_snapshot.clone());
    let result = match parsed {
        Ok(parsed) => match parsed.target {
            RunTarget::LocalDir { base_dir } => {
                let base_dir = base_dir.clone();
                let job_id = task.job_id.clone();
                let run_id = task.run_id.clone();
                match tokio::task::spawn_blocking(move || {
                    cleanup_local_dir_run(&base_dir, &job_id, &run_id)
                })
                .await
                {
                    Ok(v) => v,
                    Err(error) => CleanupResult::Failed {
                        kind: ErrorKind::Unknown,
                        error: anyhow::anyhow!("join error: {error}"),
                    },
                }
            }
            RunTarget::Webdav {
                base_url,
                secret_name,
            } => cleanup_webdav_run(
                db,
                secrets,
                &task.node_id,
                &base_url,
                &secret_name,
                task,
            )
            .await,
        },
        Err(error) => CleanupResult::Failed {
            kind: ErrorKind::Config,
            error: anyhow::anyhow!("invalid target snapshot: {error}"),
        },
    };

    let duration_ms = attempt_started.elapsed().as_millis() as i64;

    match result {
        CleanupResult::SkipComplete => {
            incomplete_cleanup_repo::mark_done(db, &task.run_id, now).await?;
            let _ = incomplete_cleanup_repo::append_event(
                db,
                &task.run_id,
                "info",
                "skip_complete",
                "complete marker present; skip cleanup",
                Some(serde_json::json!({ "duration_ms": duration_ms })),
                now,
            )
            .await;
            stats.done = stats.done.saturating_add(1);
        }
        CleanupResult::SkipNotFound { message } => {
            incomplete_cleanup_repo::mark_done(db, &task.run_id, now).await?;
            let _ = incomplete_cleanup_repo::append_event(
                db,
                &task.run_id,
                "info",
                "skip_not_found",
                message,
                Some(serde_json::json!({ "duration_ms": duration_ms })),
                now,
            )
            .await;
            stats.done = stats.done.saturating_add(1);
        }
        CleanupResult::Deleted => {
            incomplete_cleanup_repo::mark_done(db, &task.run_id, now).await?;
            let _ = incomplete_cleanup_repo::append_event(
                db,
                &task.run_id,
                "info",
                "deleted",
                "deleted incomplete target run",
                Some(serde_json::json!({ "duration_ms": duration_ms })),
                now,
            )
            .await;
            stats.deleted = stats.deleted.saturating_add(1);
        }
        CleanupResult::Failed { kind, error } => {
            let last_error = sanitize_error_string(&error.to_string());
            let last_error_kind = kind.as_str();

            if should_abandon(task, now) {
                incomplete_cleanup_repo::mark_abandoned(
                    db,
                    &task.run_id,
                    last_error_kind,
                    &last_error,
                    now,
                )
                .await?;
                let _ = incomplete_cleanup_repo::append_event(
                    db,
                    &task.run_id,
                    "error",
                    "abandoned",
                    &format!("abandoned: {last_error}"),
                    Some(serde_json::json!({
                        "duration_ms": duration_ms,
                        "error_kind": last_error_kind,
                    })),
                    now,
                )
                .await;
                stats.abandoned = stats.abandoned.saturating_add(1);
                return Ok(());
            }

            let next_attempt_at = now.saturating_add(backoff_seconds(&task.run_id, task.attempts, kind));
            if matches!(kind, ErrorKind::Auth | ErrorKind::Config) {
                incomplete_cleanup_repo::mark_blocked(
                    db,
                    &task.run_id,
                    next_attempt_at,
                    last_error_kind,
                    &last_error,
                    now,
                )
                .await?;
                let _ = incomplete_cleanup_repo::append_event(
                    db,
                    &task.run_id,
                    "warn",
                    "blocked",
                    &format!("blocked: {last_error}"),
                    Some(serde_json::json!({
                        "duration_ms": duration_ms,
                        "error_kind": last_error_kind,
                        "next_attempt_at": next_attempt_at,
                    })),
                    now,
                )
                .await;
                stats.blocked = stats.blocked.saturating_add(1);
            } else {
                incomplete_cleanup_repo::mark_retrying(
                    db,
                    &task.run_id,
                    next_attempt_at,
                    last_error_kind,
                    &last_error,
                    now,
                )
                .await?;
                let _ = incomplete_cleanup_repo::append_event(
                    db,
                    &task.run_id,
                    "warn",
                    "failed",
                    &format!("failed: {last_error}"),
                    Some(serde_json::json!({
                        "duration_ms": duration_ms,
                        "error_kind": last_error_kind,
                        "next_attempt_at": next_attempt_at,
                    })),
                    now,
                )
                .await;
                stats.retrying = stats.retrying.saturating_add(1);
            }
        }
    }

    Ok(())
}

fn should_abandon(task: &incomplete_cleanup_repo::CleanupTaskRow, now: i64) -> bool {
    if task.attempts >= MAX_ATTEMPTS {
        return true;
    }

    let age = now.saturating_sub(task.created_at);
    age >= MAX_AGE_SECS
}

fn sanitize_error_string(s: &str) -> String {
    const MAX_LEN: usize = 500;

    let s = s.replace(['\n', '\r'], " ");
    let s = s.trim();
    if s.len() <= MAX_LEN {
        return s.to_string();
    }

    let mut out = s[..MAX_LEN].to_string();
    out.push('…');
    out
}

fn backoff_seconds(run_id: &str, attempts: i64, kind: ErrorKind) -> i64 {
    let attempts = attempts.max(1);

    let (base, cap, max_jitter) = match kind {
        ErrorKind::Network | ErrorKind::Http => (60_i64, 6 * 60 * 60, 30_i64),
        ErrorKind::Unknown => (5 * 60, 6 * 60 * 60, 60_i64),
        ErrorKind::Auth | ErrorKind::Config => (6 * 60 * 60, 24 * 60 * 60, 10 * 60_i64),
    };

    let exp = 1_i64 << (attempts.saturating_sub(1).min(30) as u32);
    let delay = base.saturating_mul(exp).min(cap);
    delay.saturating_add(jitter_seconds(run_id, attempts, max_jitter))
}

fn jitter_seconds(run_id: &str, attempts: i64, max_jitter: i64) -> i64 {
    if max_jitter <= 0 {
        return 0;
    }

    let mut hash = 0_u64;
    for byte in run_id.as_bytes() {
        hash = hash.wrapping_mul(131).wrapping_add(*byte as u64);
    }
    hash = hash.wrapping_add(attempts as u64 * 97);
    (hash % max_jitter as u64) as i64
}

#[derive(Debug)]
enum CleanupResult {
    SkipComplete,
    SkipNotFound { message: &'static str },
    Deleted,
    Failed { kind: ErrorKind, error: anyhow::Error },
}

fn cleanup_local_dir_run(
    base_dir: &str,
    job_id: &str,
    run_id: &str,
) -> CleanupResult {
    use bastion_backup::{COMPLETE_NAME, ENTRIES_INDEX_NAME, MANIFEST_NAME};

    let run_dir = std::path::Path::new(base_dir).join(job_id).join(run_id);
    if !run_dir.exists() {
        return CleanupResult::SkipNotFound {
            message: "local run dir missing; nothing to cleanup",
        };
    }
    if run_dir.join(COMPLETE_NAME).exists() {
        return CleanupResult::SkipComplete;
    }

    let mut looks_like_bastion = false;
    if run_dir.join(MANIFEST_NAME).exists() || run_dir.join(ENTRIES_INDEX_NAME).exists() {
        looks_like_bastion = true;
    } else if let Ok(entries) = std::fs::read_dir(&run_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("payload.part") || name.ends_with(".partial") {
                looks_like_bastion = true;
                break;
            }
        }
    }
    if !looks_like_bastion {
        return CleanupResult::SkipNotFound {
            message: "local run dir did not look like bastion data; skip cleanup",
        };
    }

    match std::fs::remove_dir_all(&run_dir) {
        Ok(()) => CleanupResult::Deleted,
        Err(error) => CleanupResult::Failed {
            kind: ErrorKind::Unknown,
            error: anyhow::Error::from(error),
        },
    }
}

async fn cleanup_webdav_run(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    node_id: &str,
    base_url: &str,
    secret_name: &str,
    task: &incomplete_cleanup_repo::CleanupTaskRow,
) -> CleanupResult {
    match cleanup_webdav_run_inner(db, secrets, node_id, base_url, secret_name, task).await {
        Ok(v) => v,
        Err(error) => CleanupResult::Failed {
            kind: classify_error(&error),
            error,
        },
    }
}

async fn cleanup_webdav_run_inner(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    node_id: &str,
    base_url: &str,
    secret_name: &str,
    task: &incomplete_cleanup_repo::CleanupTaskRow,
) -> Result<CleanupResult, anyhow::Error> {
    use bastion_backup::COMPLETE_NAME;

    let cred_bytes = secrets_repo::get_secret(db, secrets, node_id, "webdav", secret_name)
        .await?
        .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
    let credentials = WebdavCredentials::from_json(&cred_bytes)?;

    let mut base_url = Url::parse(base_url)?;
    if !base_url.path().ends_with('/') {
        base_url.set_path(&format!("{}/", base_url.path()));
    }

    let client = WebdavClient::new(base_url.clone(), credentials)?;
    let job_url = base_url.join(&format!("{}/", task.job_id))?;
    let run_url = job_url.join(&format!("{}/", task.run_id))?;
    let complete_url = run_url.join(COMPLETE_NAME)?;
    if client.head_size(&complete_url).await?.is_some() {
        return Ok(CleanupResult::SkipComplete);
    }

    match client.delete(&run_url).await {
        Ok(true) => Ok(CleanupResult::Deleted),
        Ok(false) => Ok(CleanupResult::SkipNotFound {
            message: "remote run dir missing; nothing to cleanup",
        }),
        Err(error) => Ok(CleanupResult::Failed {
            kind: classify_error(&error),
            error,
        }),
    }
}

fn classify_error(error: &anyhow::Error) -> ErrorKind {
    let msg = error.to_string();

    if msg.contains("missing webdav secret") || msg.contains("invalid target snapshot") {
        return ErrorKind::Config;
    }

    if msg.contains("HTTP 401")
        || msg.contains("HTTP 403")
        || msg.contains(" Unauthorized")
        || msg.contains(" Forbidden")
    {
        return ErrorKind::Auth;
    }

    if msg.contains("HTTP ") {
        return ErrorKind::Http;
    }

    if msg.contains("error sending request")
        || msg.contains("timed out")
        || msg.contains("dns")
        || msg.contains("connection")
    {
        return ErrorKind::Network;
    }

    ErrorKind::Unknown
}
