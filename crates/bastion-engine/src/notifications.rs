use std::sync::Arc;

use sqlx::Row;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use bastion_core::HUB_NODE_ID;
use bastion_storage::notification_destinations_repo;
use bastion_storage::notifications_repo;
use bastion_storage::notifications_settings_repo;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;

use crate::run_events;
use crate::run_events_bus::RunEventsBus;
use bastion_notify::{smtp, wecom};

const MAX_ATTEMPTS: i64 = 10;
const BACKOFF_BASE_SECONDS: i64 = 30;
const BACKOFF_MAX_SECONDS: i64 = 60 * 60;

enum SendOutcome {
    Sent,
    Canceled { reason: String },
}

pub fn spawn(
    db: SqlitePool,
    secrets: Arc<SecretsCrypto>,
    run_events_bus: Arc<RunEventsBus>,
    notifications_notify: Arc<Notify>,
    shutdown: CancellationToken,
) {
    tokio::spawn(run_loop(
        db,
        secrets,
        run_events_bus,
        notifications_notify,
        shutdown,
    ));
}

async fn run_loop(
    db: SqlitePool,
    secrets: Arc<SecretsCrypto>,
    run_events_bus: Arc<RunEventsBus>,
    notifications_notify: Arc<Notify>,
    shutdown: CancellationToken,
) {
    loop {
        if shutdown.is_cancelled() {
            break;
        }

        let now = OffsetDateTime::now_utc().unix_timestamp();
        let next = match notifications_repo::claim_next_due(&db, now).await {
            Ok(v) => v,
            Err(error) => {
                warn!(error = %error, "failed to claim due notification");
                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    _ = notifications_notify.notified() => {}
                    _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
                }
                continue;
            }
        };

        let Some(notification) = next else {
            let next_due_at = match notifications_repo::next_due_at(&db).await {
                Ok(v) => v,
                Err(error) => {
                    warn!(error = %error, "failed to fetch next due notification time");
                    tokio::select! {
                        _ = shutdown.cancelled() => break,
                        _ = notifications_notify.notified() => {}
                        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
                    }
                    continue;
                }
            };

            let Some(next_due_at) = next_due_at else {
                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    _ = notifications_notify.notified() => {}
                }
                continue;
            };

            let delay = next_due_at.saturating_sub(now);
            tokio::select! {
                _ = shutdown.cancelled() => break,
                _ = notifications_notify.notified() => {}
                _ = tokio::time::sleep(std::time::Duration::from_secs(delay as u64)) => {}
            }
            continue;
        };

        match send_one(&db, &secrets, &run_events_bus, &notification).await {
            Ok(SendOutcome::Sent) => {
                if let Err(error) = notifications_repo::mark_sent(&db, &notification.id, now).await
                {
                    warn!(error = %error, id = %notification.id, "failed to mark notification sent");
                }
            }
            Ok(SendOutcome::Canceled { reason }) => {
                debug!(id = %notification.id, reason = %reason, "notification canceled");
                if let Err(error) =
                    notifications_repo::mark_canceled(&db, &notification.id, &reason, now).await
                {
                    warn!(error = %error, id = %notification.id, "failed to mark notification canceled");
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

                let _ = run_events::append_and_broadcast(
                    &db,
                    &run_events_bus,
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
    let shift = (attempts.saturating_sub(1)).clamp(0, 20) as u32;
    let exp = 1_i64.checked_shl(shift).unwrap_or(i64::MAX);
    let delay = BACKOFF_BASE_SECONDS.saturating_mul(exp);
    delay.clamp(BACKOFF_BASE_SECONDS, BACKOFF_MAX_SECONDS)
}

async fn send_one(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    run_events_bus: &RunEventsBus,
    notification: &notifications_repo::NotificationRow,
) -> Result<SendOutcome, anyhow::Error> {
    let settings = notifications_settings_repo::get_or_default(db).await?;
    if !settings.enabled {
        return Ok(SendOutcome::Canceled {
            reason: "canceled: notifications disabled".to_string(),
        });
    }

    match notification.channel.as_str() {
        notifications_repo::CHANNEL_WECOM_BOT => {
            if !settings.channels.wecom_bot.enabled {
                return Ok(SendOutcome::Canceled {
                    reason: "canceled: channel disabled".to_string(),
                });
            }
            if !notification_destinations_repo::is_enabled(
                db,
                notifications_repo::CHANNEL_WECOM_BOT,
                &notification.secret_name,
            )
            .await?
            {
                return Ok(SendOutcome::Canceled {
                    reason: "canceled: destination disabled".to_string(),
                });
            }

            let secret_bytes = secrets_repo::get_secret(
                db,
                secrets,
                HUB_NODE_ID,
                "wecom_bot",
                &notification.secret_name,
            )
            .await?;
            let Some(secret_bytes) = secret_bytes else {
                return Ok(SendOutcome::Canceled {
                    reason: "canceled: destination deleted".to_string(),
                });
            };
            #[derive(serde::Deserialize)]
            struct Payload {
                webhook_url: String,
            }
            let payload: Payload = serde_json::from_slice(&secret_bytes)?;

            let ctx = build_context(db, &notification.run_id).await?;
            let content = render_template(&settings.templates.wecom_markdown, &ctx);
            wecom::send_markdown(&payload.webhook_url, &content).await?;

            let _ = run_events::append_and_broadcast(
                db,
                run_events_bus,
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

            Ok(SendOutcome::Sent)
        }
        notifications_repo::CHANNEL_EMAIL => {
            if !settings.channels.email.enabled {
                return Ok(SendOutcome::Canceled {
                    reason: "canceled: channel disabled".to_string(),
                });
            }
            if !notification_destinations_repo::is_enabled(
                db,
                notifications_repo::CHANNEL_EMAIL,
                &notification.secret_name,
            )
            .await?
            {
                return Ok(SendOutcome::Canceled {
                    reason: "canceled: destination disabled".to_string(),
                });
            }

            let secret_bytes = secrets_repo::get_secret(
                db,
                secrets,
                HUB_NODE_ID,
                "smtp",
                &notification.secret_name,
            )
            .await?;
            let Some(secret_bytes) = secret_bytes else {
                return Ok(SendOutcome::Canceled {
                    reason: "canceled: destination deleted".to_string(),
                });
            };
            let payload: smtp::SmtpSecretPayload = serde_json::from_slice(&secret_bytes)?;

            let ctx = build_context(db, &notification.run_id).await?;
            let subject = render_template(&settings.templates.email_subject, &ctx);
            let body = render_template(&settings.templates.email_body, &ctx);
            smtp::send_plain_text(&payload, &subject, &body).await?;

            let _ = run_events::append_and_broadcast(
                db,
                run_events_bus,
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

            Ok(SendOutcome::Sent)
        }
        other => anyhow::bail!("unsupported notification channel: {other}"),
    }
}

struct TemplateContext {
    title: String,
    job_id: String,
    job_name: String,
    run_id: String,
    status: String,
    status_text: String,
    started_at: String,
    ended_at: String,
    target_type: String,
    target_location: String,
    target: String,
    error: String,
    target_line_wecom: String,
    error_line_wecom: String,
    target_line_email: String,
    error_line_email: String,
}

async fn build_context(db: &SqlitePool, run_id: &str) -> Result<TemplateContext, anyhow::Error> {
    let row = sqlx::query(
        "SELECT job_id, status, started_at, ended_at, error, summary_json FROM runs WHERE id = ? LIMIT 1",
    )
    .bind(run_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(TemplateContext {
            title: "Bastion 备份完成".to_string(),
            job_id: "-".to_string(),
            job_name: "-".to_string(),
            run_id: run_id.to_string(),
            status: "unknown".to_string(),
            status_text: "未知".to_string(),
            started_at: "-".to_string(),
            ended_at: "-".to_string(),
            target_type: "-".to_string(),
            target_location: "-".to_string(),
            target: "-".to_string(),
            error: String::new(),
            target_line_wecom: String::new(),
            error_line_wecom: String::new(),
            target_line_email: String::new(),
            error_line_email: String::new(),
        });
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

    let (title, status_text) = match status.as_str() {
        "success" => ("Bastion 备份成功".to_string(), "备份成功".to_string()),
        "failed" => ("Bastion 备份失败".to_string(), "备份失败".to_string()),
        "rejected" => ("Bastion 备份被拒绝".to_string(), "备份被拒绝".to_string()),
        other => (format!("Bastion 备份完成 ({other})"), other.to_string()),
    };

    let started_at_str = format_ts(started_at);
    let ended_at_str = ended_at.map(format_ts).unwrap_or_else(|| "-".to_string());

    let mut target_type = "-".to_string();
    let mut target_location = "-".to_string();
    if let Some(summary) = summary_json
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(&summary)
        && let Some(target) = v.get("target")
    {
        target_type = target
            .get("type")
            .and_then(|x| x.as_str())
            .unwrap_or("-")
            .to_string();
        target_location = target
            .get("run_url")
            .or_else(|| target.get("run_dir"))
            .and_then(|x| x.as_str())
            .unwrap_or("-")
            .to_string();
    }

    let target = if target_type != "-" && target_location != "-" {
        format!("{target_type} {target_location}")
    } else if target_type != "-" {
        target_type.clone()
    } else if target_location != "-" {
        target_location.clone()
    } else {
        "-".to_string()
    };

    let error = error.unwrap_or_default();
    let error = error.trim().to_string();

    let target_line_wecom = if target != "-" {
        format!("> Target: {target}\n")
    } else {
        String::new()
    };
    let error_line_wecom = if !error.is_empty() {
        format!("> Error: {error}\n")
    } else {
        String::new()
    };

    let target_line_email = if target != "-" {
        format!("Target: {target}\n")
    } else {
        String::new()
    };
    let error_line_email = if !error.is_empty() {
        format!("Error: {error}\n")
    } else {
        String::new()
    };

    Ok(TemplateContext {
        title,
        job_id,
        job_name,
        run_id: run_id.to_string(),
        status,
        status_text,
        started_at: started_at_str,
        ended_at: ended_at_str,
        target_type,
        target_location,
        target,
        error,
        target_line_wecom,
        error_line_wecom,
        target_line_email,
        error_line_email,
    })
}

fn render_template(template: &str, ctx: &TemplateContext) -> String {
    let pairs = [
        ("{{title}}", ctx.title.as_str()),
        ("{{job_id}}", ctx.job_id.as_str()),
        ("{{job_name}}", ctx.job_name.as_str()),
        ("{{run_id}}", ctx.run_id.as_str()),
        ("{{status}}", ctx.status.as_str()),
        ("{{status_text}}", ctx.status_text.as_str()),
        ("{{started_at}}", ctx.started_at.as_str()),
        ("{{ended_at}}", ctx.ended_at.as_str()),
        ("{{target_type}}", ctx.target_type.as_str()),
        ("{{target_location}}", ctx.target_location.as_str()),
        ("{{target}}", ctx.target.as_str()),
        ("{{error}}", ctx.error.as_str()),
        ("{{target_line_wecom}}", ctx.target_line_wecom.as_str()),
        ("{{error_line_wecom}}", ctx.error_line_wecom.as_str()),
        ("{{target_line_email}}", ctx.target_line_email.as_str()),
        ("{{error_line_email}}", ctx.error_line_email.as_str()),
    ];

    let mut out = template.to_string();
    for (k, v) in pairs {
        out = out.replace(k, v);
    }
    out
}

fn format_ts(ts: i64) -> String {
    OffsetDateTime::from_unix_timestamp(ts)
        .ok()
        .and_then(|t| t.format(&Rfc3339).ok())
        .unwrap_or_else(|| ts.to_string())
}
