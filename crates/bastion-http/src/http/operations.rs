use axum::Json;
use axum::extract::Path;
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;

use super::shared::{require_csrf, require_session};
use super::{AppError, AppState};
use bastion_backup::backup_encryption;
use bastion_backup::restore;
use bastion_core::HUB_NODE_ID;
use bastion_core::agent_protocol::{
    HubToAgentMessageV1, PROTOCOL_VERSION, RestoreSelectionV1, RestoreTaskV1,
};
use bastion_core::job_spec;
use bastion_engine::agent_snapshots;
use bastion_storage::agent_tasks_repo;
use bastion_storage::jobs_repo;
use bastion_storage::operations_repo;
use bastion_storage::runs_repo;

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
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

    let job = jobs_repo::get_job(&state.db, &run.job_id)
        .await?
        .ok_or_else(|| AppError::not_found("job_not_found", "Job not found"))?;
    let run_node_id = job
        .agent_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(HUB_NODE_ID)
        .to_string();

    let destination = match &req.destination {
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
            req.executor
                .as_ref()
                .map(|e| e.node_id.trim().to_string())
                .filter(|v| !v.is_empty())
                .map(|executor_node_id| {
                    if executor_node_id != node_id {
                        Err(AppError::bad_request(
                            "invalid_executor",
                            "executor.node_id must match destination.node_id for local_fs destinations",
                        ))
                    } else {
                        Ok(executor_node_id)
                    }
                })
                .transpose()?;

            (
                node_id.to_string(),
                restore::RestoreDestination::LocalFs {
                    directory: std::path::PathBuf::from(directory),
                },
                bastion_core::agent_protocol::RestoreDestinationV1::LocalFs {
                    directory: directory.to_string(),
                },
            )
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

            (
                run_node_id.clone(),
                restore::RestoreDestination::Webdav {
                    base_url: base_url.to_string(),
                    secret_name: secret_name.to_string(),
                    prefix: prefix.to_string(),
                },
                bastion_core::agent_protocol::RestoreDestinationV1::Webdav {
                    base_url: base_url.to_string(),
                    secret_name: secret_name.to_string(),
                    prefix: prefix.to_string(),
                },
            )
        }
    };
    let (default_executor_node_id, destination_for_hub, destination_for_agent) = destination;

    let executor_node_id = req
        .executor
        .as_ref()
        .map(|e| e.node_id.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or(default_executor_node_id);

    let op = operations_repo::create_operation(
        &state.db,
        operations_repo::OperationKind::Restore,
        Some(("run", run_id.as_str())),
    )
    .await?;
    let _ = operations_repo::append_event(
        &state.db,
        &op.id,
        "info",
        "requested",
        "requested",
        Some(serde_json::json!({
            "run_id": run_id.clone(),
            "destination": serde_json::to_value(&req.destination).ok(),
            "executor": serde_json::json!({ "node_id": executor_node_id }),
            "conflict_policy": conflict.as_str(),
            "selection": req.selection.as_ref().map(|s| serde_json::json!({
                "files": s.files.len(),
                "dirs": s.dirs.len(),
            })),
        })),
    )
    .await;

    if executor_node_id != HUB_NODE_ID {
        if !state.agent_manager.is_connected(&executor_node_id).await {
            let _ = operations_repo::complete_operation(
                &state.db,
                &op.id,
                operations_repo::OperationStatus::Failed,
                None,
                Some("destination agent is not connected"),
            )
            .await;
            return Err(AppError::bad_request(
                "agent_not_connected",
                "executor agent is not connected",
            ));
        }

        // Ensure the destination agent has any required decryption key before we dispatch the task,
        // so the agent can immediately start restoring once it pulls the manifest/payload.
        let spec = match job_spec::parse_value(&job.spec) {
            Ok(spec) => spec,
            Err(error) => {
                let msg = format!("{error:#}");
                let _ = operations_repo::complete_operation(
                    &state.db,
                    &op.id,
                    operations_repo::OperationStatus::Failed,
                    None,
                    Some(&msg),
                )
                .await;
                return Err(AppError::bad_request("invalid_job_spec", msg));
            }
        };
        if let Err(error) = job_spec::validate(&spec) {
            let msg = format!("{error:#}");
            let _ = operations_repo::complete_operation(
                &state.db,
                &op.id,
                operations_repo::OperationStatus::Failed,
                None,
                Some(&msg),
            )
            .await;
            return Err(AppError::bad_request("invalid_job_spec", msg));
        }

        let pipeline = match &spec {
            job_spec::JobSpecV1::Filesystem { pipeline, .. } => pipeline,
            job_spec::JobSpecV1::Sqlite { pipeline, .. } => pipeline,
            job_spec::JobSpecV1::Vaultwarden { pipeline, .. } => pipeline,
        };
        if let job_spec::EncryptionV1::AgeX25519 { key_name } = &pipeline.encryption {
            let key_name = key_name.trim();
            if !key_name.is_empty() {
                if let Err(error) = backup_encryption::distribute_age_identity_to_node(
                    &state.db,
                    &state.secrets,
                    &executor_node_id,
                    key_name,
                )
                .await
                {
                    let msg = format!("{error:#}");
                    let _ = operations_repo::complete_operation(
                        &state.db,
                        &op.id,
                        operations_repo::OperationStatus::Failed,
                        None,
                        Some(&msg),
                    )
                    .await;
                    return Err(AppError::bad_request("age_identity_distribute_failed", msg));
                }

                let _ = operations_repo::append_event(
                    &state.db,
                    &op.id,
                    "info",
                    "age_identity",
                    "age_identity",
                    Some(serde_json::json!({
                        "action": "distributed",
                        "node_id": executor_node_id.as_str(),
                        "key_name": key_name,
                    })),
                )
                .await;

                // Send an updated secrets snapshot so the agent can persist the distributed key.
                if let Err(error) = agent_snapshots::send_node_secrets_snapshot(
                    &state.db,
                    &state.secrets,
                    &state.agent_manager,
                    &executor_node_id,
                )
                .await
                {
                    let msg = format!("{error:#}");
                    let _ = operations_repo::complete_operation(
                        &state.db,
                        &op.id,
                        operations_repo::OperationStatus::Failed,
                        None,
                        Some(&msg),
                    )
                    .await;
                    return Err(AppError::bad_request("secrets_snapshot_failed", msg));
                }
            }
        }

        // Ensure any WebDAV destination secret exists in the executor scope.
        if let bastion_core::agent_protocol::RestoreDestinationV1::Webdav { secret_name, .. } =
            &destination_for_agent
            && bastion_storage::secrets_repo::get_secret(
                &state.db,
                &state.secrets,
                &executor_node_id,
                "webdav",
                secret_name,
            )
            .await?
            .is_none()
        {
            let _ = operations_repo::complete_operation(
                &state.db,
                &op.id,
                operations_repo::OperationStatus::Failed,
                None,
                Some("missing webdav secret"),
            )
            .await;
            return Err(AppError::bad_request(
                "missing_webdav_secret",
                "Missing WebDAV secret for executor node",
            ));
        }

        let task = RestoreTaskV1 {
            op_id: op.id.clone(),
            run_id: run_id.clone(),
            destination: Some(destination_for_agent),
            destination_dir: match &req.destination {
                RestoreDestination::LocalFs { directory, .. } => directory.trim().to_string(),
                RestoreDestination::Webdav { .. } => String::new(),
            },
            conflict_policy: conflict.as_str().to_string(),
            selection: req.selection.as_ref().map(|s| RestoreSelectionV1 {
                files: s.files.clone(),
                dirs: s.dirs.clone(),
            }),
        };
        let msg = HubToAgentMessageV1::RestoreTask {
            v: PROTOCOL_VERSION,
            task_id: op.id.clone(),
            task: Box::new(task),
        };
        let payload = serde_json::to_value(&msg)?;
        if let Err(error) = agent_tasks_repo::upsert_task(
            &state.db,
            &op.id,
            &executor_node_id,
            &run_id,
            "sent",
            &payload,
        )
        .await
        {
            let msg = format!("{error:#}");
            let _ = operations_repo::complete_operation(
                &state.db,
                &op.id,
                operations_repo::OperationStatus::Failed,
                None,
                Some(&msg),
            )
            .await;
            return Err(AppError::bad_request("dispatch_failed", msg));
        }
        state
            .agent_manager
            .send_json(&executor_node_id, &msg)
            .await
            .map_err(|e| {
                let msg = format!("{e:#}");
                let msg_for_op = msg.clone();
                // Best-effort: mark operation failed so it doesn't remain stuck in running state.
                // Ignore errors from completion, since we are already returning an error.
                let db = state.db.clone();
                let op_id = op.id.clone();
                tokio::spawn(async move {
                    let _ = operations_repo::complete_operation(
                        &db,
                        &op_id,
                        operations_repo::OperationStatus::Failed,
                        None,
                        Some(&msg_for_op),
                    )
                    .await;
                });
                AppError::bad_request("dispatch_failed", msg)
            })?;

        tracing::info!(
            op_id = %op.id,
            run_id = %run_id,
            executor_node_id = %executor_node_id,
            conflict = %conflict.as_str(),
            "restore dispatched to agent"
        );
        return Ok(Json(StartOperationResponse { op_id: op.id }));
    }

    restore::spawn_restore_operation(
        state.db.clone(),
        state.secrets.clone(),
        state.config.data_dir.clone(),
        op.id.clone(),
        run_id.clone(),
        destination_for_hub,
        conflict,
        req.selection,
    )
    .await;

    tracing::info!(
        op_id = %op.id,
        run_id = %run_id,
        executor_node_id = %executor_node_id,
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

    let op = operations_repo::create_operation(
        &state.db,
        operations_repo::OperationKind::Verify,
        Some(("run", run_id.as_str())),
    )
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    progress: Option<serde_json::Value>,
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
        progress: op.progress,
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

pub(super) async fn list_run_operations(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(run_id): Path<String>,
) -> Result<Json<Vec<OperationResponse>>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let run = runs_repo::get_run(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;

    // Run exists; list operations linked to it.
    let ops = operations_repo::list_operations_by_subject(&state.db, "run", &run.id, 200).await?;
    Ok(Json(
        ops.into_iter()
            .map(|op| OperationResponse {
                id: op.id,
                kind: op.kind,
                status: op.status,
                created_at: op.created_at,
                started_at: op.started_at,
                ended_at: op.ended_at,
                progress: op.progress,
                summary: op.summary,
                error: op.error,
            })
            .collect(),
    ))
}
