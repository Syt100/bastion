use axum::Json;
use axum::extract::Path;
use axum::http::HeaderMap;
use serde::Serialize;
use sqlx::Row;
use tower_cookies::Cookies;

use bastion_storage::jobs_repo;
use bastion_storage::runs_repo;

use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};
use bastion_engine::run_events;

#[derive(Debug, Serialize)]
pub(in crate::http) struct TriggerRunResponse {
    run_id: String,
    status: runs_repo::RunStatus,
}

pub(in crate::http) async fn trigger_job_run(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> Result<Json<TriggerRunResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let job = jobs_repo::get_job(&state.db, &job_id)
        .await?
        .ok_or_else(|| AppError::not_found("job_not_found", "Job not found"))?;

    let running_count = sqlx::query(
        "SELECT COUNT(1) AS n FROM runs WHERE job_id = ? AND status IN ('running', 'queued')",
    )
    .bind(&job.id)
    .fetch_one(&state.db)
    .await?
    .get::<i64, _>("n");

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let (status, ended_at, error) =
        if job.overlap_policy == jobs_repo::OverlapPolicy::Reject && running_count > 0 {
            (
                runs_repo::RunStatus::Rejected,
                Some(now),
                Some("overlap_rejected"),
            )
        } else {
            (runs_repo::RunStatus::Queued, None, None)
        };

    let run = runs_repo::create_run(&state.db, &job.id, status, now, ended_at, None, error).await?;

    let event_kind = match status {
        runs_repo::RunStatus::Rejected => "rejected",
        runs_repo::RunStatus::Queued => "queued",
        _ => "unknown",
    };
    run_events::append_and_broadcast(
        &state.db,
        &state.run_events_bus,
        &run.id,
        "info",
        event_kind,
        event_kind,
        Some(serde_json::json!({ "source": "manual" })),
    )
    .await?;

    if status == runs_repo::RunStatus::Queued {
        state.run_queue_notify.notify_one();
    }

    tracing::info!(
        job_id = %job.id,
        run_id = %run.id,
        status = ?run.status,
        "manual run triggered"
    );
    Ok(Json(TriggerRunResponse {
        run_id: run.id,
        status: run.status,
    }))
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct RunListItem {
    id: String,
    status: runs_repo::RunStatus,
    started_at: i64,
    ended_at: Option<i64>,
    error: Option<String>,
    executed_offline: bool,
    consistency_changed_total: u64,
}

fn consistency_changed_total_from_summary(summary: Option<&serde_json::Value>) -> u64 {
    let Some(summary) = summary else {
        return 0;
    };

    fn report_total(report: &serde_json::Value) -> u64 {
        let Some(obj) = report.as_object() else {
            return 0;
        };

        let changed_total = obj
            .get("changed_total")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let replaced_total = obj
            .get("replaced_total")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let deleted_total = obj
            .get("deleted_total")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let read_error_total = obj
            .get("read_error_total")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        changed_total
            .saturating_add(replaced_total)
            .saturating_add(deleted_total)
            .saturating_add(read_error_total)
    }

    let fs_total = summary
        .get("filesystem")
        .and_then(|v| v.get("consistency"))
        .map(report_total)
        .unwrap_or(0);
    let vw_total = summary
        .get("vaultwarden")
        .and_then(|v| v.get("consistency"))
        .map(report_total)
        .unwrap_or(0);

    fs_total.saturating_add(vw_total)
}

pub(in crate::http) async fn list_job_runs(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(job_id): Path<String>,
) -> Result<Json<Vec<RunListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let job_exists = jobs_repo::get_job(&state.db, &job_id).await?.is_some();
    if !job_exists {
        return Err(AppError::not_found("job_not_found", "Job not found"));
    }

    let runs = runs_repo::list_runs_for_job(&state.db, &job_id, 50).await?;
    Ok(Json(
        runs.into_iter()
            .map(|r| RunListItem {
                id: r.id,
                status: r.status,
                started_at: r.started_at,
                ended_at: r.ended_at,
                error: r.error,
                executed_offline: r
                    .summary
                    .as_ref()
                    .and_then(|v| v.get("executed_offline"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                consistency_changed_total: consistency_changed_total_from_summary(r.summary.as_ref()),
            })
            .collect(),
    ))
}

pub(in crate::http) async fn list_run_events(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(run_id): Path<String>,
) -> Result<Json<Vec<runs_repo::RunEvent>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let events = runs_repo::list_run_events(&state.db, &run_id, 500).await?;
    Ok(Json(events))
}
