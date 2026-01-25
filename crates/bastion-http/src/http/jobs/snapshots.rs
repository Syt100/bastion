use std::collections::HashMap;

use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tower_cookies::Cookies;

use bastion_storage::artifact_delete_repo;
use bastion_storage::jobs_repo;
use bastion_storage::run_artifacts_repo;

use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};

#[derive(Debug, Deserialize)]
pub(in crate::http) struct ListJobSnapshotsQuery {
    #[serde(default)]
    cursor: Option<u64>,
    #[serde(default)]
    limit: Option<u64>,
    #[serde(default)]
    status: Option<String>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct RunArtifactResponse {
    run_id: String,
    job_id: String,
    node_id: String,
    target_type: String,
    target_snapshot: serde_json::Value,
    artifact_format: String,
    status: String,
    started_at: i64,
    ended_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_files: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_dirs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transfer_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_error_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_attempt_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delete_task: Option<ArtifactDeleteTaskSummaryResponse>,
}

impl From<run_artifacts_repo::RunArtifact> for RunArtifactResponse {
    fn from(v: run_artifacts_repo::RunArtifact) -> Self {
        Self {
            run_id: v.run_id,
            job_id: v.job_id,
            node_id: v.node_id,
            target_type: v.target_type,
            target_snapshot: v.target_snapshot,
            artifact_format: v.artifact_format,
            status: v.status,
            started_at: v.started_at,
            ended_at: v.ended_at,
            source_files: v.source_files,
            source_dirs: v.source_dirs,
            source_bytes: v.source_bytes,
            transfer_bytes: v.transfer_bytes,
            last_error_kind: v.last_error_kind,
            last_error: v.last_error,
            last_attempt_at: v.last_attempt_at,
            delete_task: None,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(in crate::http) struct ArtifactDeleteTaskSummaryResponse {
    status: String,
    attempts: i64,
    last_attempt_at: Option<i64>,
    next_attempt_at: i64,
    last_error_kind: Option<String>,
    last_error: Option<String>,
    ignored_at: Option<i64>,
}

impl From<artifact_delete_repo::ArtifactDeleteTaskSummary> for ArtifactDeleteTaskSummaryResponse {
    fn from(v: artifact_delete_repo::ArtifactDeleteTaskSummary) -> Self {
        Self {
            status: v.status,
            attempts: v.attempts,
            last_attempt_at: v.last_attempt_at,
            next_attempt_at: v.next_attempt_at,
            last_error_kind: v.last_error_kind,
            last_error: v.last_error,
            ignored_at: v.ignored_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct ListJobSnapshotsResponse {
    items: Vec<RunArtifactResponse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    next_cursor: Option<u64>,
}

pub(in crate::http) async fn list_job_snapshots(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(job_id): Path<String>,
    Query(query): Query<ListJobSnapshotsQuery>,
) -> Result<Json<ListJobSnapshotsResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let job_exists = jobs_repo::get_job(&state.db, &job_id).await?.is_some();
    if !job_exists {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }

    let cursor = query.cursor.unwrap_or(0);
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let status = query
        .status
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());

    let artifacts =
        run_artifacts_repo::list_run_artifacts_for_job(&state.db, &job_id, cursor, limit, status)
            .await?;

    let run_ids = artifacts
        .iter()
        .map(|a| a.run_id.clone())
        .collect::<Vec<_>>();

    let delete_tasks = artifact_delete_repo::list_tasks_by_run_ids(&state.db, &run_ids).await?;
    let mut delete_map = HashMap::<String, artifact_delete_repo::ArtifactDeleteTaskSummary>::new();
    for task in delete_tasks {
        delete_map.insert(task.run_id.clone(), task);
    }

    let items = artifacts
        .into_iter()
        .map(|a| {
            let mut out = RunArtifactResponse::from(a);
            if let Some(task) = delete_map.remove(&out.run_id) {
                out.delete_task = Some(ArtifactDeleteTaskSummaryResponse::from(task));
            }
            out
        })
        .collect::<Vec<_>>();

    let next_cursor = if items.len() as u64 >= limit {
        Some(cursor.saturating_add(limit))
    } else {
        None
    };

    Ok(Json(ListJobSnapshotsResponse { items, next_cursor }))
}

pub(in crate::http) async fn get_job_snapshot(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path((job_id, run_id)): Path<(String, String)>,
) -> Result<Json<RunArtifactResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let job_exists = jobs_repo::get_job(&state.db, &job_id).await?.is_some();
    if !job_exists {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }

    let artifact = run_artifacts_repo::get_run_artifact(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("snapshot_not_found", "Snapshot not found"))?;

    if artifact.job_id != job_id {
        return Err(AppError::not_found(
            "snapshot_not_found",
            "Snapshot not found",
        ));
    }

    Ok(Json(RunArtifactResponse::from(artifact)))
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct BulkDeleteJobSnapshotsRequest {
    run_ids: Vec<String>,
}

fn normalize_reason(reason: Option<&str>) -> Option<String> {
    const MAX_LEN: usize = 200;

    let reason = reason.map(str::trim).filter(|s| !s.is_empty())?;
    if reason.len() <= MAX_LEN {
        return Some(reason.to_string());
    }

    Some(format!("{}…", &reason[..MAX_LEN]))
}

async fn enqueue_snapshot_delete(
    state: &AppState,
    user_id: i64,
    job_id: &str,
    run_id: &str,
    now: i64,
) -> Result<(), AppError> {
    let artifact = run_artifacts_repo::get_run_artifact(&state.db, run_id)
        .await?
        .ok_or_else(|| AppError::not_found("snapshot_not_found", "Snapshot not found"))?;
    if artifact.job_id != job_id {
        return Err(AppError::not_found(
            "snapshot_not_found",
            "Snapshot not found",
        ));
    }

    // Already gone -> idempotent no-op.
    if artifact.status == "deleted" || artifact.status == "missing" {
        return Ok(());
    }

    let snapshot_json = serde_json::to_string(&artifact.target_snapshot)
        .map_err(|_| AppError::bad_request("invalid_snapshot", "Invalid target snapshot"))?;

    let inserted = artifact_delete_repo::upsert_task_if_missing(
        &state.db,
        run_id,
        job_id,
        &artifact.node_id,
        &artifact.target_type,
        &snapshot_json,
        now,
    )
    .await?;

    if inserted {
        let _ = artifact_delete_repo::append_event(
            &state.db,
            run_id,
            "info",
            "queued",
            "delete queued",
            Some(serde_json::json!({ "user_id": user_id })),
            now,
        )
        .await;
    }

    run_artifacts_repo::mark_run_artifact_deleting(&state.db, run_id, now).await?;
    state.artifact_delete_notify.notify_one();
    Ok(())
}

pub(in crate::http) async fn delete_job_snapshot(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path((job_id, run_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let job_exists = jobs_repo::get_job(&state.db, &job_id).await?.is_some();
    if !job_exists {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    enqueue_snapshot_delete(&state, session.user_id, &job_id, &run_id, now).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(in crate::http) async fn delete_job_snapshots_bulk(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Json(req): Json<BulkDeleteJobSnapshotsRequest>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let job_exists = jobs_repo::get_job(&state.db, &job_id).await?.is_some();
    if !job_exists {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }

    let mut run_ids = req
        .run_ids
        .into_iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>();
    run_ids.sort();
    run_ids.dedup();

    if run_ids.is_empty() {
        return Err(AppError::bad_request("empty_run_ids", "Empty run_ids"));
    }
    if run_ids.len() > 200 {
        return Err(AppError::bad_request(
            "too_many_run_ids",
            "Too many run_ids",
        ));
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    for run_id in &run_ids {
        enqueue_snapshot_delete(&state, session.user_id, &job_id, run_id, now).await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

pub(in crate::http) async fn get_job_snapshot_delete_task(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path((job_id, run_id)): Path<(String, String)>,
) -> Result<Json<artifact_delete_repo::ArtifactDeleteTaskDetail>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let job_exists = jobs_repo::get_job(&state.db, &job_id).await?.is_some();
    if !job_exists {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }

    let artifact = run_artifacts_repo::get_run_artifact(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("snapshot_not_found", "Snapshot not found"))?;
    if artifact.job_id != job_id {
        return Err(AppError::not_found(
            "snapshot_not_found",
            "Snapshot not found",
        ));
    }

    let task = artifact_delete_repo::get_task(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("delete_task_not_found", "Delete task not found"))?;
    Ok(Json(task))
}

pub(in crate::http) async fn get_job_snapshot_delete_events(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path((job_id, run_id)): Path<(String, String)>,
) -> Result<Json<Vec<artifact_delete_repo::ArtifactDeleteEvent>>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let job_exists = jobs_repo::get_job(&state.db, &job_id).await?.is_some();
    if !job_exists {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }

    let artifact = run_artifacts_repo::get_run_artifact(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("snapshot_not_found", "Snapshot not found"))?;
    if artifact.job_id != job_id {
        return Err(AppError::not_found(
            "snapshot_not_found",
            "Snapshot not found",
        ));
    }

    let events = artifact_delete_repo::list_events(&state.db, &run_id, 200).await?;
    Ok(Json(events))
}

pub(in crate::http) async fn retry_job_snapshot_delete_now(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path((job_id, run_id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let job_exists = jobs_repo::get_job(&state.db, &job_id).await?.is_some();
    if !job_exists {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }

    let Some(task) = artifact_delete_repo::get_task(&state.db, &run_id).await? else {
        return Err(AppError::not_found(
            "delete_task_not_found",
            "Delete task not found",
        ));
    };
    if task.job_id != job_id {
        return Err(AppError::not_found(
            "delete_task_not_found",
            "Delete task not found",
        ));
    }
    if task.status == "running" {
        return Err(AppError::conflict("task_running", "Task is running"));
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let ok = artifact_delete_repo::retry_now(&state.db, &run_id, now).await?;
    if !ok {
        return Err(AppError::conflict("not_retryable", "Task is not retryable"));
    }

    let _ = artifact_delete_repo::append_event(
        &state.db,
        &run_id,
        "info",
        "retry_now",
        "retry now requested",
        Some(serde_json::json!({ "user_id": session.user_id })),
        now,
    )
    .await;

    let _ = run_artifacts_repo::mark_run_artifact_deleting(&state.db, &run_id, now).await;
    state.artifact_delete_notify.notify_one();
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct IgnoreJobSnapshotDeleteTaskRequest {
    reason: Option<String>,
}

pub(in crate::http) async fn ignore_job_snapshot_delete_task(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path((job_id, run_id)): Path<(String, String)>,
    Json(req): Json<IgnoreJobSnapshotDeleteTaskRequest>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let job_exists = jobs_repo::get_job(&state.db, &job_id).await?.is_some();
    if !job_exists {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }

    let Some(task) = artifact_delete_repo::get_task(&state.db, &run_id).await? else {
        return Err(AppError::not_found(
            "delete_task_not_found",
            "Delete task not found",
        ));
    };
    if task.job_id != job_id {
        return Err(AppError::not_found(
            "delete_task_not_found",
            "Delete task not found",
        ));
    }
    if task.status == "running" {
        return Err(AppError::conflict("task_running", "Task is running"));
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let reason = normalize_reason(req.reason.as_deref());
    let ok = artifact_delete_repo::ignore_task(
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

    let _ = artifact_delete_repo::append_event(
        &state.db,
        &run_id,
        "info",
        "ignored",
        "ignored by user",
        Some(serde_json::json!({ "user_id": session.user_id, "reason": reason })),
        now,
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}
