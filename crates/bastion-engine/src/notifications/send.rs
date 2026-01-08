use sqlx::SqlitePool;
use tracing::info;

use bastion_core::HUB_NODE_ID;
use bastion_storage::notification_destinations_repo;
use bastion_storage::notifications_repo;
use bastion_storage::notifications_settings_repo;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;

use crate::run_events;
use crate::run_events_bus::RunEventsBus;
use bastion_notify::{smtp, wecom};

use super::template::{build_context, render_template};

pub(super) enum SendOutcome {
    Sent,
    Canceled { reason: String },
}

pub(super) async fn send_one(
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
