use axum::Json;
use axum::extract::{Path, Query};
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tower_cookies::Cookies;

use super::shared::{require_csrf, require_session};
use super::{AppError, AppState};
use bastion_notify::{smtp, wecom};
use bastion_storage::notification_destinations_repo;
use bastion_storage::notifications_repo;
use bastion_storage::notifications_settings_repo;
use bastion_storage::secrets_repo;

fn require_supported_channel(channel: &str) -> Result<(), AppError> {
    if channel == notifications_repo::CHANNEL_WECOM_BOT || channel == notifications_repo::CHANNEL_EMAIL {
        return Ok(());
    }
    Err(AppError::bad_request(
        "invalid_channel",
        "Unsupported notification channel",
    )
    .with_details(json!({ "field": "channel" })))
}

async fn destination_exists(db: &SqlitePool, channel: &str, name: &str) -> Result<bool, anyhow::Error> {
    let Some(kind) = notification_destinations_repo::secret_kind_for_channel(channel) else {
        return Ok(false);
    };
    let row = sqlx::query("SELECT 1 FROM secrets WHERE kind = ? AND name = ? LIMIT 1")
        .bind(kind)
        .bind(name)
        .fetch_optional(db)
        .await?;
    Ok(row.is_some())
}

pub(super) async fn get_settings(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<notifications_settings_repo::NotificationsSettings>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let settings = notifications_settings_repo::get_or_default(&state.db).await?;
    Ok(Json(settings))
}

pub(super) async fn put_settings(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Json(req): Json<notifications_settings_repo::NotificationsSettings>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    if req.templates.wecom_markdown.trim().is_empty() {
        return Err(AppError::bad_request(
            "invalid_template",
            "WeCom template is required",
        )
        .with_details(json!({ "field": "templates.wecom_markdown" })));
    }
    if req.templates.email_subject.trim().is_empty() {
        return Err(AppError::bad_request(
            "invalid_template",
            "Email subject template is required",
        )
        .with_details(json!({ "field": "templates.email_subject" })));
    }
    if req.templates.email_body.trim().is_empty() {
        return Err(AppError::bad_request(
            "invalid_template",
            "Email body template is required",
        )
        .with_details(json!({ "field": "templates.email_body" })));
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    if !req.enabled {
        let _ = notifications_repo::cancel_all_queued(
            &state.db,
            "canceled: notifications disabled",
            now,
        )
        .await?;
    } else {
        if !req.channels.wecom_bot.enabled {
            let _ = notifications_repo::cancel_queued_for_channel(
                &state.db,
                notifications_repo::CHANNEL_WECOM_BOT,
                "canceled: channel disabled",
                now,
            )
            .await?;
        }
        if !req.channels.email.enabled {
            let _ = notifications_repo::cancel_queued_for_channel(
                &state.db,
                notifications_repo::CHANNEL_EMAIL,
                "canceled: channel disabled",
                now,
            )
            .await?;
        }
    }

    notifications_settings_repo::upsert(&state.db, &req).await?;
    tracing::info!(
        enabled = req.enabled,
        wecom_enabled = req.channels.wecom_bot.enabled,
        email_enabled = req.channels.email.enabled,
        "notification settings updated"
    );
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn list_destinations(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<DestinationListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let items = notification_destinations_repo::list_destinations(&state.db).await?;
    Ok(Json(
        items
            .into_iter()
            .map(|d| DestinationListItem {
                channel: d.channel,
                name: d.name,
                enabled: d.enabled,
                updated_at: d.updated_at,
            })
            .collect(),
    ))
}

#[derive(Debug, Serialize)]
pub(super) struct DestinationListItem {
    channel: String,
    name: String,
    enabled: bool,
    updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub(super) struct SetDestinationEnabledRequest {
    enabled: bool,
}

pub(super) async fn set_destination_enabled(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path((channel, name)): Path<(String, String)>,
    Json(req): Json<SetDestinationEnabledRequest>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    require_supported_channel(&channel)?;
    if name.trim().is_empty() {
        return Err(
            AppError::bad_request("invalid_name", "Destination name is required")
                .with_details(json!({ "field": "name" })),
        );
    }

    if !destination_exists(&state.db, &channel, name.trim()).await? {
        return Err(AppError::not_found(
            "destination_not_found",
            "Destination not found",
        ));
    }

    notification_destinations_repo::set_enabled(&state.db, &channel, name.trim(), req.enabled)
        .await?;

    let now = OffsetDateTime::now_utc().unix_timestamp();
    if !req.enabled {
        let _ = notifications_repo::cancel_queued_for_destination(
            &state.db,
            &channel,
            name.trim(),
            "canceled: destination disabled",
            now,
        )
        .await?;
    }

    tracing::info!(
        channel = %channel,
        destination = %name.trim(),
        enabled = req.enabled,
        "notification destination updated"
    );

    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn test_destination(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path((channel, name)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    require_supported_channel(&channel)?;
    if name.trim().is_empty() {
        return Err(
            AppError::bad_request("invalid_name", "Destination name is required")
                .with_details(json!({ "field": "name" })),
        );
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let ts = OffsetDateTime::from_unix_timestamp(now)
        .ok()
        .map(|t| t.to_string())
        .unwrap_or_else(|| now.to_string());

    match channel.as_str() {
        notifications_repo::CHANNEL_WECOM_BOT => {
            let secret = secrets_repo::get_secret(&state.db, &state.secrets, "wecom_bot", name.trim())
                .await?
                .ok_or_else(|| AppError::not_found("destination_not_found", "Destination not found"))?;
            #[derive(Deserialize)]
            struct Payload {
                webhook_url: String,
            }
            let payload: Payload = serde_json::from_slice(&secret)?;
            let content = format!("**Bastion 测试通知**\n> Destination: {}\n> Time: {}\n", name.trim(), ts);
            wecom::send_markdown(&payload.webhook_url, &content).await?;
        }
        notifications_repo::CHANNEL_EMAIL => {
            let secret = secrets_repo::get_secret(&state.db, &state.secrets, "smtp", name.trim())
                .await?
                .ok_or_else(|| AppError::not_found("destination_not_found", "Destination not found"))?;
            let payload: smtp::SmtpSecretPayload = serde_json::from_slice(&secret)?;
            let subject = "Bastion 测试通知".to_string();
            let body = format!("Bastion test notification\n\nDestination: {}\nTime: {}\n", name.trim(), ts);
            smtp::send_plain_text(&payload, &subject, &body).await?;
        }
        _ => {
            return Err(AppError::bad_request(
                "invalid_channel",
                "Unsupported notification channel",
            ));
        }
    }

    tracing::info!(channel = %channel, destination = %name.trim(), "test notification sent");
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub(super) struct ListQueueQuery {
    status: Option<String>,
    channel: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub(super) struct ListQueueResponse {
    items: Vec<QueueItem>,
    page: i64,
    page_size: i64,
    total: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct QueueItem {
    id: String,
    run_id: String,
    job_id: String,
    job_name: String,
    channel: String,
    destination: String,
    status: String,
    attempts: i64,
    next_attempt_at: i64,
    created_at: i64,
    updated_at: i64,
    last_error: Option<String>,
    destination_deleted: bool,
    destination_enabled: bool,
}

pub(super) async fn list_queue(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Query(q): Query<ListQueueQuery>,
) -> Result<Json<ListQueueResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    if let Some(channel) = q.channel.as_deref() {
        require_supported_channel(channel)?;
    }

    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1).saturating_mul(page_size);

    let status = q.status.as_deref();
    let channel = q.channel.as_deref();

    let total = notifications_repo::count_queue(&state.db, status, channel).await?;
    let rows = notifications_repo::list_queue(&state.db, status, channel, page_size, offset).await?;

    let items = rows
        .into_iter()
        .map(|r| QueueItem {
            id: r.id,
            run_id: r.run_id,
            job_id: r.job_id,
            job_name: r.job_name,
            channel: r.channel,
            destination: r.secret_name,
            status: r.status,
            attempts: r.attempts,
            next_attempt_at: r.next_attempt_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
            last_error: r.last_error,
            destination_deleted: r.destination_deleted,
            destination_enabled: r.destination_enabled,
        })
        .collect();

    Ok(Json(ListQueueResponse {
        items,
        page,
        page_size,
        total,
    }))
}

pub(super) async fn cancel(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let Some(_row) = notifications_repo::get_notification(&state.db, &id).await? else {
        return Err(AppError::not_found("notification_not_found", "Notification not found"));
    };

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let ok = notifications_repo::cancel_queued_by_id(&state.db, &id, "canceled: user", now).await?;
    if !ok {
        return Err(AppError::conflict(
            "not_cancelable",
            "Notification is not cancelable",
        ));
    }

    tracing::info!(notification_id = %id, "notification canceled");
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn retry_now(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let Some(row) = notifications_repo::get_notification(&state.db, &id).await? else {
        return Err(AppError::not_found("notification_not_found", "Notification not found"));
    };

    let settings = notifications_settings_repo::get_or_default(&state.db).await?;
    if !settings.enabled {
        return Err(AppError::conflict(
            "notifications_disabled",
            "Notifications are disabled",
        ));
    }
    match row.channel.as_str() {
        notifications_repo::CHANNEL_WECOM_BOT => {
            if !settings.channels.wecom_bot.enabled {
                return Err(AppError::conflict("channel_disabled", "Channel is disabled"));
            }
        }
        notifications_repo::CHANNEL_EMAIL => {
            if !settings.channels.email.enabled {
                return Err(AppError::conflict("channel_disabled", "Channel is disabled"));
            }
        }
        _ => {
            return Err(AppError::bad_request(
                "invalid_channel",
                "Unsupported notification channel",
            ));
        }
    }

    // Ensure destination exists and is enabled (retry-now should not bypass disabled destinations).
    if !destination_exists(&state.db, &row.channel, &row.secret_name).await? {
        return Err(AppError::conflict(
            "destination_deleted",
            "Destination has been deleted",
        ));
    }
    if !notification_destinations_repo::is_enabled(&state.db, &row.channel, &row.secret_name).await?
    {
        return Err(AppError::conflict(
            "destination_disabled",
            "Destination is disabled",
        ));
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let ok = notifications_repo::retry_now_by_id(&state.db, &id, now).await?;
    if !ok {
        return Err(AppError::conflict(
            "not_retryable",
            "Notification is not retryable",
        ));
    }

    state.notifications_notify.notify_one();
    tracing::info!(notification_id = %id, "notification retry-now scheduled");
    Ok(StatusCode::NO_CONTENT)
}
