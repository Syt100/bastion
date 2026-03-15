use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;
use sqlx::Row;
use tower_cookies::Cookies;

use super::shared::require_session;
use super::{AppError, AppState};
use bastion_storage::agent_labels_repo;

#[derive(Debug, Serialize)]
struct FleetSummary {
    total: i64,
    online: i64,
    offline: i64,
    revoked: i64,
    drifted: i64,
}

#[derive(Debug, Serialize)]
struct FleetOnboarding {
    public_base_url: Option<String>,
    command_generation_ready: bool,
}

#[derive(Debug, Serialize)]
struct FleetConfigSyncSummary {
    state: String,
    last_error_kind: Option<String>,
    last_error: Option<String>,
}

#[derive(Debug, Serialize)]
struct FleetListItem {
    id: String,
    name: Option<String>,
    status: String,
    last_seen_at: Option<i64>,
    labels: Vec<String>,
    config_sync: FleetConfigSyncSummary,
    assigned_jobs_total: i64,
    pending_tasks_total: i64,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct FleetListResponse {
    summary: FleetSummary,
    onboarding: FleetOnboarding,
    items: Vec<FleetListItem>,
}

#[derive(Debug, Serialize)]
struct FleetAgentDetailSummary {
    id: String,
    name: Option<String>,
    status: String,
    created_at: i64,
    last_seen_at: Option<i64>,
    labels: Vec<String>,
}

#[derive(Debug, Serialize)]
struct FleetAgentSync {
    desired_snapshot_id: Option<String>,
    desired_snapshot_at: Option<i64>,
    applied_snapshot_id: Option<String>,
    applied_snapshot_at: Option<i64>,
    state: String,
    last_error_kind: Option<String>,
    last_error: Option<String>,
    last_attempt_at: Option<i64>,
}

#[derive(Debug, Serialize)]
struct FleetActivityItem {
    run_id: String,
    job_id: String,
    job_name: String,
    status: String,
    started_at: Option<i64>,
    ended_at: Option<i64>,
}

#[derive(Debug, Serialize)]
struct FleetRelatedJob {
    id: String,
    name: String,
    schedule: Option<String>,
    updated_at: i64,
}

#[derive(Debug, Serialize)]
struct FleetCapabilities {
    can_rotate_key: bool,
    can_revoke: bool,
    can_sync_now: bool,
    can_manage_storage: bool,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct FleetAgentDetailResponse {
    agent: FleetAgentDetailSummary,
    sync: FleetAgentSync,
    recent_activity: Vec<FleetActivityItem>,
    related_jobs: Vec<FleetRelatedJob>,
    capabilities: FleetCapabilities,
}

fn agent_online(revoked: bool, last_seen_at: Option<i64>, now: i64) -> bool {
    if revoked {
        return false;
    }

    let cutoff = now.saturating_sub(60);
    last_seen_at.is_some_and(|ts| ts >= cutoff)
}

fn config_sync_state(
    online: bool,
    desired_snapshot_id: Option<&str>,
    applied_snapshot_id: Option<&str>,
    last_error_kind: Option<&str>,
) -> &'static str {
    if !online {
        return "offline";
    }
    if last_error_kind.is_some() {
        return "error";
    }
    if let Some(desired_snapshot_id) = desired_snapshot_id
        && applied_snapshot_id == Some(desired_snapshot_id)
    {
        return "synced";
    }
    "pending"
}

fn fleet_status(revoked: bool, online: bool) -> &'static str {
    if revoked {
        "revoked"
    } else if online {
        "online"
    } else {
        "offline"
    }
}

pub(in crate::http) async fn list_fleet(
    state: State<AppState>,
    cookies: Cookies,
) -> Result<Json<FleetListResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let rows = sqlx::query(
        r#"
        SELECT
          a.id,
          a.name,
          a.created_at,
          a.revoked_at,
          a.last_seen_at,
          a.desired_config_snapshot_id,
          a.applied_config_snapshot_id,
          a.last_config_sync_error_kind,
          a.last_config_sync_error,
          COALESCE(job_counts.total, 0) AS assigned_jobs_total,
          COALESCE(task_counts.total, 0) AS pending_tasks_total,
          al.label
        FROM agents a
        LEFT JOIN agent_labels al ON al.agent_id = a.id
        LEFT JOIN (
          SELECT agent_id, COUNT(*) AS total
          FROM jobs
          WHERE archived_at IS NULL AND agent_id IS NOT NULL
          GROUP BY agent_id
        ) AS job_counts ON job_counts.agent_id = a.id
        LEFT JOIN (
          SELECT agent_id, COUNT(*) AS total
          FROM agent_tasks
          WHERE completed_at IS NULL
          GROUP BY agent_id
        ) AS task_counts ON task_counts.agent_id = a.id
        ORDER BY a.created_at DESC, a.id ASC, al.label ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut items = Vec::new();
    let mut current_id: Option<String> = None;
    let mut summary = FleetSummary {
        total: 0,
        online: 0,
        offline: 0,
        revoked: 0,
        drifted: 0,
    };

    for row in rows {
        let id = row.get::<String, _>("id");
        let is_new = current_id.as_deref() != Some(id.as_str());
        if is_new {
            let revoked = row.get::<Option<i64>, _>("revoked_at").is_some();
            let last_seen_at = row.get::<Option<i64>, _>("last_seen_at");
            let online = agent_online(revoked, last_seen_at, now);
            let desired_snapshot_id = row.get::<Option<String>, _>("desired_config_snapshot_id");
            let applied_snapshot_id = row.get::<Option<String>, _>("applied_config_snapshot_id");
            let last_error_kind = row.get::<Option<String>, _>("last_config_sync_error_kind");
            let last_error = row.get::<Option<String>, _>("last_config_sync_error");
            let sync_state = config_sync_state(
                online,
                desired_snapshot_id.as_deref(),
                applied_snapshot_id.as_deref(),
                last_error_kind.as_deref(),
            );

            summary.total += 1;
            match fleet_status(revoked, online) {
                "revoked" => summary.revoked += 1,
                "online" => summary.online += 1,
                _ => summary.offline += 1,
            }
            if sync_state == "pending" || sync_state == "error" {
                summary.drifted += 1;
            }

            items.push(FleetListItem {
                id: id.clone(),
                name: row.get::<Option<String>, _>("name"),
                status: fleet_status(revoked, online).to_string(),
                last_seen_at,
                labels: Vec::new(),
                config_sync: FleetConfigSyncSummary {
                    state: sync_state.to_string(),
                    last_error_kind,
                    last_error,
                },
                assigned_jobs_total: row.get::<i64, _>("assigned_jobs_total"),
                pending_tasks_total: row.get::<i64, _>("pending_tasks_total"),
            });
            current_id = Some(id);
        }

        if let Some(label) = row.get::<Option<String>, _>("label")
            && let Some(last) = items.last_mut()
        {
            last.labels.push(label);
        }
    }

    Ok(Json(FleetListResponse {
        summary,
        onboarding: FleetOnboarding {
            public_base_url: state.hub_runtime_config.public_base_url.clone(),
            command_generation_ready: state.hub_runtime_config.public_base_url.is_some(),
        },
        items,
    }))
}

pub(in crate::http) async fn get_fleet_agent(
    state: State<AppState>,
    cookies: Cookies,
    Path(agent_id): Path<String>,
) -> Result<Json<FleetAgentDetailResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let row = sqlx::query(
        r#"
        SELECT
          id,
          name,
          created_at,
          revoked_at,
          last_seen_at,
          desired_config_snapshot_id,
          desired_config_snapshot_at,
          applied_config_snapshot_id,
          applied_config_snapshot_at,
          last_config_sync_attempt_at,
          last_config_sync_error_kind,
          last_config_sync_error
        FROM agents
        WHERE id = ?
        LIMIT 1
        "#,
    )
    .bind(&agent_id)
    .fetch_optional(&state.db)
    .await?;

    let Some(row) = row else {
        return Err(AppError::not_found("agent_not_found", "Agent not found"));
    };

    let revoked = row.get::<Option<i64>, _>("revoked_at").is_some();
    let last_seen_at = row.get::<Option<i64>, _>("last_seen_at");
    let online = agent_online(revoked, last_seen_at, now);
    let desired_snapshot_id = row.get::<Option<String>, _>("desired_config_snapshot_id");
    let applied_snapshot_id = row.get::<Option<String>, _>("applied_config_snapshot_id");
    let last_error_kind = row.get::<Option<String>, _>("last_config_sync_error_kind");
    let last_error = row.get::<Option<String>, _>("last_config_sync_error");
    let labels = agent_labels_repo::list_labels_for_agent(&state.db, &agent_id).await?;

    let recent_activity_rows = sqlx::query(
        r#"
        SELECT r.id AS run_id, r.status, r.started_at, r.ended_at, j.id AS job_id, j.name AS job_name
        FROM runs r
        JOIN jobs j ON j.id = r.job_id
        WHERE j.agent_id = ?
        ORDER BY COALESCE(r.ended_at, r.started_at, 0) DESC, r.id DESC
        LIMIT 6
        "#,
    )
    .bind(&agent_id)
    .fetch_all(&state.db)
    .await?;

    let related_job_rows = sqlx::query(
        r#"
        SELECT id, name, schedule, updated_at
        FROM jobs
        WHERE agent_id = ? AND archived_at IS NULL
        ORDER BY updated_at DESC, id ASC
        LIMIT 8
        "#,
    )
    .bind(&agent_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(FleetAgentDetailResponse {
        agent: FleetAgentDetailSummary {
            id: row.get::<String, _>("id"),
            name: row.get::<Option<String>, _>("name"),
            status: fleet_status(revoked, online).to_string(),
            created_at: row.get::<i64, _>("created_at"),
            last_seen_at,
            labels,
        },
        sync: FleetAgentSync {
            desired_snapshot_id: desired_snapshot_id.clone(),
            desired_snapshot_at: row.get::<Option<i64>, _>("desired_config_snapshot_at"),
            applied_snapshot_id: applied_snapshot_id.clone(),
            applied_snapshot_at: row.get::<Option<i64>, _>("applied_config_snapshot_at"),
            state: config_sync_state(
                online,
                desired_snapshot_id.as_deref(),
                applied_snapshot_id.as_deref(),
                last_error_kind.as_deref(),
            )
            .to_string(),
            last_error_kind,
            last_error,
            last_attempt_at: row.get::<Option<i64>, _>("last_config_sync_attempt_at"),
        },
        recent_activity: recent_activity_rows
            .into_iter()
            .map(|row| FleetActivityItem {
                run_id: row.get::<String, _>("run_id"),
                job_id: row.get::<String, _>("job_id"),
                job_name: row.get::<String, _>("job_name"),
                status: row.get::<String, _>("status"),
                started_at: row.get::<Option<i64>, _>("started_at"),
                ended_at: row.get::<Option<i64>, _>("ended_at"),
            })
            .collect(),
        related_jobs: related_job_rows
            .into_iter()
            .map(|row| FleetRelatedJob {
                id: row.get::<String, _>("id"),
                name: row.get::<String, _>("name"),
                schedule: row.get::<Option<String>, _>("schedule"),
                updated_at: row.get::<i64, _>("updated_at"),
            })
            .collect(),
        capabilities: FleetCapabilities {
            can_rotate_key: !revoked,
            can_revoke: !revoked,
            can_sync_now: !revoked,
            can_manage_storage: true,
        },
    }))
}
