use std::sync::Arc;
use std::time::Duration;

use sqlx::SqlitePool;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use url::Url;

use bastion_core::HUB_NODE_ID;
use bastion_core::agent_protocol::{HubToAgentMessageV1, PROTOCOL_VERSION, SnapshotDeleteTaskV1};
use bastion_storage::artifact_delete_repo;
use bastion_storage::run_artifacts_repo;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;
use bastion_targets::{WebdavClient, WebdavCredentials, WebdavHttpError};

use crate::agent_manager::AgentManager;
use crate::error_envelope::{
    envelope, insert_error_envelope, origin, retriable, retriable_with_reason, transport,
    with_context_param,
};

const PROCESS_BATCH_LIMIT: u32 = 20;

const MAX_SLEEP_SECS: u64 = 60 * 60;
const SHORT_SLEEP_SECS: u64 = 5;

const RUNNING_TTL_SECS: i64 = 30 * 60;
const MAX_ATTEMPTS: i64 = 20;
const MAX_AGE_SECS: i64 = 30 * 24 * 60 * 60;

pub(super) async fn run_artifact_delete_loop(
    db: SqlitePool,
    secrets: Arc<SecretsCrypto>,
    agent_manager: AgentManager,
    notify: Arc<Notify>,
    shutdown: CancellationToken,
) {
    loop {
        if shutdown.is_cancelled() {
            break;
        }

        let now = time::OffsetDateTime::now_utc().unix_timestamp();

        let stats = match tick(&db, &secrets, &agent_manager, now).await {
            Ok(v) => v,
            Err(error) => {
                warn!(error = %error, "artifact delete tick failed");
                TickStats::default()
            }
        };

        if stats.any_activity() {
            info!(
                recovered_running = stats.recovered_running,
                processed = stats.processed,
                deleted = stats.deleted,
                missing = stats.missing,
                retrying = stats.retrying,
                blocked = stats.blocked,
                abandoned = stats.abandoned,
                "artifact delete tick"
            );
        }

        let sleep = match compute_sleep(&db, now, &stats).await {
            Ok(v) => v,
            Err(error) => {
                warn!(error = %error, "failed to compute artifact delete sleep");
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
    recovered_running: u64,
    processed: u64,
    deleted: u64,
    missing: u64,
    retrying: u64,
    blocked: u64,
    abandoned: u64,
    hit_process_limit: bool,
}

impl TickStats {
    fn any_activity(&self) -> bool {
        self.recovered_running > 0 || self.processed > 0
    }
}

async fn tick(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    now: i64,
) -> Result<TickStats, anyhow::Error> {
    let recovered_running = recover_stuck_running(db, now).await?;
    let (pstats, hit_limit) = process_due_tasks(db, secrets, agent_manager, now).await?;

    Ok(TickStats {
        recovered_running,
        processed: pstats.processed,
        deleted: pstats.deleted,
        missing: pstats.missing,
        retrying: pstats.retrying,
        blocked: pstats.blocked,
        abandoned: pstats.abandoned,
        hit_process_limit: hit_limit,
    })
}

async fn compute_sleep(
    db: &SqlitePool,
    now: i64,
    stats: &TickStats,
) -> Result<Duration, anyhow::Error> {
    if stats.hit_process_limit {
        return Ok(Duration::from_secs(SHORT_SLEEP_SECS));
    }

    let next_due_at = artifact_delete_repo::next_due_at(db).await?;
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
        UPDATE artifact_delete_tasks
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
        let _ = artifact_delete_repo::append_event(
            db,
            &run_id,
            "warn",
            "failed",
            "stuck running; recovered",
            None,
            now,
        )
        .await;
        let _ = run_artifacts_repo::mark_run_artifact_deleting_with_error(
            db,
            &run_id,
            ErrorKind::Unknown.as_str(),
            "stuck running; recovered",
            now,
            now,
        )
        .await;
    }

    Ok(rows.len() as u64)
}

#[derive(Debug, Default)]
struct ProcessStats {
    processed: u64,
    deleted: u64,
    missing: u64,
    retrying: u64,
    blocked: u64,
    abandoned: u64,
}

async fn process_due_tasks(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    now: i64,
) -> Result<(ProcessStats, bool), anyhow::Error> {
    let mut stats = ProcessStats::default();

    for _ in 0..PROCESS_BATCH_LIMIT {
        let Some(task) = artifact_delete_repo::claim_next_due(db, now).await? else {
            break;
        };

        stats.processed = stats.processed.saturating_add(1);
        if let Err(error) = process_task(db, secrets, agent_manager, &task, now, &mut stats).await {
            warn!(run_id = %task.run_id, error = %error, "artifact delete task processing failed");
        }
    }

    let hit_limit = stats.processed >= PROCESS_BATCH_LIMIT as u64;
    Ok((stats, hit_limit))
}

#[derive(Debug, serde::Deserialize)]
struct RunTargetSnapshot {
    node_id: String,
    target: RunTarget,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RunTarget {
    Webdav {
        base_url: String,
        secret_name: String,
    },
    LocalDir {
        base_dir: String,
    },
}

async fn process_task(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    task: &artifact_delete_repo::ArtifactDeleteTaskRow,
    now: i64,
    stats: &mut ProcessStats,
) -> Result<(), anyhow::Error> {
    let attempt_started = std::time::Instant::now();

    let _ = artifact_delete_repo::append_event(
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
                if parsed.node_id == HUB_NODE_ID {
                    let base_dir = base_dir.clone();
                    let job_id = task.job_id.clone();
                    let run_id = task.run_id.clone();
                    match tokio::task::spawn_blocking(move || {
                        delete_local_dir_snapshot(&base_dir, &job_id, &run_id)
                    })
                    .await
                    {
                        Ok(v) => v,
                        Err(error) => DeleteResult::Failed {
                            kind: ErrorKind::Unknown,
                            error: anyhow::anyhow!("join error: {error}"),
                        },
                    }
                } else {
                    let msg = HubToAgentMessageV1::SnapshotDeleteTask {
                        v: PROTOCOL_VERSION,
                        task: SnapshotDeleteTaskV1 {
                            run_id: task.run_id.clone(),
                            job_id: task.job_id.clone(),
                            base_dir: base_dir.clone(),
                        },
                    };
                    match agent_manager.send_json(&parsed.node_id, &msg).await {
                        Ok(()) => DeleteResult::Dispatched,
                        Err(error) => DeleteResult::Failed {
                            kind: ErrorKind::Network,
                            error,
                        },
                    }
                }
            }
            RunTarget::Webdav {
                base_url,
                secret_name,
            } => {
                delete_webdav_snapshot(db, secrets, &parsed.node_id, &base_url, &secret_name, task)
                    .await
            }
        },
        Err(error) => DeleteResult::Failed {
            kind: ErrorKind::Config,
            error: anyhow::anyhow!("invalid target snapshot: {error}"),
        },
    };

    let duration_ms = attempt_started.elapsed().as_millis() as i64;

    match result {
        DeleteResult::Dispatched => {
            let _ = artifact_delete_repo::append_event(
                db,
                &task.run_id,
                "info",
                "dispatched",
                "dispatched to agent",
                Some(serde_json::json!({ "duration_ms": duration_ms, "agent_id": task.node_id })),
                now,
            )
            .await;
        }
        DeleteResult::NotFound { message } => {
            artifact_delete_repo::mark_done(db, &task.run_id, now).await?;
            let _ = artifact_delete_repo::append_event(
                db,
                &task.run_id,
                "info",
                "not_found",
                message,
                Some(serde_json::json!({ "duration_ms": duration_ms })),
                now,
            )
            .await;
            let _ = run_artifacts_repo::mark_run_artifact_missing(db, &task.run_id, now).await;
            stats.missing = stats.missing.saturating_add(1);
        }
        DeleteResult::Deleted => {
            artifact_delete_repo::mark_done(db, &task.run_id, now).await?;
            let _ = artifact_delete_repo::append_event(
                db,
                &task.run_id,
                "info",
                "deleted",
                "deleted snapshot artifacts",
                Some(serde_json::json!({ "duration_ms": duration_ms })),
                now,
            )
            .await;
            let _ = run_artifacts_repo::mark_run_artifact_deleted(db, &task.run_id, now).await;
            stats.deleted = stats.deleted.saturating_add(1);
        }
        DeleteResult::Failed { kind, error } => {
            let last_error = sanitize_error_string(&error.to_string());
            let last_error_kind = kind.as_str();

            if should_abandon(task, now) {
                artifact_delete_repo::mark_abandoned(
                    db,
                    &task.run_id,
                    last_error_kind,
                    &last_error,
                    now,
                )
                .await?;
                let _ = artifact_delete_repo::append_event(
                    db,
                    &task.run_id,
                    "error",
                    "abandoned",
                    &format!("abandoned: {last_error}"),
                    Some(build_failed_event_fields(
                        task,
                        kind,
                        "abandoned",
                        duration_ms,
                        None,
                        &error,
                        &last_error,
                    )),
                    now,
                )
                .await;
                let _ = run_artifacts_repo::mark_run_artifact_error(
                    db,
                    &task.run_id,
                    last_error_kind,
                    &last_error,
                    now,
                    now,
                )
                .await;
                stats.abandoned = stats.abandoned.saturating_add(1);
                return Ok(());
            }

            let next_attempt_at =
                now.saturating_add(backoff_seconds(&task.run_id, task.attempts, kind));
            if matches!(kind, ErrorKind::Auth | ErrorKind::Config) {
                artifact_delete_repo::mark_blocked(
                    db,
                    &task.run_id,
                    next_attempt_at,
                    last_error_kind,
                    &last_error,
                    now,
                )
                .await?;
                let _ = artifact_delete_repo::append_event(
                    db,
                    &task.run_id,
                    "warn",
                    "blocked",
                    &format!("blocked: {last_error}"),
                    Some(build_failed_event_fields(
                        task,
                        kind,
                        "blocked",
                        duration_ms,
                        Some(next_attempt_at),
                        &error,
                        &last_error,
                    )),
                    now,
                )
                .await;
                let _ = run_artifacts_repo::mark_run_artifact_error(
                    db,
                    &task.run_id,
                    last_error_kind,
                    &last_error,
                    now,
                    now,
                )
                .await;
                stats.blocked = stats.blocked.saturating_add(1);
            } else {
                artifact_delete_repo::mark_retrying(
                    db,
                    &task.run_id,
                    next_attempt_at,
                    last_error_kind,
                    &last_error,
                    now,
                )
                .await?;
                let _ = artifact_delete_repo::append_event(
                    db,
                    &task.run_id,
                    "warn",
                    "failed",
                    &format!("failed: {last_error}"),
                    Some(build_failed_event_fields(
                        task,
                        kind,
                        "failed",
                        duration_ms,
                        Some(next_attempt_at),
                        &error,
                        &last_error,
                    )),
                    now,
                )
                .await;
                let _ = run_artifacts_repo::mark_run_artifact_deleting_with_error(
                    db,
                    &task.run_id,
                    last_error_kind,
                    &last_error,
                    now,
                    now,
                )
                .await;
                stats.retrying = stats.retrying.saturating_add(1);
            }
        }
    }

    Ok(())
}

fn should_abandon(task: &artifact_delete_repo::ArtifactDeleteTaskRow, now: i64) -> bool {
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

#[derive(Debug, Clone, Copy)]
enum ErrorKind {
    Config,
    Auth,
    Network,
    Http,
    Unknown,
}

impl ErrorKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Auth => "auth",
            Self::Network => "network",
            Self::Http => "http",
            Self::Unknown => "unknown",
        }
    }
}

fn hint_for_error_kind(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Config => "target snapshot configuration is invalid; verify target settings",
        ErrorKind::Auth => {
            "target authentication/permission failure; verify credentials and access"
        }
        ErrorKind::Network => "temporary network failure; verify connectivity and retry",
        ErrorKind::Http => {
            "remote service returned an unexpected HTTP response; inspect status and response body"
        }
        ErrorKind::Unknown => "unknown delete failure; inspect task error details and service logs",
    }
}

fn hint_key_for_error_kind(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Config => "diagnostics.hint.artifact_delete.config",
        ErrorKind::Auth => "diagnostics.hint.artifact_delete.auth",
        ErrorKind::Network => "diagnostics.hint.artifact_delete.network",
        ErrorKind::Http => "diagnostics.hint.artifact_delete.http",
        ErrorKind::Unknown => "diagnostics.hint.artifact_delete.unknown",
    }
}

fn message_key_for_error_kind(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Config => "diagnostics.message.artifact_delete.config",
        ErrorKind::Auth => "diagnostics.message.artifact_delete.auth",
        ErrorKind::Network => "diagnostics.message.artifact_delete.network",
        ErrorKind::Http => "diagnostics.message.artifact_delete.http",
        ErrorKind::Unknown => "diagnostics.message.artifact_delete.unknown",
    }
}

fn retriable_for_error_kind(kind: ErrorKind) -> bool {
    matches!(
        kind,
        ErrorKind::Network | ErrorKind::Http | ErrorKind::Unknown
    )
}

fn webdav_http_meta(error: &anyhow::Error) -> Option<(u16, String, Option<u64>)> {
    error
        .chain()
        .find_map(|cause| cause.downcast_ref::<WebdavHttpError>())
        .map(|http| {
            (
                http.status.as_u16(),
                http.message.clone(),
                http.retry_after.map(|value| value.as_secs()),
            )
        })
}

fn transport_for_delete_error(
    task: &artifact_delete_repo::ArtifactDeleteTaskRow,
    error: &anyhow::Error,
) -> bastion_core::error_envelope::ErrorTransportV1 {
    match task.target_type {
        artifact_delete_repo::DeleteTargetType::Webdav => {
            let mut out = transport("http").with_provider("webdav");
            if let Some((status_code, status_text, _)) = webdav_http_meta(error) {
                out = out.with_status_code(status_code);
                if !status_text.is_empty() {
                    out = out.with_status_text(status_text);
                }
            }
            out
        }
        artifact_delete_repo::DeleteTargetType::LocalDir => {
            if task.node_id == HUB_NODE_ID {
                transport("file")
            } else {
                transport("internal").with_provider("agent")
            }
        }
    }
}

fn build_failed_event_fields(
    task: &artifact_delete_repo::ArtifactDeleteTaskRow,
    kind: ErrorKind,
    event_kind: &'static str,
    duration_ms: i64,
    next_attempt_at: Option<i64>,
    error: &anyhow::Error,
    last_error: &str,
) -> serde_json::Value {
    let last_error_kind = kind.as_str();
    let hint = hint_for_error_kind(kind);

    let mut fields = serde_json::Map::new();
    fields.insert("duration_ms".to_string(), serde_json::json!(duration_ms));
    fields.insert(
        "error_kind".to_string(),
        serde_json::Value::String(last_error_kind.to_string()),
    );
    fields.insert(
        "hint".to_string(),
        serde_json::Value::String(hint.to_string()),
    );
    if let Some(next_attempt_at) = next_attempt_at {
        fields.insert(
            "next_attempt_at".to_string(),
            serde_json::json!(next_attempt_at),
        );
    }

    let mut retry = if retriable_for_error_kind(kind) {
        retriable_with_reason(true, last_error_kind)
    } else {
        retriable(false)
    };
    if let Some((_, _, Some(retry_after_sec))) = webdav_http_meta(error) {
        retry = retry.with_retry_after_sec(retry_after_sec);
    }

    let mut env = envelope(
        format!("maintenance.artifact_delete.{event_kind}.{last_error_kind}"),
        last_error_kind,
        retry,
        hint_key_for_error_kind(kind),
        message_key_for_error_kind(kind),
        transport_for_delete_error(task, error),
    )
    .with_origin(origin("maintenance", "artifact_delete", event_kind))
    .with_stage("cleanup");
    env = with_context_param(env, "run_id", task.run_id.clone());
    env = with_context_param(env, "job_id", task.job_id.clone());
    env = with_context_param(env, "target_type", task.target_type.as_str());
    env = with_context_param(env, "error", last_error.to_string());
    if let Some(next_attempt_at) = next_attempt_at {
        env = with_context_param(env, "next_attempt_at", next_attempt_at);
    }
    if let Some((status_code, _, retry_after_sec)) = webdav_http_meta(error) {
        env = with_context_param(env, "http_status", status_code);
        if let Some(retry_after_sec) = retry_after_sec {
            env = with_context_param(env, "retry_after_secs", retry_after_sec);
        }
    }
    insert_error_envelope(&mut fields, env);

    serde_json::Value::Object(fields)
}

#[derive(Debug)]
enum DeleteResult {
    Dispatched,
    Deleted,
    NotFound {
        message: &'static str,
    },
    Failed {
        kind: ErrorKind,
        error: anyhow::Error,
    },
}

fn delete_local_dir_snapshot(base_dir: &str, job_id: &str, run_id: &str) -> DeleteResult {
    use bastion_backup::{COMPLETE_NAME, ENTRIES_INDEX_NAME, MANIFEST_NAME};

    let run_dir = std::path::Path::new(base_dir).join(job_id).join(run_id);
    if !run_dir.exists() {
        return DeleteResult::NotFound {
            message: "local snapshot dir missing; nothing to delete",
        };
    }

    let mut looks_like_bastion = false;
    if run_dir.join(COMPLETE_NAME).exists()
        || run_dir.join(MANIFEST_NAME).exists()
        || run_dir.join(ENTRIES_INDEX_NAME).exists()
    {
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
        return DeleteResult::Failed {
            kind: ErrorKind::Config,
            error: anyhow::anyhow!("local snapshot dir did not look like bastion data"),
        };
    }

    match std::fs::remove_dir_all(&run_dir) {
        Ok(()) => DeleteResult::Deleted,
        Err(error) => DeleteResult::Failed {
            kind: ErrorKind::Unknown,
            error: anyhow::Error::from(error),
        },
    }
}

async fn delete_webdav_snapshot(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    node_id: &str,
    base_url: &str,
    secret_name: &str,
    task: &artifact_delete_repo::ArtifactDeleteTaskRow,
) -> DeleteResult {
    match delete_webdav_snapshot_inner(db, secrets, node_id, base_url, secret_name, task).await {
        Ok(v) => v,
        Err(error) => DeleteResult::Failed {
            kind: classify_error(&error),
            error,
        },
    }
}

async fn delete_webdav_snapshot_inner(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    node_id: &str,
    base_url: &str,
    secret_name: &str,
    task: &artifact_delete_repo::ArtifactDeleteTaskRow,
) -> Result<DeleteResult, anyhow::Error> {
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

    match client.delete(&run_url).await {
        Ok(true) => Ok(DeleteResult::Deleted),
        Ok(false) => Ok(DeleteResult::NotFound {
            message: "remote snapshot dir missing; nothing to delete",
        }),
        Err(error) => Ok(DeleteResult::Failed {
            kind: classify_error(&error),
            error,
        }),
    }
}

fn classify_error(error: &anyhow::Error) -> ErrorKind {
    if let Some(http) = error
        .chain()
        .find_map(|cause| cause.downcast_ref::<WebdavHttpError>())
    {
        return match http.status.as_u16() {
            401 | 403 => ErrorKind::Auth,
            408 | 429 | 502 | 503 | 504 => ErrorKind::Network,
            500..=599 => ErrorKind::Network,
            400..=499 => ErrorKind::Http,
            _ => ErrorKind::Unknown,
        };
    }

    let msg = error
        .chain()
        .map(|cause| cause.to_string().to_lowercase())
        .collect::<Vec<_>>()
        .join(" | ");

    if msg.contains("missing webdav secret") || msg.contains("invalid target snapshot") {
        return ErrorKind::Config;
    }

    if msg.contains("http 401")
        || msg.contains("http 403")
        || msg.contains("unauthorized")
        || msg.contains("forbidden")
    {
        return ErrorKind::Auth;
    }

    if msg.contains("http 408")
        || msg.contains("http 429")
        || msg.contains("http 500")
        || msg.contains("http 502")
        || msg.contains("http 503")
        || msg.contains("http 504")
    {
        return ErrorKind::Network;
    }

    if msg.contains("http ") {
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

#[cfg(test)]
mod tests {
    use bastion_storage::artifact_delete_repo;
    use tempfile::TempDir;

    use super::{
        ErrorKind, backoff_seconds, build_failed_event_fields, classify_error,
        delete_local_dir_snapshot, hint_for_error_kind, sanitize_error_string,
    };

    fn dummy_task(
        target_type: artifact_delete_repo::DeleteTargetType,
        node_id: &str,
    ) -> artifact_delete_repo::ArtifactDeleteTaskRow {
        artifact_delete_repo::ArtifactDeleteTaskRow {
            run_id: "run".to_string(),
            job_id: "job".to_string(),
            node_id: node_id.to_string(),
            target_type,
            target_snapshot: serde_json::json!({}),
            attempts: 1,
            created_at: 0,
        }
    }

    #[test]
    fn sanitize_error_string_trims_and_single_lines() {
        let s = " hello\nworld\r\n ";
        assert_eq!(sanitize_error_string(s), "hello world");
    }

    #[test]
    fn backoff_seconds_is_stable_and_increases() {
        let run_id = "r1";
        let a1 = backoff_seconds(run_id, 1, ErrorKind::Network);
        let a2 = backoff_seconds(run_id, 2, ErrorKind::Network);
        assert!(a2 >= a1);
    }

    #[test]
    fn local_delete_requires_bastion_markers() {
        let tmp = TempDir::new().expect("tmp");
        let base = tmp.path();
        let job_id = "job";
        let run_id = "run";
        let dir = base.join(job_id).join(run_id);
        std::fs::create_dir_all(&dir).expect("mkdir");

        // Not a bastion dir -> config failure
        match delete_local_dir_snapshot(base.to_str().unwrap(), job_id, run_id) {
            super::DeleteResult::Failed { kind, .. } => assert!(matches!(kind, ErrorKind::Config)),
            other => panic!("unexpected result: {other:?}"),
        }

        // Add marker -> delete succeeds
        std::fs::write(dir.join(bastion_backup::COMPLETE_NAME), b"{}").expect("write");
        match delete_local_dir_snapshot(base.to_str().unwrap(), job_id, run_id) {
            super::DeleteResult::Deleted => {}
            other => panic!("unexpected result: {other:?}"),
        }
        assert!(!dir.exists());
    }

    #[test]
    fn classify_error_uses_http_message_hints() {
        let auth = anyhow::anyhow!("webdav request failed: HTTP 401: unauthorized");
        assert!(matches!(classify_error(&auth), ErrorKind::Auth));

        let net = anyhow::anyhow!("webdav request failed: HTTP 503: busy");
        assert!(matches!(classify_error(&net), ErrorKind::Network));
    }

    #[test]
    fn hint_for_error_kind_is_actionable() {
        assert!(hint_for_error_kind(ErrorKind::Auth).contains("credentials"));
        assert!(hint_for_error_kind(ErrorKind::Config).contains("configuration"));
        assert!(hint_for_error_kind(ErrorKind::Network).contains("connectivity"));
        assert!(hint_for_error_kind(ErrorKind::Http).contains("HTTP"));
    }

    #[test]
    fn failed_event_fields_include_canonical_envelope_for_webdav_target() {
        let task = dummy_task(artifact_delete_repo::DeleteTargetType::Webdav, "hub");
        let error = anyhow::anyhow!("webdav request failed: HTTP 503: busy");
        let fields = build_failed_event_fields(
            &task,
            ErrorKind::Network,
            "failed",
            123,
            Some(456),
            &error,
            "webdav request failed: HTTP 503: busy",
        );

        let obj = fields.as_object().expect("object");
        assert_eq!(
            obj.get("error_kind").and_then(|v| v.as_str()),
            Some("network")
        );
        assert_eq!(
            obj.get("error_envelope")
                .and_then(|v| v.get("transport"))
                .and_then(|v| v.get("protocol"))
                .and_then(|v| v.as_str()),
            Some("http")
        );
        assert_eq!(
            obj.get("error_envelope")
                .and_then(|v| v.get("transport"))
                .and_then(|v| v.get("provider"))
                .and_then(|v| v.as_str()),
            Some("webdav")
        );
        assert_eq!(
            obj.get("error_envelope")
                .and_then(|v| v.get("retriable"))
                .and_then(|v| v.get("value"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn failed_event_fields_use_internal_transport_for_agent_delete() {
        let task = dummy_task(artifact_delete_repo::DeleteTargetType::LocalDir, "agent-1");
        let error = anyhow::anyhow!("agent not connected");
        let fields = build_failed_event_fields(
            &task,
            ErrorKind::Network,
            "failed",
            12,
            Some(34),
            &error,
            "agent not connected",
        );

        let obj = fields.as_object().expect("object");
        assert_eq!(
            obj.get("error_envelope")
                .and_then(|v| v.get("transport"))
                .and_then(|v| v.get("protocol"))
                .and_then(|v| v.as_str()),
            Some("internal")
        );
        assert_eq!(
            obj.get("error_envelope")
                .and_then(|v| v.get("transport"))
                .and_then(|v| v.get("provider"))
                .and_then(|v| v.as_str()),
            Some("agent")
        );
    }
}
