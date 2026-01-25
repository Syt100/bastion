use axum::Json;
use axum::extract::{Path, Query};
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tower_cookies::Cookies;

use bastion_storage::hub_runtime_config_repo;
use bastion_storage::jobs_repo;
use bastion_storage::{artifact_delete_repo, run_artifacts_repo};

use super::super::agents::send_node_config_snapshot;
use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};
use super::validation::{validate_job_spec, validate_job_target_scope};
use bastion_engine::scheduler;

fn require_job_name(name: &str) -> Result<&str, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::bad_request(
            "invalid_name",
            "Job name is required",
        ));
    }
    Ok(name)
}

fn normalize_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn validate_schedule(schedule: Option<&str>) -> Result<(), AppError> {
    if let Some(schedule) = schedule {
        scheduler::validate_cron(schedule)
            .map_err(|_| AppError::bad_request("invalid_schedule", "Invalid cron schedule"))?;
    }
    Ok(())
}

fn normalize_timezone(value: Option<&str>, default: &str) -> Result<String, AppError> {
    let v = value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or(default)
        .trim();
    let _ = v
        .parse::<chrono_tz::Tz>()
        .map_err(|_| AppError::bad_request("invalid_timezone", "Invalid schedule timezone"))?;
    Ok(v.to_string())
}

async fn validate_agent_id(db: &sqlx::SqlitePool, agent_id: Option<&str>) -> Result<(), AppError> {
    let Some(agent_id) = agent_id else {
        return Ok(());
    };

    let row = sqlx::query("SELECT revoked_at FROM agents WHERE id = ? LIMIT 1")
        .bind(agent_id)
        .fetch_optional(db)
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

    Ok(())
}

async fn try_send_agent_config_snapshot(state: &AppState, agent_id: &str) {
    if let Err(error) = send_node_config_snapshot(
        &state.db,
        state.secrets.as_ref(),
        &state.agent_manager,
        agent_id,
    )
    .await
    {
        tracing::warn!(agent_id = %agent_id, error = %error, "failed to send agent config snapshot");
    }
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct CreateJobRequest {
    name: String,
    agent_id: Option<String>,
    schedule: Option<String>,
    schedule_timezone: Option<String>,
    overlap_policy: jobs_repo::OverlapPolicy,
    spec: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct UpdateJobRequest {
    name: String,
    agent_id: Option<String>,
    schedule: Option<String>,
    schedule_timezone: Option<String>,
    overlap_policy: jobs_repo::OverlapPolicy,
    spec: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct JobListItem {
    id: String,
    name: String,
    agent_id: Option<String>,
    schedule: Option<String>,
    schedule_timezone: String,
    overlap_policy: jobs_repo::OverlapPolicy,
    created_at: i64,
    updated_at: i64,
    archived_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct ListJobsQuery {
    include_archived: Option<bool>,
}

pub(in crate::http) async fn list_jobs(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Query(q): Query<ListJobsQuery>,
) -> Result<Json<Vec<JobListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let include_archived = q.include_archived.unwrap_or(false);
    let jobs = if include_archived {
        jobs_repo::list_jobs_including_archived(&state.db).await?
    } else {
        jobs_repo::list_jobs(&state.db).await?
    };

    Ok(Json(
        jobs.into_iter()
            .map(|j| JobListItem {
                id: j.id,
                name: j.name,
                agent_id: j.agent_id,
                schedule: j.schedule,
                schedule_timezone: j.schedule_timezone,
                overlap_policy: j.overlap_policy,
                created_at: j.created_at,
                updated_at: j.updated_at,
                archived_at: j.archived_at,
            })
            .collect(),
    ))
}

pub(in crate::http) async fn create_job(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Json(mut req): Json<CreateJobRequest>,
) -> Result<Json<jobs_repo::Job>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let name = require_job_name(&req.name)?;

    let schedule = normalize_optional_string(req.schedule.as_deref());
    let schedule_timezone =
        normalize_timezone(req.schedule_timezone.as_deref(), &state.config.hub_timezone)?;

    let agent_id = normalize_optional_string(req.agent_id.as_deref());
    validate_agent_id(&state.db, agent_id.as_deref()).await?;

    validate_job_spec(&req.spec)?;
    validate_job_target_scope(&state.db, agent_id.as_deref(), &req.spec).await?;
    validate_schedule(schedule.as_deref())?;

    // New jobs inherit the Hub default retention, unless explicitly set by the request.
    if let Some(spec) = req.spec.as_object_mut()
        && !spec.contains_key("retention")
    {
        let saved = hub_runtime_config_repo::get(&state.db)
            .await?
            .unwrap_or_default();

        if saved.default_backup_retention.enabled {
            spec.insert(
                "retention".to_string(),
                serde_json::to_value(saved.default_backup_retention)
                    .map_err(|e| anyhow::anyhow!("invalid default retention: {e}"))?,
            );
        }
    }

    let job = jobs_repo::create_job(
        &state.db,
        name,
        agent_id.as_deref(),
        schedule.as_deref(),
        Some(&schedule_timezone),
        req.overlap_policy,
        req.spec,
    )
    .await?;

    tracing::info!(
        job_id = %job.id,
        name = %job.name,
        agent_id = ?job.agent_id,
        schedule = ?job.schedule,
        schedule_timezone = %job.schedule_timezone,
        overlap_policy = ?job.overlap_policy,
        "job created"
    );
    state.jobs_notify.notify_one();

    if let Some(agent_id) = job.agent_id.as_deref() {
        try_send_agent_config_snapshot(&state, agent_id).await;
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

    let name = require_job_name(&req.name)?;

    let schedule = normalize_optional_string(req.schedule.as_deref());
    let schedule_timezone =
        normalize_timezone(req.schedule_timezone.as_deref(), &state.config.hub_timezone)?;

    let agent_id = normalize_optional_string(req.agent_id.as_deref());
    validate_agent_id(&state.db, agent_id.as_deref()).await?;

    validate_job_spec(&req.spec)?;
    validate_job_target_scope(&state.db, agent_id.as_deref(), &req.spec).await?;
    validate_schedule(schedule.as_deref())?;

    let updated = jobs_repo::update_job(
        &state.db,
        jobs_repo::UpdateJobParams {
            job_id: &job_id,
            name,
            agent_id: agent_id.as_deref(),
            schedule: schedule.as_deref(),
            schedule_timezone: Some(&schedule_timezone),
            overlap_policy: req.overlap_policy,
            spec: req.spec,
        },
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
        schedule_timezone = %job.schedule_timezone,
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
        try_send_agent_config_snapshot(&state, &agent_id).await;
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

    if let Some(agent_id) = previous_agent_id.as_deref() {
        try_send_agent_config_snapshot(&state, agent_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct ArchiveJobQuery {
    #[serde(default)]
    cascade_snapshots: bool,
}

pub(in crate::http) async fn archive_job(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Query(query): Query<ArchiveJobQuery>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let job = jobs_repo::get_job(&state.db, &job_id)
        .await?
        .ok_or_else(|| AppError::not_found("job_not_found", "Job not found"))?;

    let ok = jobs_repo::archive_job(&state.db, &job_id).await?;
    if ok {
        tracing::info!(job_id = %job_id, "job archived");
        state.jobs_notify.notify_one();
        if let Some(agent_id) = job.agent_id.as_deref() {
            try_send_agent_config_snapshot(&state, agent_id).await;
        }

        if query.cascade_snapshots {
            let now = time::OffsetDateTime::now_utc().unix_timestamp();
            if let Err(error) =
                cascade_enqueue_snapshot_deletes(&state, session.user_id, &job_id, now).await
            {
                tracing::warn!(
                    job_id = %job_id,
                    error = %error,
                    "job archived but snapshot cascade enqueue failed"
                );
            }
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn cascade_enqueue_snapshot_deletes(
    state: &AppState,
    user_id: i64,
    job_id: &str,
    now: i64,
) -> Result<(), anyhow::Error> {
    const PAGE_LIMIT: u64 = 200;
    const MAX_ENQUEUE: usize = 1000;

    let mut cursor = 0_u64;
    let mut enqueued = 0_usize;
    let mut skipped_pinned = 0_usize;

    while enqueued < MAX_ENQUEUE {
        let items = run_artifacts_repo::list_run_artifacts_for_job(
            &state.db,
            job_id,
            cursor,
            PAGE_LIMIT,
            Some("present"),
        )
        .await?;

        if items.is_empty() {
            break;
        }

        cursor = cursor.saturating_add(items.len() as u64);

        for artifact in items {
            if artifact.pinned_at.is_some() {
                skipped_pinned = skipped_pinned.saturating_add(1);
                continue;
            }

            let snapshot_json = serde_json::to_string(&artifact.target_snapshot)?;
            let inserted = artifact_delete_repo::upsert_task_if_missing(
                &state.db,
                &artifact.run_id,
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
                    &artifact.run_id,
                    "info",
                    "queued",
                    "delete queued (job archive cascade)",
                    Some(
                        serde_json::json!({ "user_id": user_id, "reason": "job_archive_cascade" }),
                    ),
                    now,
                )
                .await;
            }

            let _ =
                run_artifacts_repo::mark_run_artifact_deleting(&state.db, &artifact.run_id, now)
                    .await;
            enqueued = enqueued.saturating_add(1);
            if enqueued >= MAX_ENQUEUE {
                break;
            }
        }
    }

    if skipped_pinned > 0 {
        tracing::info!(
            job_id = %job_id,
            skipped_pinned,
            "job archive cascade skipped pinned snapshots"
        );
    }

    if enqueued > 0 {
        state.artifact_delete_notify.notify_one();
    }

    Ok(())
}

pub(in crate::http) async fn unarchive_job(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let job = jobs_repo::get_job(&state.db, &job_id)
        .await?
        .ok_or_else(|| AppError::not_found("job_not_found", "Job not found"))?;

    let ok = jobs_repo::unarchive_job(&state.db, &job_id).await?;
    if ok {
        tracing::info!(job_id = %job_id, "job unarchived");
        state.jobs_notify.notify_one();
        if let Some(agent_id) = job.agent_id.as_deref() {
            try_send_agent_config_snapshot(&state, agent_id).await;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
