use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use serde::Deserialize;
use sqlx::Row;

use bastion_core::job_spec;
use bastion_engine::notifications;
use bastion_storage::runs_repo;

use super::super::{AppError, AppState};
use super::agent_auth::authenticate_agent;

#[derive(Debug, Deserialize)]
pub(in crate::http) struct AgentIngestRunRequest {
    run: AgentIngestRun,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AgentIngestRunStatus {
    Success,
    Failed,
    Rejected,
}

#[derive(Debug, Deserialize)]
struct AgentIngestRun {
    id: String,
    job_id: String,
    status: AgentIngestRunStatus,
    started_at: i64,
    ended_at: i64,
    #[serde(default)]
    summary: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    events: Vec<AgentIngestRunEvent>,
}

#[derive(Debug, Deserialize)]
struct AgentIngestRunEvent {
    seq: i64,
    ts: i64,
    level: String,
    kind: String,
    message: String,
    #[serde(default)]
    fields: Option<serde_json::Value>,
}

pub(in crate::http) async fn agent_ingest_runs(
    state: axum::extract::State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AgentIngestRunRequest>,
) -> Result<StatusCode, AppError> {
    const MAX_EVENTS_PER_RUN: usize = 2000;
    const MAX_ID_LEN: usize = 128;
    const MAX_EVENT_LEVEL_LEN: usize = 16;
    const MAX_EVENT_KIND_LEN: usize = 64;
    const MAX_EVENT_MESSAGE_LEN: usize = 4096;
    const MAX_ERROR_LEN: usize = 4096;

    let agent_id = authenticate_agent(&state.db, &headers).await?;

    let run = req.run;
    if run.id.trim().is_empty() || run.id.len() > MAX_ID_LEN {
        return Err(AppError::bad_request(
            "invalid_run_id",
            "Run id is required",
        ));
    }
    if run.job_id.trim().is_empty() || run.job_id.len() > MAX_ID_LEN {
        return Err(AppError::bad_request(
            "invalid_job_id",
            "Job id is required",
        ));
    }
    if run.events.len() > MAX_EVENTS_PER_RUN {
        return Err(AppError::bad_request(
            "too_many_events",
            "Too many run events",
        ));
    }
    if run.started_at < 0 {
        return Err(AppError::bad_request(
            "invalid_started_at",
            "started_at is invalid",
        ));
    }
    if run.ended_at < run.started_at {
        return Err(AppError::bad_request(
            "invalid_ended_at",
            "ended_at is invalid",
        ));
    }
    if run.error.as_ref().is_some_and(|v| v.len() > MAX_ERROR_LEN) {
        return Err(AppError::bad_request("invalid_error", "error is too long"));
    }
    for ev in &run.events {
        if ev.seq <= 0 {
            continue;
        }
        let level = ev.level.trim();
        let kind = ev.kind.trim();
        let message = ev.message.trim();
        if level.is_empty()
            || kind.is_empty()
            || message.is_empty()
            || level.len() > MAX_EVENT_LEVEL_LEN
            || kind.len() > MAX_EVENT_KIND_LEN
            || message.len() > MAX_EVENT_MESSAGE_LEN
        {
            return Err(AppError::bad_request("invalid_event", "Invalid run event"));
        }
    }

    let row = sqlx::query("SELECT agent_id, spec_json FROM jobs WHERE id = ? LIMIT 1")
        .bind(&run.job_id)
        .fetch_optional(&state.db)
        .await?;

    let Some(row) = row else {
        return Err(AppError::bad_request("invalid_job_id", "Job not found"));
    };
    let job_agent_id = row.get::<Option<String>, _>("agent_id");
    if job_agent_id.as_deref() != Some(agent_id.as_str()) {
        return Err(AppError::bad_request(
            "invalid_job_id",
            "Job is not assigned to this Agent",
        ));
    }
    let spec_json = row.get::<String, _>("spec_json");

    if let Some(row) = sqlx::query("SELECT job_id FROM runs WHERE id = ? LIMIT 1")
        .bind(&run.id)
        .fetch_optional(&state.db)
        .await?
        && row.get::<String, _>("job_id") != run.job_id
    {
        return Err(AppError::bad_request(
            "invalid_run_id",
            "Run id is already associated with a different job",
        ));
    }

    let status = match run.status {
        AgentIngestRunStatus::Success => runs_repo::RunStatus::Success,
        AgentIngestRunStatus::Failed => runs_repo::RunStatus::Failed,
        AgentIngestRunStatus::Rejected => runs_repo::RunStatus::Rejected,
    };

    let summary_json = run
        .summary
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;

    let mut inserted_events = Vec::new();
    let mut tx = state.db.begin().await?;

    let _ = sqlx::query(
        r#"
        INSERT INTO runs (id, job_id, status, started_at, ended_at, summary_json, error)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          status = excluded.status,
          started_at = excluded.started_at,
          ended_at = excluded.ended_at,
          summary_json = excluded.summary_json,
          error = excluded.error
        "#,
    )
    .bind(&run.id)
    .bind(&run.job_id)
    .bind(status.as_str())
    .bind(run.started_at)
    .bind(run.ended_at)
    .bind(summary_json)
    .bind(run.error.as_deref())
    .execute(&mut *tx)
    .await?;

    for ev in &run.events {
        if ev.seq <= 0 {
            continue;
        }
        let fields_json = ev.fields.as_ref().map(serde_json::to_string).transpose()?;
        let result = sqlx::query(
            "INSERT OR IGNORE INTO run_events (run_id, seq, ts, level, kind, message, fields_json) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&run.id)
        .bind(ev.seq)
        .bind(ev.ts)
        .bind(ev.level.trim())
        .bind(ev.kind.trim())
        .bind(ev.message.trim())
        .bind(fields_json)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() > 0 {
            inserted_events.push(runs_repo::RunEvent {
                run_id: run.id.clone(),
                seq: ev.seq,
                ts: ev.ts,
                level: ev.level.clone(),
                kind: ev.kind.clone(),
                message: ev.message.clone(),
                fields: ev.fields.clone(),
            });
        }
    }

    tx.commit().await?;

    for ev in &inserted_events {
        state.run_events_bus.publish(ev);
    }

    // Enqueue notifications after ingestion (may be delayed while offline).
    if let Ok(spec_value) = serde_json::from_str::<serde_json::Value>(&spec_json)
        && let Ok(spec) = job_spec::parse_value(&spec_value)
        && job_spec::validate(&spec).is_ok()
    {
        match notifications::enqueue_for_run_spec(&state.db, &spec, &run.id).await {
            Ok(true) => state.notifications_notify.notify_one(),
            Ok(false) => {}
            Err(error) => {
                tracing::warn!(run_id = %run.id, error = %error, "failed to enqueue notifications for ingested run");
            }
        }
    }

    tracing::info!(
        agent_id = %agent_id,
        run_id = %run.id,
        job_id = %run.job_id,
        status = ?status,
        events = run.events.len(),
        inserted_events = inserted_events.len(),
        "agent run ingested"
    );
    Ok(StatusCode::NO_CONTENT)
}
