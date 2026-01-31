use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::OffsetDateTime;
use tower_cookies::Cookies;

use bastion_core::HUB_NODE_ID;
use bastion_notify::{smtp, wecom};
use bastion_storage::notification_destinations_repo;
use bastion_storage::notifications_repo;
use bastion_storage::secrets_repo;

use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};
use super::validation::{destination_exists, require_supported_channel};

pub(in crate::http) async fn list_destinations(
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
pub(in crate::http) struct DestinationListItem {
    channel: String,
    name: String,
    enabled: bool,
    updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct SetDestinationEnabledRequest {
    enabled: bool,
}

pub(in crate::http) async fn set_destination_enabled(
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

pub(in crate::http) async fn test_destination(
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
            let secret = secrets_repo::get_secret(
                &state.db,
                &state.secrets,
                HUB_NODE_ID,
                "wecom_bot",
                name.trim(),
            )
            .await?
            .ok_or_else(|| AppError::not_found("destination_not_found", "Destination not found"))?;
            #[derive(Deserialize)]
            struct Payload {
                webhook_url: String,
            }
            let payload: Payload = serde_json::from_slice(&secret)?;
            let content = format!(
                "**Bastion test notification**\n> Destination: {}\n> Time: {}\n",
                name.trim(),
                ts
            );
            wecom::send_markdown(&payload.webhook_url, &content).await?;
        }
        notifications_repo::CHANNEL_EMAIL => {
            let secret = secrets_repo::get_secret(
                &state.db,
                &state.secrets,
                HUB_NODE_ID,
                "smtp",
                name.trim(),
            )
            .await?
            .ok_or_else(|| AppError::not_found("destination_not_found", "Destination not found"))?;
            let payload: smtp::SmtpSecretPayload = serde_json::from_slice(&secret)?;
            let subject = "Bastion test notification".to_string();
            let body = format!(
                "Bastion test notification\n\nDestination: {}\nTime: {}\n",
                name.trim(),
                ts
            );
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
