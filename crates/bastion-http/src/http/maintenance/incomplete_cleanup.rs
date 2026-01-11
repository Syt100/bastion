use axum::Json;
use axum::extract::{Path, Query};
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tower_cookies::Cookies;

use bastion_storage::incomplete_cleanup_repo;

use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};

#[derive(Debug, Deserialize)]
pub(in crate::http) struct ListIncompleteCleanupTasksQuery {
    status: Option<String>,
    target_type: Option<String>,
    node_id: Option<String>,
    job_id: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct ListIncompleteCleanupTasksResponse {
    items: Vec<incomplete_cleanup_repo::CleanupTaskListItem>,
    page: i64,
    page_size: i64,
    total: i64,
}

fn validate_status(status: &str) -> Result<(), AppError> {
    match status {
        "queued" | "running" | "retrying" | "blocked" | "done" | "ignored" | "abandoned" => Ok(()),
        _ => Err(AppError::bad_request("invalid_status", "Invalid status")),
    }
}

fn validate_target_type(target_type: &str) -> Result<(), AppError> {
    match target_type {
        "webdav" | "local_dir" => Ok(()),
        _ => Err(AppError::bad_request(
            "invalid_target_type",
            "Invalid target_type",
        )),
    }
}

pub(in crate::http) async fn list_incomplete_cleanup_tasks(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Query(q): Query<ListIncompleteCleanupTasksQuery>,
) -> Result<Json<ListIncompleteCleanupTasksResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    if let Some(status) = q.status.as_deref() {
        validate_status(status)?;
    }
    if let Some(target_type) = q.target_type.as_deref() {
        validate_target_type(target_type)?;
    }

    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1).saturating_mul(page_size);

    let total = incomplete_cleanup_repo::count_tasks(
        &state.db,
        q.status.as_deref(),
        q.target_type.as_deref(),
        q.node_id.as_deref(),
        q.job_id.as_deref(),
    )
    .await?;

    let items = incomplete_cleanup_repo::list_tasks(
        &state.db,
        q.status.as_deref(),
        q.target_type.as_deref(),
        q.node_id.as_deref(),
        q.job_id.as_deref(),
        page_size,
        offset,
    )
    .await?;

    Ok(Json(ListIncompleteCleanupTasksResponse {
        items,
        page,
        page_size,
        total,
    }))
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct GetIncompleteCleanupTaskResponse {
    task: incomplete_cleanup_repo::CleanupTaskDetail,
    events: Vec<incomplete_cleanup_repo::CleanupEvent>,
}

pub(in crate::http) async fn get_incomplete_cleanup_task(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(run_id): Path<String>,
) -> Result<Json<GetIncompleteCleanupTaskResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let task = incomplete_cleanup_repo::get_task(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("cleanup_task_not_found", "Cleanup task not found"))?;
    let events = incomplete_cleanup_repo::list_events(&state.db, &run_id, 200).await?;

    Ok(Json(GetIncompleteCleanupTaskResponse { task, events }))
}

fn normalize_reason(reason: Option<&str>) -> Option<String> {
    const MAX_LEN: usize = 200;

    let reason = reason.map(str::trim).filter(|s| !s.is_empty())?;
    if reason.len() <= MAX_LEN {
        return Some(reason.to_string());
    }

    Some(format!("{}…", &reason[..MAX_LEN]))
}

pub(in crate::http) async fn retry_incomplete_cleanup_task_now(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(run_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let Some(task) = incomplete_cleanup_repo::get_task(&state.db, &run_id).await? else {
        return Err(AppError::not_found(
            "cleanup_task_not_found",
            "Cleanup task not found",
        ));
    };
    if task.status == "running" {
        return Err(AppError::conflict("task_running", "Task is running"));
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let ok = incomplete_cleanup_repo::retry_now(&state.db, &run_id, now).await?;
    if !ok {
        return Err(AppError::conflict("not_retryable", "Task is not retryable"));
    }

    let _ = incomplete_cleanup_repo::append_event(
        &state.db,
        &run_id,
        "info",
        "retry_now",
        "retry now requested",
        Some(serde_json::json!({ "user_id": session.user_id })),
        now,
    )
    .await;

    state.incomplete_cleanup_notify.notify_one();
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct IgnoreIncompleteCleanupTaskRequest {
    reason: Option<String>,
}

pub(in crate::http) async fn ignore_incomplete_cleanup_task(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(run_id): Path<String>,
    Json(req): Json<IgnoreIncompleteCleanupTaskRequest>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let Some(task) = incomplete_cleanup_repo::get_task(&state.db, &run_id).await? else {
        return Err(AppError::not_found(
            "cleanup_task_not_found",
            "Cleanup task not found",
        ));
    };
    if task.status == "running" {
        return Err(AppError::conflict("task_running", "Task is running"));
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let reason = normalize_reason(req.reason.as_deref());
    let ok = incomplete_cleanup_repo::ignore_task(
        &state.db,
        &run_id,
        Some(session.user_id),
        reason.as_deref(),
        now,
    )
    .await?;
    if !ok {
        return Err(AppError::conflict("not_ignorable", "Task is not ignorable"));
    }

    let _ = incomplete_cleanup_repo::append_event(
        &state.db,
        &run_id,
        "info",
        "ignored",
        "ignored",
        Some(serde_json::json!({ "user_id": session.user_id, "reason": reason })),
        now,
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub(in crate::http) async fn unignore_incomplete_cleanup_task(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(run_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let Some(task) = incomplete_cleanup_repo::get_task(&state.db, &run_id).await? else {
        return Err(AppError::not_found(
            "cleanup_task_not_found",
            "Cleanup task not found",
        ));
    };
    if task.status != "ignored" {
        return Err(AppError::conflict("not_ignored", "Task is not ignored"));
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let ok = incomplete_cleanup_repo::unignore_task(&state.db, &run_id, now).await?;
    if !ok {
        return Err(AppError::conflict("not_ignored", "Task is not ignored"));
    }

    let _ = incomplete_cleanup_repo::append_event(
        &state.db,
        &run_id,
        "info",
        "unignored",
        "unignored",
        Some(serde_json::json!({ "user_id": session.user_id })),
        now,
    )
    .await;

    state.incomplete_cleanup_notify.notify_one();
    Ok(StatusCode::NO_CONTENT)
}
