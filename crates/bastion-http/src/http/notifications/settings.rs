use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use serde_json::json;
use time::OffsetDateTime;
use tower_cookies::Cookies;

use bastion_storage::notifications_repo;
use bastion_storage::notifications_settings_repo;

use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};

pub(in crate::http) async fn get_settings(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<notifications_settings_repo::NotificationsSettings>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let settings = notifications_settings_repo::get_or_default(&state.db).await?;
    Ok(Json(settings))
}

pub(in crate::http) async fn put_settings(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Json(req): Json<notifications_settings_repo::NotificationsSettings>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    if req.templates.wecom_markdown.trim().is_empty() {
        return Err(
            AppError::bad_request("invalid_template", "WeCom template is required")
                .with_details(json!({ "field": "templates.wecom_markdown" })),
        );
    }
    if req.templates.email_subject.trim().is_empty() {
        return Err(AppError::bad_request(
            "invalid_template",
            "Email subject template is required",
        )
        .with_details(json!({ "field": "templates.email_subject" })));
    }
    if req.templates.email_body.trim().is_empty() {
        return Err(
            AppError::bad_request("invalid_template", "Email body template is required")
                .with_details(json!({ "field": "templates.email_body" })),
        );
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
