use std::sync::Arc;

use axum::Json;
use axum::extract::ConnectInfo;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use bastion_core::HUB_NODE_ID;
use bastion_core::job_spec;
use bastion_storage::jobs_repo;
use bastion_storage::runs_repo;
use bastion_storage::secrets_repo;

use super::shared::{require_csrf, require_session};
use super::{AppError, AppState};
use bastion_engine::run_events;
use bastion_engine::run_events_bus::RunEventsBus;
use bastion_engine::scheduler;

use super::agents::send_node_config_snapshot;

fn validate_job_spec(spec: &serde_json::Value) -> Result<(), AppError> {
    job_spec::validate_value(spec).map_err(|error| {
        AppError::bad_request("invalid_spec", format!("Invalid job spec: {error}"))
    })
}

async fn validate_job_target_scope(
    db: &SqlitePool,
    agent_id: Option<&str>,
    spec: &serde_json::Value,
) -> Result<(), AppError> {
    let node_id = agent_id.unwrap_or(HUB_NODE_ID);

    let parsed = job_spec::parse_value(spec).map_err(|error| {
        AppError::bad_request("invalid_spec", format!("Invalid job spec: {error}"))
    })?;

    let target = match &parsed {
        job_spec::JobSpecV1::Filesystem { target, .. } => target,
        job_spec::JobSpecV1::Sqlite { target, .. } => target,
        job_spec::JobSpecV1::Vaultwarden { target, .. } => target,
    };

    if let job_spec::TargetV1::Webdav { secret_name, .. } = target {
        let secret_name = secret_name.trim();
        if secret_name.is_empty() {
            return Err(AppError::bad_request(
                "invalid_webdav_secret",
                "WebDAV credential name is required",
            ));
        }

        let exists = secrets_repo::secret_exists(db, node_id, "webdav", secret_name).await?;
        if !exists {
            return Err(AppError::bad_request(
                "invalid_webdav_secret",
                "WebDAV credential not found",
            )
            .with_details(serde_json::json!({ "field": "spec.target.secret_name" })));
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
pub(super) struct CreateJobRequest {
    name: String,
    agent_id: Option<String>,
    schedule: Option<String>,
    overlap_policy: jobs_repo::OverlapPolicy,
    spec: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(super) struct UpdateJobRequest {
    name: String,
    agent_id: Option<String>,
    schedule: Option<String>,
    overlap_policy: jobs_repo::OverlapPolicy,
    spec: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub(super) struct JobListItem {
    id: String,
    name: String,
    agent_id: Option<String>,
    schedule: Option<String>,
    overlap_policy: jobs_repo::OverlapPolicy,
    created_at: i64,
    updated_at: i64,
}

pub(super) async fn list_jobs(
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

pub(super) async fn create_job(
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

pub(super) async fn get_job(
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

pub(super) async fn update_job(
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

pub(super) async fn delete_job(
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

#[derive(Debug, Serialize)]
pub(super) struct TriggerRunResponse {
    run_id: String,
    status: runs_repo::RunStatus,
}

pub(super) async fn trigger_job_run(
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
pub(super) struct RunListItem {
    id: String,
    status: runs_repo::RunStatus,
    started_at: i64,
    ended_at: Option<i64>,
    error: Option<String>,
    executed_offline: bool,
}

pub(super) async fn list_job_runs(
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
            })
            .collect(),
    ))
}

pub(super) async fn list_run_events(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(run_id): Path<String>,
) -> Result<Json<Vec<runs_repo::RunEvent>>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let events = runs_repo::list_run_events(&state.db, &run_id, 500).await?;
    Ok(Json(events))
}

#[derive(Debug, Deserialize)]
pub(super) struct RunEventsWsQuery {
    #[serde(default, alias = "after_seq")]
    after: Option<i64>,
}

pub(super) async fn run_events_ws(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    Query(query): Query<RunEventsWsQuery>,
    Path(run_id): Path<String>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let _session = require_session(&state, &cookies).await?;
    require_ws_same_origin(&state, &headers, peer.ip())?;

    let run_exists = runs_repo::get_run(&state.db, &run_id).await?.is_some();
    if !run_exists {
        return Err(AppError::not_found("run_not_found", "Run not found"));
    }

    let after_seq = query.after.unwrap_or(0).max(0);
    let db = state.db.clone();
    let run_events_bus = state.run_events_bus.clone();
    Ok(ws.on_upgrade(move |socket| {
        handle_run_events_socket(db, run_id, after_seq, run_events_bus, socket)
    }))
}

fn require_ws_same_origin(
    state: &AppState,
    headers: &HeaderMap,
    peer_ip: std::net::IpAddr,
) -> Result<(), AppError> {
    let origin = headers
        .get(axum::http::header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::unauthorized("invalid_origin", "Invalid origin"))?;

    let expected_host = if super::shared::is_trusted_proxy(state, peer_ip) {
        headers
            .get("x-forwarded-host")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(|s| s.trim().to_string())
            .or_else(|| {
                headers
                    .get(axum::http::header::HOST)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
            })
    } else {
        headers
            .get(axum::http::header::HOST)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    }
    .ok_or_else(|| AppError::unauthorized("invalid_origin", "Invalid origin"))?;

    let expected_host = expected_host
        .split(':')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();

    let origin_host = match url::Url::parse(origin) {
        Ok(url) => url.host_str().unwrap_or("").to_ascii_lowercase(),
        Err(_) => return Err(AppError::unauthorized("invalid_origin", "Invalid origin")),
    };

    if origin_host != expected_host {
        return Err(AppError::unauthorized("invalid_origin", "Invalid origin"));
    }

    Ok(())
}

async fn handle_run_events_socket(
    db: SqlitePool,
    run_id: String,
    after_seq: i64,
    run_events_bus: Arc<RunEventsBus>,
    mut socket: WebSocket,
) {
    let mut last_seq = after_seq.max(0);
    let mut idle_after_end = 0u32;

    let mut rx = run_events_bus.subscribe(&run_id);

    // Catch up from SQLite after the requested sequence.
    loop {
        let events = match runs_repo::list_run_events_after_seq(&db, &run_id, last_seq, 200).await {
            Ok(v) => v,
            Err(_) => return,
        };
        if events.is_empty() {
            break;
        }
        idle_after_end = 0;
        for event in events {
            last_seq = last_seq.max(event.seq);
            let payload = match serde_json::to_string(&event) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if socket.send(Message::Text(payload.into())).await.is_err() {
                return;
            }
        }
    }

    let mut status_interval = tokio::time::interval(std::time::Duration::from_secs(3));
    status_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    status_interval.tick().await; // discard immediate tick

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
            ev = rx.recv() => {
                match ev {
                    Ok(event) => {
                        if event.seq <= last_seq {
                            continue;
                        }
                        idle_after_end = 0;
                        last_seq = event.seq;
                        let payload = match serde_json::to_string(&event) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        if socket.send(Message::Text(payload.into())).await.is_err() {
                            return;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        // The client fell behind; resync from SQLite after the last confirmed seq.
                        loop {
                            let events = match runs_repo::list_run_events_after_seq(&db, &run_id, last_seq, 200).await {
                                Ok(v) => v,
                                Err(_) => return,
                            };
                            if events.is_empty() {
                                break;
                            }
                            idle_after_end = 0;
                            for event in events {
                                last_seq = last_seq.max(event.seq);
                                let payload = match serde_json::to_string(&event) {
                                    Ok(s) => s,
                                    Err(_) => continue,
                                };
                                if socket.send(Message::Text(payload.into())).await.is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            _ = status_interval.tick() => {
                match runs_repo::get_run(&db, &run_id).await {
                    Ok(Some(run)) => {
                        let ended = !matches!(run.status, runs_repo::RunStatus::Queued | runs_repo::RunStatus::Running);
                        if ended {
                            idle_after_end += 1;
                            if idle_after_end >= 10 {
                                break;
                            }
                        } else {
                            idle_after_end = 0;
                        }
                    }
                    Ok(None) | Err(_) => break,
                }
            }
        }
    }
}
