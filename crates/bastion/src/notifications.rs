use std::sync::Arc;

use sqlx::Row;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tracing::{info, warn};

use crate::notifications_repo;
use crate::runs_repo;
use crate::secrets::SecretsCrypto;
use crate::secrets_repo;
use crate::smtp;
use crate::wecom;

const MAX_ATTEMPTS: i64 = 10;
const BACKOFF_BASE_SECONDS: i64 = 30;
const BACKOFF_MAX_SECONDS: i64 = 60 * 60;

pub fn spawn(db: SqlitePool, secrets: Arc<SecretsCrypto>) {
    tokio::spawn(run_loop(db, secrets));
}

async fn run_loop(db: SqlitePool, secrets: Arc<SecretsCrypto>) {
    loop {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let next = match notifications_repo::claim_next_due(&db, now).await {
            Ok(v) => v,
            Err(error) => {
                warn!(error = %error, "failed to claim due notification");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        let Some(notification) = next else {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        };

        match send_one(&db, &secrets, &notification).await {
            Ok(()) => {
                if let Err(error) = notifications_repo::mark_sent(&db, &notification.id, now).await
                {
                    warn!(error = %error, id = %notification.id, "failed to mark notification sent");
                }
            }
            Err(error) => {
                let attempts = notification.attempts.saturating_add(1);
                let error_str = format!("{error:#}");
                if attempts >= MAX_ATTEMPTS {
                    warn!(id = %notification.id, attempts, error = %error_str, "notification failed permanently");
                    let _ = notifications_repo::mark_failed(
                        &db,
                        &notification.id,
                        attempts,
                        &error_str,
                        now,
                    )
                    .await;
                } else {
                    let delay = backoff_seconds(attempts);
                    let next_attempt_at = now.saturating_add(delay);
                    warn!(id = %notification.id, attempts, delay, error = %error_str, "notification failed; will retry");
                    let _ = notifications_repo::reschedule(
                        &db,
                        &notification.id,
                        attempts,
                        next_attempt_at,
                        &error_str,
                        now,
                    )
                    .await;
                }

                let _ = runs_repo::append_run_event(
                    &db,
                    &notification.run_id,
                    "warn",
                    "notify_failed",
                    "notify_failed",
                    Some(serde_json::json!({
                        "channel": notification.channel,
                        "secret_name": notification.secret_name,
                        "attempts": attempts,
                        "error": error_str,
                    })),
                )
                .await;
            }
        }
    }
}

fn backoff_seconds(attempts: i64) -> i64 {
    let shift = (attempts.saturating_sub(1)).min(20).max(0) as u32;
    let exp = 1_i64.checked_shl(shift).unwrap_or(i64::MAX);
    let delay = BACKOFF_BASE_SECONDS.saturating_mul(exp);
    delay.clamp(BACKOFF_BASE_SECONDS, BACKOFF_MAX_SECONDS)
}

async fn send_one(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    notification: &notifications_repo::NotificationRow,
) -> Result<(), anyhow::Error> {
    match notification.channel.as_str() {
        notifications_repo::CHANNEL_WECOM_BOT => {
            let secret_bytes =
                secrets_repo::get_secret(db, secrets, "wecom_bot", &notification.secret_name)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("missing wecom_bot secret: {}", notification.secret_name)
                    })?;
            #[derive(serde::Deserialize)]
            struct Payload {
                webhook_url: String,
            }
            let payload: Payload = serde_json::from_slice(&secret_bytes)?;

            let content = build_wecom_markdown(db, &notification.run_id).await?;
            wecom::send_markdown(&payload.webhook_url, &content).await?;

            let _ = runs_repo::append_run_event(
                db,
                &notification.run_id,
                "info",
                "notify_sent",
                "notify_sent",
                Some(serde_json::json!({
                    "channel": notification.channel,
                    "secret_name": notification.secret_name,
                })),
            )
            .await;

            info!(
                run_id = %notification.run_id,
                secret_name = %notification.secret_name,
                "wecom notification sent"
            );

            Ok(())
        }
        notifications_repo::CHANNEL_EMAIL => {
            let secret_bytes =
                secrets_repo::get_secret(db, secrets, "smtp", &notification.secret_name)
                    .await?
                    .ok_or_else(|| {
                        anyhow::anyhow!("missing smtp secret: {}", notification.secret_name)
                    })?;
            let payload: smtp::SmtpSecretPayload = serde_json::from_slice(&secret_bytes)?;

            let (subject, body) = build_email_text(db, &notification.run_id).await?;
            smtp::send_plain_text(&payload, &subject, &body).await?;

            let _ = runs_repo::append_run_event(
                db,
                &notification.run_id,
                "info",
                "notify_sent",
                "notify_sent",
                Some(serde_json::json!({
                    "channel": notification.channel,
                    "secret_name": notification.secret_name,
                })),
            )
            .await;

            info!(
                run_id = %notification.run_id,
                secret_name = %notification.secret_name,
                "email notification sent"
            );

            Ok(())
        }
        other => anyhow::bail!("unsupported notification channel: {other}"),
    }
}

async fn build_wecom_markdown(db: &SqlitePool, run_id: &str) -> Result<String, anyhow::Error> {
    let row = sqlx::query(
        "SELECT job_id, status, started_at, ended_at, error, summary_json FROM runs WHERE id = ? LIMIT 1",
    )
    .bind(run_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(format!("**Bastion**\n> Run: {run_id}\n> Status: unknown\n"));
    };

    let job_id = row.get::<String, _>("job_id");
    let status = row.get::<String, _>("status");
    let started_at = row.get::<i64, _>("started_at");
    let ended_at = row.get::<Option<i64>, _>("ended_at");
    let error = row.get::<Option<String>, _>("error");
    let summary_json = row.get::<Option<String>, _>("summary_json");

    let job_name = sqlx::query_scalar::<_, String>("SELECT name FROM jobs WHERE id = ? LIMIT 1")
        .bind(&job_id)
        .fetch_optional(db)
        .await?
        .unwrap_or_else(|| job_id.clone());

    let title = match status.as_str() {
        "success" => "Bastion 备份成功",
        "failed" => "Bastion 备份失败",
        "rejected" => "Bastion 备份被拒绝",
        other => return Ok(format!("**Bastion**\n> Run: {run_id}\n> Status: {other}\n")),
    };

    let started = format_ts(started_at);
    let ended = ended_at.map(format_ts).unwrap_or_else(|| "-".to_string());

    let mut target_line = String::new();
    if let Some(summary) = summary_json {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&summary) {
            let target = v.get("target");
            if let Some(target) = target {
                let ttype = target.get("type").and_then(|x| x.as_str()).unwrap_or("-");
                let loc = target
                    .get("run_url")
                    .or_else(|| target.get("run_dir"))
                    .and_then(|x| x.as_str())
                    .unwrap_or("-");
                target_line = format!("{ttype} {loc}");
            }
        }
    }

    let mut content = String::new();
    content.push_str(&format!("**{title}**\n"));
    content.push_str(&format!("> Job: {job_name}\n"));
    content.push_str(&format!("> Run: {run_id}\n"));
    content.push_str(&format!("> Started: {started}\n"));
    content.push_str(&format!("> Ended: {ended}\n"));
    if !target_line.is_empty() {
        content.push_str(&format!("> Target: {target_line}\n"));
    }
    if let Some(error) = error {
        if !error.trim().is_empty() {
            content.push_str(&format!("> Error: {error}\n"));
        }
    }

    Ok(content)
}

fn format_ts(ts: i64) -> String {
    OffsetDateTime::from_unix_timestamp(ts)
        .ok()
        .and_then(|t| t.format(&Rfc3339).ok())
        .unwrap_or_else(|| ts.to_string())
}

async fn build_email_text(
    db: &SqlitePool,
    run_id: &str,
) -> Result<(String, String), anyhow::Error> {
    let row = sqlx::query(
        "SELECT job_id, status, started_at, ended_at, error, summary_json FROM runs WHERE id = ? LIMIT 1",
    )
    .bind(run_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok((
            "Bastion 备份完成".to_string(),
            format!("Bastion backup\nRun: {run_id}\nStatus: unknown\n"),
        ));
    };

    let job_id = row.get::<String, _>("job_id");
    let status = row.get::<String, _>("status");
    let started_at = row.get::<i64, _>("started_at");
    let ended_at = row.get::<Option<i64>, _>("ended_at");
    let error = row.get::<Option<String>, _>("error");
    let summary_json = row.get::<Option<String>, _>("summary_json");

    let job_name = sqlx::query_scalar::<_, String>("SELECT name FROM jobs WHERE id = ? LIMIT 1")
        .bind(&job_id)
        .fetch_optional(db)
        .await?
        .unwrap_or_else(|| job_id.clone());

    let subject = match status.as_str() {
        "success" => format!("Bastion 备份成功 - {job_name}"),
        "failed" => format!("Bastion 备份失败 - {job_name}"),
        "rejected" => format!("Bastion 备份被拒绝 - {job_name}"),
        _ => format!("Bastion 备份完成 - {job_name}"),
    };

    let started = format_ts(started_at);
    let ended = ended_at.map(format_ts).unwrap_or_else(|| "-".to_string());

    let mut target_line = String::new();
    if let Some(summary) = summary_json {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&summary) {
            let target = v.get("target");
            if let Some(target) = target {
                let ttype = target.get("type").and_then(|x| x.as_str()).unwrap_or("-");
                let loc = target
                    .get("run_url")
                    .or_else(|| target.get("run_dir"))
                    .and_then(|x| x.as_str())
                    .unwrap_or("-");
                target_line = format!("{ttype} {loc}");
            }
        }
    }

    let mut body = String::new();
    body.push_str("Bastion backup\n\n");
    body.push_str(&format!("Job: {job_name}\n"));
    body.push_str(&format!("Run: {run_id}\n"));
    body.push_str(&format!("Status: {status}\n"));
    body.push_str(&format!("Started: {started}\n"));
    body.push_str(&format!("Ended: {ended}\n"));
    if !target_line.is_empty() {
        body.push_str(&format!("Target: {target_line}\n"));
    }
    if let Some(error) = error {
        if !error.trim().is_empty() {
            body.push_str(&format!("Error: {error}\n"));
        }
    }

    Ok((subject, body))
}
