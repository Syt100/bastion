use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use bastion_storage::agent_labels_repo;

use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};

const LABEL_MAX_LEN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LabelsMode {
    And,
    Or,
}

pub(super) fn parse_labels_mode(value: Option<&str>) -> Result<LabelsMode, AppError> {
    let value = value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("and");

    match value {
        "and" => Ok(LabelsMode::And),
        "or" => Ok(LabelsMode::Or),
        _ => Err(AppError::bad_request(
            "invalid_labels_mode",
            "Invalid labels_mode",
        )),
    }
}

pub(super) fn normalize_labels(values: Vec<String>) -> Result<Vec<String>, AppError> {
    let mut out = Vec::new();
    for v in values {
        let label = validate_label(&v)?;
        out.push(label);
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn validate_label(value: &str) -> Result<String, AppError> {
    let label = value.trim();
    if label.is_empty() {
        return Err(AppError::bad_request("invalid_label", "Label is required")
            .with_details(serde_json::json!({ "field": "labels" })));
    }
    if label.len() > LABEL_MAX_LEN {
        return Err(AppError::bad_request("invalid_label", "Label is too long")
            .with_details(serde_json::json!({ "max_len": LABEL_MAX_LEN })));
    }
    if !label.is_ascii() {
        return Err(AppError::bad_request(
            "invalid_label",
            "Label must be ASCII lowercase",
        ));
    }
    if label != label.to_ascii_lowercase() {
        return Err(AppError::bad_request(
            "invalid_label",
            "Label must be lowercase",
        ));
    }
    if !label
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return Err(AppError::bad_request(
            "invalid_label",
            "Label contains invalid characters",
        ));
    }
    if label.chars().next().is_some_and(|c| c == '-' || c == '_') {
        return Err(AppError::bad_request(
            "invalid_label",
            "Label must start with a letter or digit",
        ));
    }
    Ok(label.to_string())
}

async fn ensure_agent_exists(db: &SqlitePool, agent_id: &str) -> Result<(), AppError> {
    let row = sqlx::query("SELECT 1 FROM agents WHERE id = ? LIMIT 1")
        .bind(agent_id)
        .fetch_optional(db)
        .await?;
    if row.is_none() {
        return Err(AppError::not_found("agent_not_found", "Agent not found"));
    }
    Ok(())
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct AgentLabelIndexItem {
    label: String,
    count: i64,
}

pub(in crate::http) async fn list_agent_labels_index(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<AgentLabelIndexItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let counts = agent_labels_repo::list_label_counts(&state.db).await?;
    Ok(Json(
        counts
            .into_iter()
            .map(|c| AgentLabelIndexItem {
                label: c.label,
                count: c.count,
            })
            .collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct UpdateAgentLabelsRequest {
    labels: Vec<String>,
}

pub(in crate::http) async fn set_agent_labels(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
    Json(req): Json<UpdateAgentLabelsRequest>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    ensure_agent_exists(&state.db, &agent_id).await?;
    let labels = normalize_labels(req.labels)?;
    agent_labels_repo::set_labels(&state.db, &agent_id, &labels).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(in crate::http) async fn add_agent_labels(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
    Json(req): Json<UpdateAgentLabelsRequest>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    ensure_agent_exists(&state.db, &agent_id).await?;
    let labels = normalize_labels(req.labels)?;
    agent_labels_repo::add_labels(&state.db, &agent_id, &labels).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(in crate::http) async fn remove_agent_labels(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
    Json(req): Json<UpdateAgentLabelsRequest>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    ensure_agent_exists(&state.db, &agent_id).await?;
    let labels = normalize_labels(req.labels)?;
    agent_labels_repo::remove_labels(&state.db, &agent_id, &labels).await?;
    Ok(StatusCode::NO_CONTENT)
}
