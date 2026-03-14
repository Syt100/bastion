use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tower_cookies::Cookies;

use bastion_backup::restore;
use bastion_core::agent_protocol::{HubToAgentMessageV1, PROTOCOL_VERSION};
use bastion_engine::cancel_registry::global_cancel_registry;
use bastion_engine::run_events;
use bastion_storage::agent_tasks_repo;
use bastion_storage::runs_repo;

use super::shared::{require_csrf, require_session};
use super::{AppError, AppState};

fn invalid_kind_error(message: impl Into<String>) -> AppError {
    AppError::bad_request("invalid_kind", message)
        .with_reason("unsupported_value")
        .with_field("kind")
}

#[derive(Debug, Serialize)]
pub(super) struct RunResponse {
    id: String,
    job_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    job_name: Option<String>,
    node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    node_name: Option<String>,
    status: runs_repo::RunStatus,
    started_at: i64,
    ended_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cancel_requested_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cancel_requested_by_user_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cancel_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    progress: Option<serde_json::Value>,
    summary: Option<serde_json::Value>,
    error: Option<String>,
}

async fn get_run_response(db: &sqlx::SqlitePool, run_id: &str) -> Result<Option<RunResponse>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT
          r.id AS id,
          r.job_id AS job_id,
          r.status AS status,
          r.started_at AS started_at,
          r.ended_at AS ended_at,
          r.cancel_requested_at AS cancel_requested_at,
          r.cancel_requested_by_user_id AS cancel_requested_by_user_id,
          r.cancel_reason AS cancel_reason,
          r.progress_json AS progress_json,
          r.summary_json AS summary_json,
          r.error AS error,
          j.name AS job_name,
          COALESCE(j.agent_id, 'hub') AS node_id,
          a.name AS node_name
        FROM runs r
        JOIN jobs j ON j.id = r.job_id
        LEFT JOIN agents a ON a.id = j.agent_id
        WHERE r.id = ?
        LIMIT 1
        "#,
    )
    .bind(run_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let status = row.get::<String, _>("status").parse::<runs_repo::RunStatus>()?;
    let progress_json = row.get::<Option<String>, _>("progress_json");
    let progress = progress_json
        .map(|value| serde_json::from_str::<serde_json::Value>(&value))
        .transpose()?;
    let summary_json = row.get::<Option<String>, _>("summary_json");
    let summary = summary_json
        .map(|value| serde_json::from_str::<serde_json::Value>(&value))
        .transpose()?;

    Ok(Some(RunResponse {
        id: row.get::<String, _>("id"),
        job_id: row.get::<String, _>("job_id"),
        job_name: row.get::<Option<String>, _>("job_name"),
        node_id: row.get::<String, _>("node_id"),
        node_name: row.get::<Option<String>, _>("node_name"),
        status,
        started_at: row.get::<i64, _>("started_at"),
        ended_at: row.get::<Option<i64>, _>("ended_at"),
        cancel_requested_at: row.get::<Option<i64>, _>("cancel_requested_at"),
        cancel_requested_by_user_id: row.get::<Option<i64>, _>("cancel_requested_by_user_id"),
        cancel_reason: row.get::<Option<String>, _>("cancel_reason"),
        progress,
        summary,
        error: row.get::<Option<String>, _>("error"),
    }))
}

pub(super) async fn get_run(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(run_id): Path<String>,
) -> Result<Json<RunResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let run = get_run_response(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;

    Ok(Json(run))
}

#[derive(Debug, Deserialize)]
pub(super) struct CancelRunRequest {
    #[serde(default)]
    reason: Option<String>,
}

pub(super) async fn cancel_run(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(run_id): Path<String>,
    Json(req): Json<CancelRunRequest>,
) -> Result<Json<RunResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let before = runs_repo::get_run(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;

    let run =
        runs_repo::request_run_cancel(&state.db, &run_id, session.user_id, req.reason.as_deref())
            .await?
            .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;

    if before.status == runs_repo::RunStatus::Running {
        let _ = run_events::append_and_broadcast(
            &state.db,
            &state.run_events_bus,
            &run.id,
            "info",
            "cancel_requested",
            "cancel requested",
            None,
        )
        .await;
        let _ = global_cancel_registry().cancel_run(&run.id);
        if let Ok(Some(task)) = agent_tasks_repo::get_task(&state.db, &run.id).await
            && task.completed_at.is_none()
        {
            let _ = state
                .agent_manager
                .send_json(
                    &task.agent_id,
                    &HubToAgentMessageV1::CancelRunTask {
                        v: PROTOCOL_VERSION,
                        run_id: run.id.clone(),
                    },
                )
                .await;
        }
    } else if before.status == runs_repo::RunStatus::Queued
        && run.status == runs_repo::RunStatus::Canceled
    {
        let _ = run_events::append_and_broadcast(
            &state.db,
            &state.run_events_bus,
            &run.id,
            "info",
            "canceled",
            "canceled",
            None,
        )
        .await;
    }

    let response = get_run_response(&state.db, &run.id)
        .await?
        .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub(super) struct ListRunEntriesQuery {
    #[serde(default)]
    prefix: Option<String>,
    #[serde(default)]
    cursor: Option<u64>,
    #[serde(default)]
    limit: Option<u64>,
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    hide_dotfiles: Option<bool>,
    #[serde(default)]
    min_size_bytes: Option<u64>,
    #[serde(default)]
    max_size_bytes: Option<u64>,
    #[serde(default)]
    type_sort: Option<String>,
}

pub(super) async fn list_run_entries(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(run_id): Path<String>,
    Query(query): Query<ListRunEntriesQuery>,
) -> Result<Json<restore::RunEntriesChildrenResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let ListRunEntriesQuery {
        prefix,
        cursor,
        limit,
        q,
        kind,
        hide_dotfiles,
        min_size_bytes,
        max_size_bytes,
        type_sort,
    } = query;

    let cursor = cursor.unwrap_or(0);
    let limit = limit.unwrap_or(200);
    let kind = match kind.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        None => None,
        Some(v) if matches!(v, "file" | "dir" | "symlink") => Some(v.to_string()),
        Some(_) => return Err(invalid_kind_error("invalid kind")),
    };
    let hide_dotfiles = hide_dotfiles.unwrap_or(false);
    let (min_size_bytes, max_size_bytes) = match (min_size_bytes, max_size_bytes) {
        (Some(a), Some(b)) if a > b => (Some(b), Some(a)),
        other => other,
    };
    let type_sort = type_sort
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let type_sort_file_first = match type_sort {
        None | Some("dir_first") => false,
        Some("file_first") => true,
        Some(_) => {
            return Err(
                AppError::bad_request("invalid_type_sort", "invalid type_sort")
                    .with_details(serde_json::json!({ "field": "type_sort" })),
            );
        }
    };

    let result = restore::list_run_entries_children_with_options(
        &state.db,
        state.secrets.as_ref(),
        &state.config.data_dir,
        &run_id,
        restore::ListRunEntriesChildrenOptions {
            prefix,
            cursor,
            limit,
            q,
            kind,
            hide_dotfiles,
            min_size_bytes,
            max_size_bytes,
            type_sort_file_first,
        },
    )
    .await;

    match result {
        Ok(v) => Ok(Json(v)),
        Err(error) => {
            let msg = format!("{error:#}");
            if msg.contains("run not found") {
                return Err(AppError::not_found("run_not_found", "Run not found"));
            }
            Err(AppError::bad_request(
                "run_entries_failed",
                format!("Run entries list failed: {error}"),
            ))
        }
    }
}
