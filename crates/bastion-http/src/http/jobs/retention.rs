use std::collections::HashMap;

use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tower_cookies::Cookies;

use bastion_core::backup_retention::{RetentionSnapshot, select_retention};
use bastion_core::job_spec;
use bastion_storage::artifact_delete_repo;
use bastion_storage::jobs_repo;
use bastion_storage::run_artifacts_repo;

use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};

const RETENTION_SCAN_LIMIT: u64 = 20_000;
const PREVIEW_KEEP_LIMIT: usize = 100;
const PREVIEW_DELETE_LIMIT: usize = 200;

pub(in crate::http) async fn get_job_retention(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(job_id): Path<String>,
) -> Result<Json<job_spec::RetentionPolicyV1>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let job = jobs_repo::get_job(&state.db, &job_id)
        .await?
        .ok_or_else(|| AppError::not_found("job_not_found", "Job not found"))?;

    let parsed =
        job_spec::parse_value(&job.spec).map_err(|e| anyhow::anyhow!("invalid job spec: {e}"))?;

    Ok(Json(parsed.retention().clone()))
}

pub(in crate::http) async fn put_job_retention(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Json(retention): Json<job_spec::RetentionPolicyV1>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let job = jobs_repo::get_job(&state.db, &job_id)
        .await?
        .ok_or_else(|| AppError::not_found("job_not_found", "Job not found"))?;

    let mut spec = job.spec.clone();
    let map = spec
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("invalid job spec: not an object"))?;
    map.insert(
        "retention".to_string(),
        serde_json::to_value(&retention)
            .map_err(|_| AppError::bad_request("invalid_retention", "Invalid retention"))?,
    );

    // Validate full spec after mutation (retention rules are part of the job spec contract).
    job_spec::validate_value(&spec)
        .map_err(|e| AppError::bad_request("invalid_spec", format!("Invalid job spec: {e}")))?;

    let ok = jobs_repo::update_job(
        &state.db,
        jobs_repo::UpdateJobParams {
            job_id: &job_id,
            name: &job.name,
            agent_id: job.agent_id.as_deref(),
            schedule: job.schedule.as_deref(),
            schedule_timezone: Some(&job.schedule_timezone),
            overlap_policy: job.overlap_policy,
            spec,
        },
    )
    .await?;

    if !ok {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize, Default)]
pub(in crate::http) struct RetentionPreviewRequest {
    #[serde(default)]
    retention: Option<job_spec::RetentionPolicyV1>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct RetentionPreviewItem {
    run_id: String,
    ended_at: i64,
    pinned: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transfer_bytes: Option<u64>,
    reasons: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct RetentionPreviewResponse {
    retention: job_spec::RetentionPolicyV1,
    keep_total: u64,
    delete_total: u64,
    keep: Vec<RetentionPreviewItem>,
    delete: Vec<RetentionPreviewItem>,
    #[serde(default)]
    scan_truncated: bool,
    #[serde(default)]
    result_truncated: bool,
}

fn day_start_utc(ts: i64) -> i64 {
    ts.saturating_div(24 * 60 * 60).saturating_mul(24 * 60 * 60)
}

async fn load_retention_job(
    state: &AppState,
    job_id: &str,
) -> Result<(jobs_repo::Job, job_spec::RetentionPolicyV1), AppError> {
    let job = jobs_repo::get_job(&state.db, job_id)
        .await?
        .ok_or_else(|| AppError::not_found("job_not_found", "Job not found"))?;

    let parsed =
        job_spec::parse_value(&job.spec).map_err(|e| anyhow::anyhow!("invalid job spec: {e}"))?;

    Ok((job, parsed.retention().clone()))
}

fn validate_retention_override(
    job: &jobs_repo::Job,
    retention: &job_spec::RetentionPolicyV1,
) -> Result<(), AppError> {
    let mut spec = job.spec.clone();
    let map = spec
        .as_object_mut()
        .ok_or_else(|| AppError::bad_request("invalid_spec", "Invalid job spec"))?;
    map.insert(
        "retention".to_string(),
        serde_json::to_value(retention)
            .map_err(|_| AppError::bad_request("invalid_retention", "Invalid retention"))?,
    );

    job_spec::validate_value(&spec).map_err(|e| {
        AppError::bad_request("invalid_retention", format!("Invalid retention: {e}"))
    })?;
    Ok(())
}

async fn compute_preview(
    state: &AppState,
    job_id: &str,
    retention: job_spec::RetentionPolicyV1,
    now: i64,
) -> Result<RetentionPreviewResponse, AppError> {
    let mut rows = run_artifacts_repo::list_retention_items_for_job(
        &state.db,
        job_id,
        RETENTION_SCAN_LIMIT.saturating_add(1),
    )
    .await?;

    let scan_truncated = rows.len() as u64 > RETENTION_SCAN_LIMIT;
    if scan_truncated {
        rows.truncate(RETENTION_SCAN_LIMIT as usize);
    }

    let snapshots = rows
        .iter()
        .map(|r| RetentionSnapshot {
            run_id: r.run_id.clone(),
            ended_at: r.ended_at,
            pinned: r.pinned_at.is_some(),
        })
        .collect::<Vec<_>>();

    let selection = select_retention(&retention, now, &snapshots);

    let mut map: HashMap<&str, &run_artifacts_repo::RunArtifactRetentionItem> = HashMap::new();
    for r in &rows {
        map.insert(r.run_id.as_str(), r);
    }

    let mut keep_items = Vec::new();
    for d in &selection.keep {
        if keep_items.len() >= PREVIEW_KEEP_LIMIT {
            break;
        }
        if let Some(row) = map.get(d.run_id.as_str()) {
            keep_items.push(RetentionPreviewItem {
                run_id: d.run_id.clone(),
                ended_at: d.ended_at,
                pinned: row.pinned_at.is_some(),
                source_bytes: row.source_bytes,
                transfer_bytes: row.transfer_bytes,
                reasons: d.reasons.iter().map(|s| (*s).to_string()).collect(),
            });
        }
    }

    let mut delete_items = Vec::new();
    for d in &selection.delete {
        if delete_items.len() >= PREVIEW_DELETE_LIMIT {
            break;
        }
        if let Some(row) = map.get(d.run_id.as_str()) {
            delete_items.push(RetentionPreviewItem {
                run_id: d.run_id.clone(),
                ended_at: d.ended_at,
                pinned: row.pinned_at.is_some(),
                source_bytes: row.source_bytes,
                transfer_bytes: row.transfer_bytes,
                reasons: d.reasons.iter().map(|s| (*s).to_string()).collect(),
            });
        }
    }

    let keep_total = selection.keep.len() as u64;
    let delete_total = selection.delete.len() as u64;
    let result_truncated =
        keep_total as usize > keep_items.len() || delete_total as usize > delete_items.len();

    Ok(RetentionPreviewResponse {
        retention,
        keep_total,
        delete_total,
        keep: keep_items,
        delete: delete_items,
        scan_truncated,
        result_truncated,
    })
}

pub(in crate::http) async fn preview_job_retention(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(job_id): Path<String>,
    Json(req): Json<RetentionPreviewRequest>,
) -> Result<Json<RetentionPreviewResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let (job, saved) = load_retention_job(&state, &job_id).await?;
    let retention = req.retention.unwrap_or(saved);
    validate_retention_override(&job, &retention)?;
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let preview = compute_preview(&state, &job_id, retention, now).await?;
    Ok(Json(preview))
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct RetentionApplyResponse {
    enqueued: Vec<String>,
    #[serde(default)]
    already_exists: u64,
    #[serde(default)]
    skipped_due_to_limits: u64,
}

pub(in crate::http) async fn apply_job_retention(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    Json(req): Json<RetentionPreviewRequest>,
) -> Result<Json<RetentionApplyResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let (job, saved) = load_retention_job(&state, &job_id).await?;
    let retention = req.retention.unwrap_or(saved);
    validate_retention_override(&job, &retention)?;

    if !retention.enabled {
        return Err(AppError::bad_request(
            "retention_disabled",
            "Retention is disabled",
        ));
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();

    let rows =
        run_artifacts_repo::list_retention_items_for_job(&state.db, &job_id, RETENTION_SCAN_LIMIT)
            .await?;

    let snapshots = rows
        .iter()
        .map(|r| RetentionSnapshot {
            run_id: r.run_id.clone(),
            ended_at: r.ended_at,
            pinned: r.pinned_at.is_some(),
        })
        .collect::<Vec<_>>();

    let selection = select_retention(&retention, now, &snapshots);

    let day_start = day_start_utc(now);
    let already =
        artifact_delete_repo::count_retention_enqueues_for_job_since(&state.db, &job_id, day_start)
            .await?
            .min(u64::from(u32::MAX)) as u32;

    let remaining_day = retention.max_delete_per_day.saturating_sub(already);
    let allowed = std::cmp::min(retention.max_delete_per_tick, remaining_day) as usize;

    if allowed == 0 {
        return Ok(Json(RetentionApplyResponse {
            enqueued: Vec::new(),
            already_exists: 0,
            skipped_due_to_limits: selection.delete.len() as u64,
        }));
    }

    let mut enqueued = Vec::new();
    let mut already_exists = 0_u64;
    let mut skipped_due_to_limits = 0_u64;

    for (idx, d) in selection.delete.iter().enumerate() {
        if idx >= allowed {
            skipped_due_to_limits = (selection.delete.len().saturating_sub(allowed)) as u64;
            break;
        }

        let Some(artifact) = run_artifacts_repo::get_run_artifact(&state.db, &d.run_id).await?
        else {
            continue;
        };

        // Already gone -> idempotent no-op.
        if artifact.status == "deleted" || artifact.status == "missing" {
            continue;
        }

        let snapshot_json = serde_json::to_string(&artifact.target_snapshot)
            .map_err(|_| AppError::bad_request("invalid_snapshot", "Invalid target snapshot"))?;

        let inserted = artifact_delete_repo::upsert_task_if_missing(
            &state.db,
            &artifact.run_id,
            &artifact.job_id,
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
                "retention_queued",
                "retention delete queued",
                Some(serde_json::json!({
                    "job_id": job_id,
                    "keep_last": retention.keep_last,
                    "keep_days": retention.keep_days
                })),
                now,
            )
            .await;
            enqueued.push(artifact.run_id.clone());
        } else {
            already_exists = already_exists.saturating_add(1);
        }

        let _ =
            run_artifacts_repo::mark_run_artifact_deleting(&state.db, &artifact.run_id, now).await;
        state.artifact_delete_notify.notify_one();
    }

    Ok(Json(RetentionApplyResponse {
        enqueued,
        already_exists,
        skipped_due_to_limits,
    }))
}
