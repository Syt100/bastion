use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;

use bastion_storage::bulk_operations_repo;

use super::agents::{LabelsMode, normalize_labels, parse_labels_mode};
use super::shared::{require_csrf, require_session};
use super::{AppError, AppState};

const DEFAULT_LIST_LIMIT: i64 = 50;

fn normalize_node_ids(values: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = values
        .into_iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    out.sort();
    out.dedup();
    out
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct BulkSelectorRequest {
    node_ids: Option<Vec<String>>,
    labels: Option<Vec<String>>,
    labels_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct AgentLabelsPayloadRequest {
    labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct CreateBulkOperationRequest {
    kind: String,
    selector: BulkSelectorRequest,
    payload: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct CreateBulkOperationResponse {
    op_id: String,
}

fn validate_kind(kind: &str) -> Result<&str, AppError> {
    match kind {
        "agent_labels_add" | "agent_labels_remove" | "sync_config_now" => Ok(kind),
        _ => Err(AppError::bad_request("invalid_kind", "Invalid kind")),
    }
}

async fn validate_agent_ids_exist(
    db: &sqlx::SqlitePool,
    agent_ids: &[String],
) -> Result<(), AppError> {
    if agent_ids.is_empty() {
        return Err(AppError::bad_request(
            "invalid_selector",
            "Selector is required",
        ));
    }

    let mut qb: sqlx::QueryBuilder<sqlx::Sqlite> =
        sqlx::QueryBuilder::new("SELECT id FROM agents WHERE id IN (");
    let mut separated = qb.separated(", ");
    for id in agent_ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");

    let rows = qb.build().fetch_all(db).await?;
    if rows.len() != agent_ids.len() {
        return Err(AppError::bad_request(
            "invalid_selector",
            "One or more agents were not found",
        ));
    }
    Ok(())
}

pub(in crate::http) async fn create_bulk_operation(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Json(req): Json<CreateBulkOperationRequest>,
) -> Result<Json<CreateBulkOperationResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let kind = validate_kind(req.kind.trim())?.to_string();

    let node_ids = req.selector.node_ids.unwrap_or_default();
    let selector_labels = req.selector.labels.unwrap_or_default();

    let (target_agent_ids, selector_json) = if !node_ids.is_empty() {
        let ids = normalize_node_ids(node_ids);
        validate_agent_ids_exist(&state.db, &ids).await?;
        (ids.clone(), serde_json::json!({ "node_ids": ids }))
    } else if !selector_labels.is_empty() {
        let labels = normalize_labels(selector_labels)?;
        let mode = parse_labels_mode(req.selector.labels_mode.as_deref())?;
        let mode_str = match mode {
            LabelsMode::And => "and",
            LabelsMode::Or => "or",
        };

        let ids = bulk_operations_repo::resolve_agent_ids_by_selector_labels(
            &state.db, &labels, mode_str,
        )
        .await?;
        if ids.is_empty() {
            return Err(AppError::bad_request(
                "invalid_selector",
                "Selector resolved to no agents",
            ));
        }
        (
            ids,
            serde_json::json!({ "labels": labels, "labels_mode": mode_str }),
        )
    } else {
        return Err(AppError::bad_request(
            "invalid_selector",
            "Selector is required",
        ));
    };

    let payload_json = match kind.as_str() {
        "agent_labels_add" | "agent_labels_remove" => {
            let Some(payload) = req.payload else {
                return Err(AppError::bad_request(
                    "invalid_payload",
                    "Payload is required",
                ));
            };
            let parsed: AgentLabelsPayloadRequest = serde_json::from_value(payload)
                .map_err(|_| AppError::bad_request("invalid_payload", "Invalid payload"))?;

            let payload_labels = normalize_labels(parsed.labels)?;
            if payload_labels.is_empty() {
                return Err(AppError::bad_request(
                    "invalid_payload",
                    "Labels is required",
                ));
            }

            serde_json::json!({ "labels": payload_labels })
        }
        "sync_config_now" => serde_json::json!({}),
        _ => return Err(AppError::bad_request("invalid_kind", "Invalid kind")),
    };

    let op_id = bulk_operations_repo::create_operation(
        &state.db,
        session.user_id,
        &kind,
        &selector_json,
        &payload_json,
        &target_agent_ids,
    )
    .await?;

    state.bulk_ops_notify.notify_one();

    Ok(Json(CreateBulkOperationResponse { op_id }))
}

pub(in crate::http) async fn list_bulk_operations(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<bulk_operations_repo::BulkOperationListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let ops = bulk_operations_repo::list_operations(&state.db, DEFAULT_LIST_LIMIT).await?;
    Ok(Json(ops))
}

pub(in crate::http) async fn get_bulk_operation(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(op_id): Path<String>,
) -> Result<Json<bulk_operations_repo::BulkOperationDetail>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let op = bulk_operations_repo::get_operation(&state.db, &op_id)
        .await?
        .ok_or_else(|| {
            AppError::not_found("bulk_operation_not_found", "Bulk operation not found")
        })?;
    Ok(Json(op))
}

pub(in crate::http) async fn cancel_bulk_operation(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(op_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    bulk_operations_repo::cancel_operation(&state.db, &op_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(in crate::http) async fn retry_bulk_operation_failed(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(op_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let changed = bulk_operations_repo::retry_failed(&state.db, &op_id)
        .await
        .map_err(|_| AppError::bad_request("cannot_retry", "Cannot retry this operation"))?;
    if changed > 0 {
        state.bulk_ops_notify.notify_one();
    }
    Ok(StatusCode::NO_CONTENT)
}
