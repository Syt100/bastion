use axum::Json;
use axum::extract::{Path, RawQuery};
use axum::http::{HeaderMap, StatusCode};
use serde::Serialize;
use time::OffsetDateTime;
use tower_cookies::Cookies;

use bastion_storage::notification_destinations_repo;
use bastion_storage::notifications_repo;
use bastion_storage::notifications_settings_repo;

use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};
use super::validation::{destination_exists, require_supported_channel};

#[derive(Debug, Serialize)]
pub(in crate::http) struct ListQueueResponse {
    items: Vec<QueueItem>,
    page: i64,
    page_size: i64,
    total: i64,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct QueueItem {
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

pub(in crate::http) async fn list_queue(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    RawQuery(raw): RawQuery,
) -> Result<Json<ListQueueResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    fn normalize_filter_list(values: Vec<String>) -> Vec<String> {
        let mut out: Vec<String> = values
            .into_iter()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect();
        out.sort();
        out.dedup();
        out
    }

    let mut statuses = Vec::new();
    let mut channels = Vec::new();
    let mut page: Option<i64> = None;
    let mut page_size: Option<i64> = None;

    if let Some(raw) = raw {
        for (key, value) in url::form_urlencoded::parse(raw.as_bytes()) {
            match key.as_ref() {
                "status" | "status[]" => statuses.push(value.into_owned()),
                "channel" | "channel[]" => channels.push(value.into_owned()),
                "page" => {
                    let value = value
                        .parse::<i64>()
                        .map_err(|_| AppError::bad_request("invalid_page", "Invalid page"))?;
                    page = Some(value);
                }
                "page_size" => {
                    let value = value.parse::<i64>().map_err(|_| {
                        AppError::bad_request("invalid_page_size", "Invalid page_size")
                    })?;
                    page_size = Some(value);
                }
                _ => {}
            }
        }
    }

    let statuses = normalize_filter_list(statuses);
    let channels = normalize_filter_list(channels);
    for channel in &channels {
        require_supported_channel(channel)?;
    }

    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1).saturating_mul(page_size);

    let status_filter = if statuses.is_empty() {
        None
    } else {
        Some(statuses.as_slice())
    };
    let channel_filter = if channels.is_empty() {
        None
    } else {
        Some(channels.as_slice())
    };

    let total = notifications_repo::count_queue(&state.db, status_filter, channel_filter).await?;
    let rows =
        notifications_repo::list_queue(&state.db, status_filter, channel_filter, page_size, offset)
            .await?;

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

pub(in crate::http) async fn cancel(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let Some(_row) = notifications_repo::get_notification(&state.db, &id).await? else {
        return Err(AppError::not_found(
            "notification_not_found",
            "Notification not found",
        ));
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

pub(in crate::http) async fn retry_now(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let Some(row) = notifications_repo::get_notification(&state.db, &id).await? else {
        return Err(AppError::not_found(
            "notification_not_found",
            "Notification not found",
        ));
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
                return Err(AppError::conflict(
                    "channel_disabled",
                    "Channel is disabled",
                ));
            }
        }
        notifications_repo::CHANNEL_EMAIL => {
            if !settings.channels.email.enabled {
                return Err(AppError::conflict(
                    "channel_disabled",
                    "Channel is disabled",
                ));
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
    if !notification_destinations_repo::is_enabled(&state.db, &row.channel, &row.secret_name)
        .await?
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
