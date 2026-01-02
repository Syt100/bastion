use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::OffsetDateTime;
use tower_cookies::Cookies;

use super::shared::{require_csrf, require_session};
use super::{AppError, AppState};
use bastion_notify::smtp::{SmtpSecretPayload, SmtpTlsMode, is_valid_mailbox};
use bastion_storage::notifications_repo;
use bastion_storage::secrets_repo;

#[derive(Debug, Serialize)]
pub(super) struct SecretListItem {
    name: String,
    updated_at: i64,
}

pub(super) async fn list_webdav_secrets(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<SecretListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let secrets = secrets_repo::list_secrets(&state.db, "webdav").await?;
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

pub(super) async fn list_wecom_bot_secrets(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<SecretListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let secrets = secrets_repo::list_secrets(&state.db, "wecom_bot").await?;
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

pub(super) async fn list_smtp_secrets(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<SecretListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let secrets = secrets_repo::list_secrets(&state.db, "smtp").await?;
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
pub(super) struct UpsertWebdavSecretRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub(super) struct WebdavSecretResponse {
    name: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebdavSecretPayload {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct UpsertWecomBotSecretRequest {
    webhook_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct WecomBotSecretResponse {
    name: String,
    webhook_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WecomBotSecretPayload {
    webhook_url: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct UpsertSmtpSecretRequest {
    host: String,
    port: u16,
    username: String,
    password: String,
    from: String,
    to: Vec<String>,
    tls: SmtpTlsMode,
}

#[derive(Debug, Serialize)]
pub(super) struct SmtpSecretResponse {
    name: String,
    host: String,
    port: u16,
    username: String,
    password: String,
    from: String,
    to: Vec<String>,
    tls: SmtpTlsMode,
}

pub(super) async fn upsert_webdav_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(req): Json<UpsertWebdavSecretRequest>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    if name.trim().is_empty() {
        return Err(
            AppError::bad_request("invalid_name", "Secret name is required")
                .with_details(json!({ "field": "name" })),
        );
    }
    if req.username.trim().is_empty() {
        return Err(
            AppError::bad_request("invalid_username", "Username is required")
                .with_details(json!({ "field": "username" })),
        );
    }

    let payload = WebdavSecretPayload {
        username: req.username.trim().to_string(),
        password: req.password,
    };
    let bytes = serde_json::to_vec(&payload)?;

    secrets_repo::upsert_secret(&state.db, &state.secrets, "webdav", name.trim(), &bytes).await?;
    tracing::info!(secret_kind = "webdav", secret_name = %name.trim(), "secret upserted");
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn get_webdav_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(name): Path<String>,
) -> Result<Json<WebdavSecretResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let bytes = secrets_repo::get_secret(&state.db, &state.secrets, "webdav", &name)
        .await?
        .ok_or_else(|| AppError::not_found("secret_not_found", "Secret not found"))?;

    let payload: WebdavSecretPayload = serde_json::from_slice(&bytes)?;
    Ok(Json(WebdavSecretResponse {
        name,
        username: payload.username,
        password: payload.password,
    }))
}

pub(super) async fn delete_webdav_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let deleted = secrets_repo::delete_secret(&state.db, "webdav", &name).await?;
    if !deleted {
        return Err(AppError::not_found("secret_not_found", "Secret not found"));
    }
    tracing::info!(secret_kind = "webdav", secret_name = %name, "secret deleted");
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn upsert_wecom_bot_secret(
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

    secrets_repo::upsert_secret(&state.db, &state.secrets, "wecom_bot", name.trim(), &bytes)
        .await?;
    tracing::info!(
        secret_kind = "wecom_bot",
        secret_name = %name.trim(),
        "secret upserted"
    );
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn get_wecom_bot_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(name): Path<String>,
) -> Result<Json<WecomBotSecretResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let bytes = secrets_repo::get_secret(&state.db, &state.secrets, "wecom_bot", &name)
        .await?
        .ok_or_else(|| AppError::not_found("secret_not_found", "Secret not found"))?;

    let payload: WecomBotSecretPayload = serde_json::from_slice(&bytes)?;
    Ok(Json(WecomBotSecretResponse {
        name,
        webhook_url: payload.webhook_url,
    }))
}

pub(super) async fn delete_wecom_bot_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let deleted = secrets_repo::delete_secret(&state.db, "wecom_bot", &name).await?;
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

pub(super) async fn upsert_smtp_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(req): Json<UpsertSmtpSecretRequest>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    if name.trim().is_empty() {
        return Err(
            AppError::bad_request("invalid_name", "Secret name is required")
                .with_details(json!({ "field": "name" })),
        );
    }

    let host = req.host.trim();
    if host.is_empty() {
        return Err(
            AppError::bad_request("invalid_host", "SMTP host is required")
                .with_details(json!({ "field": "host" })),
        );
    }
    if req.port == 0 {
        return Err(
            AppError::bad_request("invalid_port", "SMTP port is required")
                .with_details(json!({ "field": "port" })),
        );
    }

    let from = req.from.trim();
    if from.is_empty() {
        return Err(
            AppError::bad_request("invalid_from", "SMTP from is required")
                .with_details(json!({ "field": "from" })),
        );
    }
    if !is_valid_mailbox(from) {
        return Err(
            AppError::bad_request("invalid_from", "Invalid SMTP from address")
                .with_details(json!({ "field": "from" })),
        );
    }

    let mut to = Vec::new();
    for (index, item) in req.to.into_iter().enumerate() {
        let addr = item.trim();
        if addr.is_empty() {
            continue;
        }
        if !is_valid_mailbox(addr) {
            return Err(
                AppError::bad_request("invalid_to", "Invalid SMTP recipient address")
                    .with_details(json!({ "field": "to", "index": index })),
            );
        }
        to.push(addr.to_string());
    }
    if to.is_empty() {
        return Err(AppError::bad_request("invalid_to", "SMTP to is required")
            .with_details(json!({ "field": "to" })));
    }

    let username = req.username.trim().to_string();
    if !username.is_empty() && req.password.trim().is_empty() {
        return Err(AppError::bad_request(
            "invalid_password",
            "SMTP password is required when username is set",
        )
        .with_details(json!({ "field": "password" })));
    }

    let payload = SmtpSecretPayload {
        host: host.to_string(),
        port: req.port,
        username,
        password: req.password,
        from: from.to_string(),
        to,
        tls: req.tls,
    };
    let bytes = serde_json::to_vec(&payload)?;

    secrets_repo::upsert_secret(&state.db, &state.secrets, "smtp", name.trim(), &bytes).await?;
    tracing::info!(secret_kind = "smtp", secret_name = %name.trim(), "secret upserted");
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn get_smtp_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(name): Path<String>,
) -> Result<Json<SmtpSecretResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let bytes = secrets_repo::get_secret(&state.db, &state.secrets, "smtp", &name)
        .await?
        .ok_or_else(|| AppError::not_found("secret_not_found", "Secret not found"))?;

    let payload: SmtpSecretPayload = serde_json::from_slice(&bytes)?;
    Ok(Json(SmtpSecretResponse {
        name,
        host: payload.host,
        port: payload.port,
        username: payload.username,
        password: payload.password,
        from: payload.from,
        to: payload.to,
        tls: payload.tls,
    }))
}

pub(super) async fn delete_smtp_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let deleted = secrets_repo::delete_secret(&state.db, "smtp", &name).await?;
    if !deleted {
        return Err(AppError::not_found("secret_not_found", "Secret not found"));
    }
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let _ = notifications_repo::cancel_queued_for_destination(
        &state.db,
        notifications_repo::CHANNEL_EMAIL,
        &name,
        "canceled: destination deleted",
        now,
    )
    .await?;
    tracing::info!(secret_kind = "smtp", secret_name = %name, "secret deleted");
    Ok(StatusCode::NO_CONTENT)
}
