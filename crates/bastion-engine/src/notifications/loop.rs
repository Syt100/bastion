use std::sync::Arc;

use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

use bastion_storage::notifications_repo;
use bastion_storage::secrets::SecretsCrypto;

use crate::error_envelope::{
    envelope, insert_error_envelope, origin, retriable, retriable_with_reason, transport,
    with_context_param,
};
use crate::run_events;
use crate::run_events_bus::RunEventsBus;
use crate::supervision::spawn_supervised;

use super::send::{SendOutcome, send_one};

const MAX_ATTEMPTS: i64 = 10;
const BACKOFF_BASE_SECONDS: i64 = 30;
const BACKOFF_MAX_SECONDS: i64 = 60 * 60;

fn classify_notification_error(error: &str) -> (&'static str, &'static str, &'static str) {
    let lower = error.to_lowercase();
    if lower.contains("http 401")
        || lower.contains("http 403")
        || lower.contains("unauthorized")
        || lower.contains("forbidden")
        || lower.contains("auth")
    {
        return (
            "auth",
            "diagnostics.hint.notification.auth",
            "diagnostics.message.notification.auth",
        );
    }

    if lower.contains("http 429")
        || lower.contains("too many requests")
        || lower.contains("rate limit")
    {
        return (
            "rate_limited",
            "diagnostics.hint.notification.rate_limited",
            "diagnostics.message.notification.rate_limited",
        );
    }

    if lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("connection")
        || lower.contains("dns")
        || lower.contains("network")
    {
        return (
            "network",
            "diagnostics.hint.notification.network",
            "diagnostics.message.notification.network",
        );
    }

    (
        "unknown",
        "diagnostics.hint.notification.unknown",
        "diagnostics.message.notification.unknown",
    )
}

fn notification_transport(channel: &str) -> bastion_core::error_envelope::ErrorTransportV1 {
    match channel {
        notifications_repo::CHANNEL_WECOM_BOT => transport("http").with_provider("wecom_bot"),
        notifications_repo::CHANNEL_EMAIL => transport("smtp").with_provider("email"),
        _ => transport("internal").with_provider(channel.to_string()),
    }
}

pub fn spawn(
    db: SqlitePool,
    secrets: Arc<SecretsCrypto>,
    run_events_bus: Arc<RunEventsBus>,
    notifications_notify: Arc<Notify>,
    shutdown: CancellationToken,
) {
    spawn_supervised(
        "notifications.loop",
        shutdown.clone(),
        run_loop(db, secrets, run_events_bus, notifications_notify, shutdown),
    );
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
                    Some({
                        let mut fields = serde_json::Map::new();
                        fields.insert(
                            "channel".to_string(),
                            serde_json::Value::String(notification.channel.clone()),
                        );
                        fields.insert(
                            "secret_name".to_string(),
                            serde_json::Value::String(notification.secret_name.clone()),
                        );
                        fields.insert("attempts".to_string(), serde_json::json!(attempts));
                        fields.insert("error".to_string(), serde_json::json!(error_str.clone()));
                        let (error_kind, hint_key, message_key) =
                            classify_notification_error(&error_str);
                        fields.insert(
                            "error_kind".to_string(),
                            serde_json::Value::String(error_kind.to_string()),
                        );
                        let mut env = envelope(
                            format!("notification.send.{error_kind}"),
                            error_kind,
                            if attempts < MAX_ATTEMPTS {
                                retriable_with_reason(true, error_kind)
                            } else {
                                retriable(false)
                            },
                            hint_key,
                            message_key,
                            notification_transport(&notification.channel),
                        )
                        .with_origin(origin("notification", "sender", "deliver"))
                        .with_stage("notify");
                        env = with_context_param(env, "attempts", attempts);
                        env = with_context_param(env, "channel", notification.channel.clone());
                        env = with_context_param(
                            env,
                            "secret_name",
                            notification.secret_name.clone(),
                        );
                        env = with_context_param(env, "error", error_str.clone());
                        insert_error_envelope(&mut fields, env);
                        serde_json::Value::Object(fields)
                    }),
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

#[cfg(test)]
mod tests {
    use super::{classify_notification_error, notification_transport};
    use bastion_storage::notifications_repo;

    #[test]
    fn classify_notification_error_uses_stable_kind_mapping() {
        assert_eq!(
            classify_notification_error("HTTP 401 unauthorized").0,
            "auth"
        );
        assert_eq!(
            classify_notification_error("too many requests").0,
            "rate_limited"
        );
        assert_eq!(classify_notification_error("dns timeout").0, "network");
        assert_eq!(classify_notification_error("unexpected panic").0, "unknown");
    }

    #[test]
    fn notification_transport_maps_channel_protocol() {
        let wecom = notification_transport(notifications_repo::CHANNEL_WECOM_BOT);
        assert_eq!(wecom.protocol, "http");
        assert_eq!(wecom.provider.as_deref(), Some("wecom_bot"));

        let email = notification_transport(notifications_repo::CHANNEL_EMAIL);
        assert_eq!(email.protocol, "smtp");
        assert_eq!(email.provider.as_deref(), Some("email"));

        let custom = notification_transport("custom");
        assert_eq!(custom.protocol, "internal");
        assert_eq!(custom.provider.as_deref(), Some("custom"));
    }
}
