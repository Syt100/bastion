use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;

use bastion_storage::jobs_repo;
use bastion_storage::run_artifacts_repo;

use super::super::shared::require_session;
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
    let status = query.status.as_deref().map(str::trim).filter(|v| !v.is_empty());

    let items = run_artifacts_repo::list_run_artifacts_for_job(
        &state.db,
        &job_id,
        cursor,
        limit,
        status,
    )
    .await?
    .into_iter()
    .map(RunArtifactResponse::from)
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
        return Err(AppError::not_found("snapshot_not_found", "Snapshot not found"));
    }

    Ok(Json(RunArtifactResponse::from(artifact)))
}
