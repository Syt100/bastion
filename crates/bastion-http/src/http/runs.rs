use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;

use bastion_backup::restore;
use bastion_storage::runs_repo;

use super::shared::require_session;
use super::{AppError, AppState};

#[derive(Debug, Serialize)]
pub(super) struct RunResponse {
    id: String,
    job_id: String,
    status: runs_repo::RunStatus,
    started_at: i64,
    ended_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    progress: Option<serde_json::Value>,
    summary: Option<serde_json::Value>,
    error: Option<String>,
}

pub(super) async fn get_run(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(run_id): Path<String>,
) -> Result<Json<RunResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let run = runs_repo::get_run(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;

    Ok(Json(RunResponse {
        id: run.id,
        job_id: run.job_id,
        status: run.status,
        started_at: run.started_at,
        ended_at: run.ended_at,
        progress: run.progress,
        summary: run.summary,
        error: run.error,
    }))
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
        Some(_) => {
            return Err(AppError::bad_request("invalid_kind", "invalid kind")
                .with_details(serde_json::json!({ "field": "kind" })));
        }
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
