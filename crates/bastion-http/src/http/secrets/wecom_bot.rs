use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::OffsetDateTime;
use tower_cookies::Cookies;

use bastion_core::HUB_NODE_ID;
use bastion_storage::notifications_repo;
use bastion_storage::secrets_repo;

use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};
use super::SecretListItem;

pub(in crate::http) async fn list_wecom_bot_secrets(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<SecretListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let secrets = secrets_repo::list_secrets(&state.db, HUB_NODE_ID, "wecom_bot").await?;
    Ok(Json(
        secrets
            .into_iter()
            .map(|s| SecretListItem {
                name: s.name,
                updated_at: s.updated_at,
            })
            .collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct UpsertWecomBotSecretRequest {
    webhook_url: String,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct WecomBotSecretResponse {
    name: String,
    webhook_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WecomBotSecretPayload {
    webhook_url: String,
}

pub(in crate::http) async fn upsert_wecom_bot_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(req): Json<UpsertWecomBotSecretRequest>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    if name.trim().is_empty() {
        return Err(
            AppError::bad_request("invalid_name", "Secret name is required")
                .with_details(json!({ "field": "name" })),
        );
    }

    let webhook_url = req.webhook_url.trim();
    if webhook_url.is_empty() {
        return Err(
            AppError::bad_request("invalid_webhook_url", "Webhook URL is required")
                .with_details(json!({ "field": "webhook_url" })),
        );
    }
    let url = url::Url::parse(webhook_url).map_err(|_| {
        AppError::bad_request("invalid_webhook_url", "Webhook URL is invalid")
            .with_details(json!({ "field": "webhook_url" }))
    })?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(
            AppError::bad_request("invalid_webhook_url", "Webhook URL must be http(s)")
                .with_details(json!({ "field": "webhook_url" })),
        );
    }

    let payload = WecomBotSecretPayload {
        webhook_url: webhook_url.to_string(),
    };
    let bytes = serde_json::to_vec(&payload)?;

    secrets_repo::upsert_secret(
        &state.db,
        &state.secrets,
        HUB_NODE_ID,
        "wecom_bot",
        name.trim(),
        &bytes,
    )
    .await?;
    tracing::info!(
        secret_kind = "wecom_bot",
        secret_name = %name.trim(),
        "secret upserted"
    );
    Ok(StatusCode::NO_CONTENT)
}

pub(in crate::http) async fn get_wecom_bot_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(name): Path<String>,
) -> Result<Json<WecomBotSecretResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let bytes =
        secrets_repo::get_secret(&state.db, &state.secrets, HUB_NODE_ID, "wecom_bot", &name)
            .await?
            .ok_or_else(|| AppError::not_found("secret_not_found", "Secret not found"))?;

    let payload: WecomBotSecretPayload = serde_json::from_slice(&bytes)?;
    Ok(Json(WecomBotSecretResponse {
        name,
        webhook_url: payload.webhook_url,
    }))
}

pub(in crate::http) async fn delete_wecom_bot_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let deleted = secrets_repo::delete_secret(&state.db, HUB_NODE_ID, "wecom_bot", &name).await?;
    if !deleted {
        return Err(AppError::not_found("secret_not_found", "Secret not found"));
    }
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let _ = notifications_repo::cancel_queued_for_destination(
        &state.db,
        notifications_repo::CHANNEL_WECOM_BOT,
        &name,
        "canceled: destination deleted",
        now,
    )
    .await?;
    tracing::info!(secret_kind = "wecom_bot", secret_name = %name, "secret deleted");
    Ok(StatusCode::NO_CONTENT)
}
