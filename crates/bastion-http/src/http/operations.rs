use axum::Json;
use axum::extract::Path;
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;

use super::shared::{require_csrf, require_session};
use super::{AppError, AppState};
use bastion_backup::restore;
use bastion_core::HUB_NODE_ID;
use bastion_storage::operations_repo;
use bastion_storage::runs_repo;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum RestoreDestination {
    LocalFs {
        node_id: String,
        directory: String,
    },
    Webdav {
        base_url: String,
        secret_name: String,
        prefix: String,
    },
}

#[derive(Debug, Deserialize)]
pub(super) struct RestoreExecutor {
    node_id: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct StartRestoreRequest {
    destination: RestoreDestination,
    #[serde(default)]
    executor: Option<RestoreExecutor>,
    conflict_policy: String,
    #[serde(default)]
    selection: Option<restore::RestoreSelection>,
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

    let (destination_node_id, destination_dir) = match &req.destination {
        RestoreDestination::LocalFs { node_id, directory } => {
            let node_id = node_id.trim();
            let directory = directory.trim();
            if node_id.is_empty() {
                return Err(AppError::bad_request(
                    "invalid_destination",
                    "destination.node_id is required",
                ));
            }
            if directory.is_empty() {
                return Err(AppError::bad_request(
                    "invalid_destination",
                    "destination.directory is required",
                ));
            }
            (node_id.to_string(), directory.to_string())
        }
        RestoreDestination::Webdav {
            base_url,
            secret_name,
            prefix,
        } => {
            let base_url = base_url.trim();
            let secret_name = secret_name.trim();
            let prefix = prefix.trim();
            if base_url.is_empty() {
                return Err(AppError::bad_request(
                    "invalid_destination",
                    "destination.base_url is required",
                ));
            }
            if secret_name.is_empty() {
                return Err(AppError::bad_request(
                    "invalid_destination",
                    "destination.secret_name is required",
                ));
            }
            if prefix.is_empty() {
                return Err(AppError::bad_request(
                    "invalid_destination",
                    "destination.prefix is required",
                ));
            }
            // TODO(spec): implement webdav destination restore (sink + .bastion-meta sidecar).
            return Err(AppError::bad_request(
                "unsupported_destination",
                "webdav destination is not supported yet",
            ));
        }
    };

    if destination_node_id != HUB_NODE_ID {
        // TODO(spec): support executor=agent and restoring to agent-local filesystems.
        return Err(AppError::bad_request(
            "unsupported_destination",
            "only hub local filesystem destination is supported yet",
        ));
    }

    if let Some(executor) = req.executor.as_ref()
        && executor.node_id.trim() != HUB_NODE_ID
    {
        // TODO(spec): support selecting agent executor (and enforce executor rules).
        return Err(AppError::bad_request(
            "unsupported_executor",
            "only hub executor is supported yet",
        ));
    }

    let conflict = req
        .conflict_policy
        .parse::<restore::ConflictPolicy>()
        .map_err(|_| AppError::bad_request("invalid_conflict_policy", "Invalid conflict policy"))?;

    if let Some(selection) = req.selection.as_ref()
        && selection
            .files
            .iter()
            .chain(selection.dirs.iter())
            .all(|v| v.trim().is_empty())
    {
        return Err(AppError::bad_request(
            "invalid_selection",
            "restore selection is empty",
        ));
    }

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
            "destination": {
                "type": "local_fs",
                "node_id": destination_node_id,
                "directory": destination_dir,
            },
            "executor": req.executor.as_ref().map(|e| serde_json::json!({
                "node_id": e.node_id.trim(),
            })),
            "conflict_policy": conflict.as_str(),
            "selection": req.selection.as_ref().map(|s| serde_json::json!({
                "files": s.files.len(),
                "dirs": s.dirs.len(),
            })),
        })),
    )
    .await;

    restore::spawn_restore_operation(
        state.db.clone(),
        state.secrets.clone(),
        state.config.data_dir.clone(),
        op.id.clone(),
        run_id.clone(),
        std::path::PathBuf::from(&destination_dir),
        conflict,
        req.selection,
    )
    .await;

    tracing::info!(
        op_id = %op.id,
        run_id = %run_id,
        destination_node_id = %destination_node_id,
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
