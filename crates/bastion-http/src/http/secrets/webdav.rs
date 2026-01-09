use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_cookies::Cookies;

use bastion_core::HUB_NODE_ID;
use bastion_storage::secrets_repo;

use super::super::agents::send_node_config_snapshot;
use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};
use super::SecretListItem;
use super::node_validation::validate_node_id;

pub(in crate::http) async fn list_webdav_secrets(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<SecretListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    list_webdav_secrets_for_node(&state, HUB_NODE_ID).await
}

pub(in crate::http) async fn list_webdav_secrets_node(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(node_id): Path<String>,
) -> Result<Json<Vec<SecretListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    validate_node_id(&state.db, &node_id).await?;

    list_webdav_secrets_for_node(&state, node_id.trim()).await
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct UpsertWebdavSecretRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct WebdavSecretResponse {
    name: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebdavSecretPayload {
    username: String,
    password: String,
}

pub(in crate::http) async fn upsert_webdav_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(req): Json<UpsertWebdavSecretRequest>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    upsert_webdav_secret_for_node(&state, HUB_NODE_ID, &name, req).await?;
    tracing::info!(secret_kind = "webdav", secret_name = %name.trim(), "secret upserted");
    Ok(StatusCode::NO_CONTENT)
}

pub(in crate::http) async fn get_webdav_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(name): Path<String>,
) -> Result<Json<WebdavSecretResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let payload = load_webdav_secret_payload(&state, HUB_NODE_ID, &name).await?;
    Ok(Json(WebdavSecretResponse {
        name,
        username: payload.username,
        password: payload.password,
    }))
}

pub(in crate::http) async fn delete_webdav_secret(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;
    delete_webdav_secret_for_node(&state, HUB_NODE_ID, &name).await?;
    tracing::info!(secret_kind = "webdav", secret_name = %name, "secret deleted");
    Ok(StatusCode::NO_CONTENT)
}

pub(in crate::http) async fn upsert_webdav_secret_node(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path((node_id, name)): Path<(String, String)>,
    Json(req): Json<UpsertWebdavSecretRequest>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    validate_node_id(&state.db, &node_id).await?;

    let node_id_trimmed = node_id.trim();
    upsert_webdav_secret_for_node(&state, node_id_trimmed, &name, req).await?;
    tracing::info!(
        node_id = %node_id_trimmed,
        secret_kind = "webdav",
        secret_name = %name.trim(),
        "secret upserted"
    );
    maybe_send_node_config_snapshot(&state, node_id_trimmed).await;

    Ok(StatusCode::NO_CONTENT)
}

pub(in crate::http) async fn get_webdav_secret_node(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path((node_id, name)): Path<(String, String)>,
) -> Result<Json<WebdavSecretResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    validate_node_id(&state.db, &node_id).await?;
    let payload = load_webdav_secret_payload(&state, node_id.trim(), &name).await?;
    Ok(Json(WebdavSecretResponse {
        name,
        username: payload.username,
        password: payload.password,
    }))
}

pub(in crate::http) async fn delete_webdav_secret_node(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path((node_id, name)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;
    validate_node_id(&state.db, &node_id).await?;

    let node_id_trimmed = node_id.trim();
    delete_webdav_secret_for_node(&state, node_id_trimmed, &name).await?;
    tracing::info!(
        node_id = %node_id_trimmed,
        secret_kind = "webdav",
        secret_name = %name,
        "secret deleted"
    );
    maybe_send_node_config_snapshot(&state, node_id_trimmed).await;

    Ok(StatusCode::NO_CONTENT)
}

async fn list_webdav_secrets_for_node(
    state: &AppState,
    node_id: &str,
) -> Result<Json<Vec<SecretListItem>>, AppError> {
    let secrets = secrets_repo::list_secrets(&state.db, node_id, "webdav").await?;
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

fn validate_webdav_secret_name(name: &str) -> Result<&str, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(
            AppError::bad_request("invalid_name", "Secret name is required")
                .with_details(json!({ "field": "name" })),
        );
    }
    Ok(name)
}

fn validate_webdav_secret_username(username: &str) -> Result<&str, AppError> {
    let username = username.trim();
    if username.is_empty() {
        return Err(
            AppError::bad_request("invalid_username", "Username is required")
                .with_details(json!({ "field": "username" })),
        );
    }
    Ok(username)
}

async fn upsert_webdav_secret_for_node(
    state: &AppState,
    node_id: &str,
    name: &str,
    req: UpsertWebdavSecretRequest,
) -> Result<(), AppError> {
    let name = validate_webdav_secret_name(name)?;
    let username = validate_webdav_secret_username(&req.username)?;

    let payload = WebdavSecretPayload {
        username: username.to_string(),
        password: req.password,
    };
    let bytes = serde_json::to_vec(&payload)?;

    secrets_repo::upsert_secret(&state.db, &state.secrets, node_id, "webdav", name, &bytes).await?;
    Ok(())
}

async fn load_webdav_secret_payload(
    state: &AppState,
    node_id: &str,
    name: &str,
) -> Result<WebdavSecretPayload, AppError> {
    let bytes = secrets_repo::get_secret(&state.db, &state.secrets, node_id, "webdav", name)
        .await?
        .ok_or_else(|| AppError::not_found("secret_not_found", "Secret not found"))?;

    Ok(serde_json::from_slice(&bytes)?)
}

async fn delete_webdav_secret_for_node(
    state: &AppState,
    node_id: &str,
    name: &str,
) -> Result<(), AppError> {
    let deleted = secrets_repo::delete_secret(&state.db, node_id, "webdav", name).await?;
    if !deleted {
        return Err(AppError::not_found("secret_not_found", "Secret not found"));
    }
    Ok(())
}

async fn maybe_send_node_config_snapshot(state: &AppState, node_id: &str) {
    if node_id == HUB_NODE_ID {
        return;
    }

    if let Err(error) = send_node_config_snapshot(
        &state.db,
        state.secrets.as_ref(),
        &state.agent_manager,
        node_id,
    )
    .await
    {
        tracing::warn!(
            node_id = %node_id,
            error = %error,
            "failed to send agent config snapshot"
        );
    }
}
