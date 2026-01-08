use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tower_cookies::Cookies;

use bastion_storage::jobs_repo;

use super::super::agents::send_node_config_snapshot;
use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};
use super::validation::{validate_job_spec, validate_job_target_scope};
use bastion_engine::scheduler;

#[derive(Debug, Deserialize)]
pub(in crate::http) struct CreateJobRequest {
    name: String,
    agent_id: Option<String>,
    schedule: Option<String>,
    overlap_policy: jobs_repo::OverlapPolicy,
    spec: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct UpdateJobRequest {
    name: String,
    agent_id: Option<String>,
    schedule: Option<String>,
    overlap_policy: jobs_repo::OverlapPolicy,
    spec: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct JobListItem {
    id: String,
    name: String,
    agent_id: Option<String>,
    schedule: Option<String>,
    overlap_policy: jobs_repo::OverlapPolicy,
    created_at: i64,
    updated_at: i64,
}

pub(in crate::http) async fn list_jobs(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<JobListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let jobs = jobs_repo::list_jobs(&state.db).await?;

    Ok(Json(
        jobs.into_iter()
            .map(|j| JobListItem {
                id: j.id,
                name: j.name,
                agent_id: j.agent_id,
                schedule: j.schedule,
                overlap_policy: j.overlap_policy,
                created_at: j.created_at,
                updated_at: j.updated_at,
            })
            .collect(),
    ))
}

pub(in crate::http) async fn create_job(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Json(req): Json<CreateJobRequest>,
) -> Result<Json<jobs_repo::Job>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    if req.name.trim().is_empty() {
        return Err(AppError::bad_request(
            "invalid_name",
            "Job name is required",
        ));
    }

    let schedule = req
        .schedule
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string());

    let agent_id = req
        .agent_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    if let Some(agent_id) = agent_id {
        let row = sqlx::query("SELECT revoked_at FROM agents WHERE id = ? LIMIT 1")
            .bind(agent_id)
            .fetch_optional(&state.db)
            .await?;

        let Some(row) = row else {
            return Err(AppError::bad_request("invalid_agent_id", "Agent not found"));
        };
        if row.get::<Option<i64>, _>("revoked_at").is_some() {
            return Err(AppError::bad_request(
                "invalid_agent_id",
                "Agent is revoked",
            ));
        }
    }

    validate_job_spec(&req.spec)?;
    validate_job_target_scope(&state.db, agent_id, &req.spec).await?;

    if let Some(schedule) = schedule.as_deref() {
        scheduler::validate_cron(schedule)
            .map_err(|_| AppError::bad_request("invalid_schedule", "Invalid cron schedule"))?;
    }

    let job = jobs_repo::create_job(
        &state.db,
        req.name.trim(),
        agent_id,
        schedule.as_deref(),
        req.overlap_policy,
        req.spec,
    )
    .await?;

    tracing::info!(
        job_id = %job.id,
        name = %job.name,
        agent_id = ?job.agent_id,
        schedule = ?job.schedule,
        overlap_policy = ?job.overlap_policy,
        "job created"
    );
    state.jobs_notify.notify_one();

    if let Some(agent_id) = job.agent_id.as_deref()
        && let Err(error) = send_node_config_snapshot(
            &state.db,
            state.secrets.as_ref(),
            &state.agent_manager,
            agent_id,
        )
        .await
    {
        tracing::warn!(agent_id = %agent_id, error = %error, "failed to send agent config snapshot");
    }

    Ok(Json(job))
}

pub(in crate::http) async fn get_job(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(job_id): Path<String>,
) -> Result<Json<jobs_repo::Job>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let job = jobs_repo::get_job(&state.db, &job_id)
        .await?
        .ok_or_else(|| AppError::not_found("job_not_found", "Job not found"))?;
    Ok(Json(job))
}

pub(in crate::http) async fn update_job(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Json(req): Json<UpdateJobRequest>,
) -> Result<Json<jobs_repo::Job>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let previous = jobs_repo::get_job(&state.db, &job_id)
        .await?
        .ok_or_else(|| AppError::not_found("job_not_found", "Job not found"))?;
    let previous_agent_id = previous.agent_id.clone();

    if req.name.trim().is_empty() {
        return Err(AppError::bad_request(
            "invalid_name",
            "Job name is required",
        ));
    }

    let schedule = req
        .schedule
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string());

    let agent_id = req
        .agent_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    if let Some(agent_id) = agent_id {
        let row = sqlx::query("SELECT revoked_at FROM agents WHERE id = ? LIMIT 1")
            .bind(agent_id)
            .fetch_optional(&state.db)
            .await?;

        let Some(row) = row else {
            return Err(AppError::bad_request("invalid_agent_id", "Agent not found"));
        };
        if row.get::<Option<i64>, _>("revoked_at").is_some() {
            return Err(AppError::bad_request(
                "invalid_agent_id",
                "Agent is revoked",
            ));
        }
    }

    validate_job_spec(&req.spec)?;
    validate_job_target_scope(&state.db, agent_id, &req.spec).await?;

    if let Some(schedule) = schedule.as_deref() {
        scheduler::validate_cron(schedule)
            .map_err(|_| AppError::bad_request("invalid_schedule", "Invalid cron schedule"))?;
    }

    let updated = jobs_repo::update_job(
        &state.db,
        &job_id,
        req.name.trim(),
        agent_id,
        schedule.as_deref(),
        req.overlap_policy,
        req.spec,
    )
    .await?;
    if !updated {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }

    let job = jobs_repo::get_job(&state.db, &job_id)
        .await?
        .ok_or_else(|| AppError::not_found("job_not_found", "Job not found"))?;
    let current_agent_id = job.agent_id.clone();

    tracing::info!(
        job_id = %job.id,
        name = %job.name,
        agent_id = ?job.agent_id,
        schedule = ?job.schedule,
        overlap_policy = ?job.overlap_policy,
        "job updated"
    );
    state.jobs_notify.notify_one();

    let mut affected = Vec::new();
    if let Some(agent_id) = previous_agent_id {
        affected.push(agent_id);
    }
    if let Some(agent_id) = current_agent_id
        && !affected.iter().any(|a| a == &agent_id)
    {
        affected.push(agent_id);
    }
    for agent_id in affected {
        if let Err(error) = send_node_config_snapshot(
            &state.db,
            state.secrets.as_ref(),
            &state.agent_manager,
            &agent_id,
        )
        .await
        {
            tracing::warn!(agent_id = %agent_id, error = %error, "failed to send agent config snapshot");
        }
    }

    Ok(Json(job))
}

pub(in crate::http) async fn delete_job(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let previous = jobs_repo::get_job(&state.db, &job_id)
        .await?
        .ok_or_else(|| AppError::not_found("job_not_found", "Job not found"))?;
    let previous_agent_id = previous.agent_id;

    let deleted = jobs_repo::delete_job(&state.db, &job_id).await?;
    if !deleted {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }
    tracing::info!(job_id = %job_id, "job deleted");
    state.jobs_notify.notify_one();

    if let Some(agent_id) = previous_agent_id.as_deref()
        && let Err(error) = send_node_config_snapshot(
            &state.db,
            state.secrets.as_ref(),
            &state.agent_manager,
            agent_id,
        )
        .await
    {
        tracing::warn!(agent_id = %agent_id, error = %error, "failed to send agent config snapshot");
    }

    Ok(StatusCode::NO_CONTENT)
}
