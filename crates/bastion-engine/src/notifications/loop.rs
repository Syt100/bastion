use std::sync::Arc;

use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

use bastion_storage::notifications_repo;
use bastion_storage::secrets::SecretsCrypto;

use crate::run_events;
use crate::run_events_bus::RunEventsBus;
use crate::supervision::spawn_supervised;

use super::send::{SendOutcome, send_one};

const MAX_ATTEMPTS: i64 = 10;
const BACKOFF_BASE_SECONDS: i64 = 30;
const BACKOFF_MAX_SECONDS: i64 = 60 * 60;

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
