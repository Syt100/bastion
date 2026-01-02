use axum::Json;
use axum::extract::Path;
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;

use super::shared::{require_csrf, require_session};
use super::{AppError, AppState};
use bastion_backup::restore;
use bastion_storage::operations_repo;
use bastion_storage::runs_repo;

#[derive(Debug, Deserialize)]
pub(super) struct StartRestoreRequest {
    destination_dir: String,
    conflict_policy: String,
}

#[derive(Debug, Serialize)]
pub(super) struct StartOperationResponse {
    op_id: String,
}

pub(super) async fn start_restore(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(run_id): Path<String>,
    Json(req): Json<StartRestoreRequest>,
) -> Result<Json<StartOperationResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let destination_dir = req.destination_dir.trim();
    if destination_dir.is_empty() {
        return Err(AppError::bad_request(
            "invalid_destination",
            "destination_dir is required",
        ));
    }

    let conflict = req
        .conflict_policy
        .parse::<restore::ConflictPolicy>()
        .map_err(|_| AppError::bad_request("invalid_conflict_policy", "Invalid conflict policy"))?;

    let run = runs_repo::get_run(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;
    if run.status != runs_repo::RunStatus::Success {
        return Err(AppError::bad_request(
            "invalid_run",
            "Run is not successful",
        ));
    }

    let op = operations_repo::create_operation(&state.db, operations_repo::OperationKind::Restore)
        .await?;
    let _ = operations_repo::append_event(
        &state.db,
        &op.id,
        "info",
        "requested",
        "requested",
        Some(serde_json::json!({
            "run_id": run_id.clone(),
            "destination_dir": destination_dir,
            "conflict_policy": conflict.as_str(),
        })),
    )
    .await;

    restore::spawn_restore_operation(
        state.db.clone(),
        state.secrets.clone(),
        state.config.data_dir.clone(),
        op.id.clone(),
        run_id.clone(),
        std::path::PathBuf::from(destination_dir),
        conflict,
    )
    .await;

    tracing::info!(
        op_id = %op.id,
        run_id = %run_id,
        destination_dir = %destination_dir,
        conflict = %conflict.as_str(),
        "restore requested"
    );
    Ok(Json(StartOperationResponse { op_id: op.id }))
}

pub(super) async fn start_verify(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(run_id): Path<String>,
) -> Result<Json<StartOperationResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let run = runs_repo::get_run(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;
    if run.status != runs_repo::RunStatus::Success {
        return Err(AppError::bad_request(
            "invalid_run",
            "Run is not successful",
        ));
    }

    let op = operations_repo::create_operation(&state.db, operations_repo::OperationKind::Verify)
        .await?;
    let _ = operations_repo::append_event(
        &state.db,
        &op.id,
        "info",
        "requested",
        "requested",
        Some(serde_json::json!({ "run_id": run_id.clone() })),
    )
    .await;

    restore::spawn_verify_operation(
        state.db.clone(),
        state.secrets.clone(),
        state.config.data_dir.clone(),
        op.id.clone(),
        run_id.clone(),
    )
    .await;

    tracing::info!(op_id = %op.id, run_id = %run_id, "verify requested");
    Ok(Json(StartOperationResponse { op_id: op.id }))
}

#[derive(Debug, Serialize)]
pub(super) struct OperationResponse {
    id: String,
    kind: operations_repo::OperationKind,
    status: operations_repo::OperationStatus,
    created_at: i64,
    started_at: i64,
    ended_at: Option<i64>,
    summary: Option<serde_json::Value>,
    error: Option<String>,
}

pub(super) async fn get_operation(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(op_id): Path<String>,
) -> Result<Json<OperationResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let op = operations_repo::get_operation(&state.db, &op_id)
        .await?
        .ok_or_else(|| AppError::not_found("operation_not_found", "Operation not found"))?;
    Ok(Json(OperationResponse {
        id: op.id,
        kind: op.kind,
        status: op.status,
        created_at: op.created_at,
        started_at: op.started_at,
        ended_at: op.ended_at,
        summary: op.summary,
        error: op.error,
    }))
}

pub(super) async fn list_operation_events(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(op_id): Path<String>,
) -> Result<Json<Vec<operations_repo::OperationEvent>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let events = operations_repo::list_events(&state.db, &op_id, 500).await?;
    Ok(Json(events))
}
