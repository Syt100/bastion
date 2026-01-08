use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::OffsetDateTime;
use tower_cookies::Cookies;

use bastion_core::HUB_NODE_ID;
use bastion_notify::smtp::{SmtpSecretPayload, SmtpTlsMode, is_valid_mailbox};
use bastion_storage::notifications_repo;
use bastion_storage::secrets_repo;

use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};
use super::SecretListItem;

pub(in crate::http) async fn list_smtp_secrets(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<SecretListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let secrets = secrets_repo::list_secrets(&state.db, HUB_NODE_ID, "smtp").await?;
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
pub(in crate::http) struct UpsertSmtpSecretRequest {
    host: String,
    port: u16,
    username: String,
    password: String,
    from: String,
    to: Vec<String>,
    tls: SmtpTlsMode,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct SmtpSecretResponse {
    name: String,
    host: String,
    port: u16,
    username: String,
    password: String,
    from: String,
    to: Vec<String>,
    tls: SmtpTlsMode,
}

pub(in crate::http) async fn upsert_smtp_secret(
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

    secrets_repo::upsert_secret(
        &state.db,
        &state.secrets,
        HUB_NODE_ID,
        "smtp",
        name.trim(),
        &bytes,
    )
    .await?;
    tracing::info!(secret_kind = "smtp", secret_name = %name.trim(), "secret upserted");
    Ok(StatusCode::NO_CONTENT)
}

pub(in crate::http) async fn get_smtp_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(name): Path<String>,
) -> Result<Json<SmtpSecretResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let bytes = secrets_repo::get_secret(&state.db, &state.secrets, HUB_NODE_ID, "smtp", &name)
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

pub(in crate::http) async fn delete_smtp_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let deleted = secrets_repo::delete_secret(&state.db, HUB_NODE_ID, "smtp", &name).await?;
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
