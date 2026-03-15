use axum::Json;
use axum::extract::{Path, RawQuery, State};
use serde::Serialize;
use sqlx::{QueryBuilder, Row};
use tower_cookies::Cookies;

use super::agents::{LabelsMode, normalize_labels, parse_labels_mode};
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
    last_attempt_at: Option<i64>,
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
    page: i64,
    page_size: i64,
    total: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FleetStatusFilter {
    All,
    Online,
    Offline,
    Revoked,
}

fn invalid_page_size_error(reason: &'static str, message: impl Into<String>) -> AppError {
    AppError::bad_request("invalid_page_size", message)
        .with_reason(reason)
        .with_field("page_size")
}

fn invalid_page_error(reason: &'static str, message: impl Into<String>) -> AppError {
    AppError::bad_request("invalid_page", message)
        .with_reason(reason)
        .with_field("page")
}

fn invalid_status_error(message: impl Into<String>) -> AppError {
    AppError::bad_request("invalid_status", message)
        .with_reason("unsupported_value")
        .with_field("status")
}

fn parse_status_filter(value: Option<&str>) -> Result<FleetStatusFilter, AppError> {
    let value = value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("all");

    match value {
        "all" => Ok(FleetStatusFilter::All),
        "online" => Ok(FleetStatusFilter::Online),
        "offline" => Ok(FleetStatusFilter::Offline),
        "revoked" => Ok(FleetStatusFilter::Revoked),
        _ => Err(invalid_status_error("Invalid status")),
    }
}

fn normalize_search_query(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn push_fleet_list_filters(
    qb: &mut QueryBuilder<sqlx::Sqlite>,
    labels: &[String],
    mode: LabelsMode,
    status: FleetStatusFilter,
    online_cutoff: i64,
    search: Option<&str>,
) {
    let mut has_where = false;
    let mut push_next = |qb: &mut QueryBuilder<sqlx::Sqlite>| {
        if has_where {
            qb.push(" AND ");
        } else {
            qb.push(" WHERE ");
            has_where = true;
        }
    };

    if !labels.is_empty() {
        push_next(qb);
        qb.push("a.id IN (");
        match mode {
            LabelsMode::And => {
                qb.push("SELECT al2.agent_id FROM agent_labels al2 WHERE al2.label IN (");
                let mut separated = qb.separated(", ");
                for label in labels {
                    separated.push_bind(label.clone());
                }
                separated.push_unseparated(")");
                qb.push(" GROUP BY al2.agent_id HAVING COUNT(DISTINCT al2.label) = ");
                qb.push_bind(labels.len() as i64);
            }
            LabelsMode::Or => {
                qb.push("SELECT DISTINCT al2.agent_id FROM agent_labels al2 WHERE al2.label IN (");
                let mut separated = qb.separated(", ");
                for label in labels {
                    separated.push_bind(label.clone());
                }
                separated.push_unseparated(")");
            }
        }
        qb.push(")");
    }

    match status {
        FleetStatusFilter::All => {}
        FleetStatusFilter::Revoked => {
            push_next(qb);
            qb.push("a.revoked_at IS NOT NULL");
        }
        FleetStatusFilter::Online => {
            push_next(qb);
            qb.push("a.revoked_at IS NULL AND a.last_seen_at IS NOT NULL AND a.last_seen_at >= ");
            qb.push_bind(online_cutoff);
        }
        FleetStatusFilter::Offline => {
            push_next(qb);
            qb.push("a.revoked_at IS NULL AND (a.last_seen_at IS NULL OR a.last_seen_at < ");
            qb.push_bind(online_cutoff);
            qb.push(")");
        }
    }

    if let Some(search) = search {
        let pattern = format!("%{}%", search.to_lowercase());
        push_next(qb);
        qb.push("(LOWER(COALESCE(a.name, '')) LIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR LOWER(a.id) LIKE ");
        qb.push_bind(pattern);
        qb.push(")");
    }
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
    RawQuery(raw): RawQuery,
) -> Result<Json<FleetListResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let online_cutoff = now.saturating_sub(60);

    let mut labels = Vec::new();
    let mut labels_mode: Option<String> = None;
    let mut status: Option<String> = None;
    let mut search: Option<String> = None;
    let mut page: Option<i64> = None;
    let mut page_size: Option<i64> = None;

    if let Some(raw) = raw {
        for (key, value) in url::form_urlencoded::parse(raw.as_bytes()) {
            match key.as_ref() {
                "labels" | "labels[]" => labels.push(value.into_owned()),
                "labels_mode" => labels_mode = Some(value.into_owned()),
                "status" => status = Some(value.into_owned()),
                "q" => search = Some(value.into_owned()),
                "page" => {
                    let parsed = value
                        .parse::<i64>()
                        .map_err(|_| invalid_page_error("invalid_format", "Invalid page"))?;
                    page = Some(parsed);
                }
                "page_size" => {
                    let parsed = value.parse::<i64>().map_err(|_| {
                        invalid_page_size_error("invalid_format", "Invalid page_size")
                    })?;
                    page_size = Some(parsed);
                }
                _ => {}
            }
        }
    }

    let labels = normalize_labels(labels)?;
    let labels_mode = parse_labels_mode(labels_mode.as_deref())?;
    let status = parse_status_filter(status.as_deref())?;
    let search = normalize_search_query(search);

    let mut summary_qb = QueryBuilder::new(
        "SELECT COUNT(*) AS total, COALESCE(SUM(CASE WHEN a.revoked_at IS NULL AND a.last_seen_at IS NOT NULL AND a.last_seen_at >= ",
    );
    summary_qb.push_bind(online_cutoff);
    summary_qb.push(
        " THEN 1 ELSE 0 END), 0) AS online, COALESCE(SUM(CASE WHEN a.revoked_at IS NULL AND (a.last_seen_at IS NULL OR a.last_seen_at < ",
    );
    summary_qb.push_bind(online_cutoff);
    summary_qb.push(
        ") THEN 1 ELSE 0 END), 0) AS offline, COALESCE(SUM(CASE WHEN a.revoked_at IS NOT NULL THEN 1 ELSE 0 END), 0) AS revoked, COALESCE(SUM(CASE WHEN a.revoked_at IS NULL AND a.last_seen_at IS NOT NULL AND a.last_seen_at >= ",
    );
    summary_qb.push_bind(online_cutoff);
    summary_qb.push(
        " AND (a.last_config_sync_error_kind IS NOT NULL OR a.desired_config_snapshot_id IS NULL OR COALESCE(a.applied_config_snapshot_id, '') != a.desired_config_snapshot_id) THEN 1 ELSE 0 END), 0) AS drifted FROM agents a",
    );
    push_fleet_list_filters(
        &mut summary_qb,
        &labels,
        labels_mode,
        status,
        online_cutoff,
        search.as_deref(),
    );
    let summary_row = summary_qb.build().fetch_one(&state.db).await?;
    let summary = FleetSummary {
        total: summary_row.get::<i64, _>("total"),
        online: summary_row.get::<i64, _>("online"),
        offline: summary_row.get::<i64, _>("offline"),
        revoked: summary_row.get::<i64, _>("revoked"),
        drifted: summary_row.get::<i64, _>("drifted"),
    };
    let total = summary.total;

    let pagination_requested = page.is_some() || page_size.is_some();
    let page = page.unwrap_or(1);
    if page < 1 {
        return Err(invalid_page_error("must_be_positive", "Invalid page").with_param("min", 1));
    }

    let page_size = if pagination_requested {
        let page_size = page_size.unwrap_or(20);
        if page_size < 1 {
            return Err(
                invalid_page_size_error("must_be_positive", "Invalid page_size")
                    .with_param("min", 1),
            );
        }
        page_size.clamp(1, 100)
    } else {
        summary.total
    };

    let mut ids_qb: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new("SELECT a.id FROM agents a");
    push_fleet_list_filters(
        &mut ids_qb,
        &labels,
        labels_mode,
        status,
        online_cutoff,
        search.as_deref(),
    );
    ids_qb.push(" ORDER BY a.created_at DESC, a.id ASC");

    if pagination_requested {
        let offset = (page - 1).saturating_mul(page_size);
        ids_qb.push(" LIMIT ");
        ids_qb.push_bind(page_size);
        ids_qb.push(" OFFSET ");
        ids_qb.push_bind(offset);
    }

    let id_rows = ids_qb.build().fetch_all(&state.db).await?;
    let ids: Vec<String> = id_rows
        .into_iter()
        .map(|row| row.get::<String, _>("id"))
        .collect();

    let mut items = Vec::new();
    if !ids.is_empty() {
        let mut rows_qb: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            r#"
            SELECT
              a.id,
              a.name,
              a.created_at,
              a.revoked_at,
              a.last_seen_at,
              a.desired_config_snapshot_id,
              a.applied_config_snapshot_id,
              a.last_config_sync_attempt_at,
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
            WHERE a.id IN (
            "#,
        );
        let mut separated = rows_qb.separated(", ");
        for id in &ids {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");
        rows_qb.push(" ORDER BY a.created_at DESC, a.id ASC, al.label ASC");

        let rows = rows_qb.build().fetch_all(&state.db).await?;
        let mut current_id: Option<String> = None;

        for row in rows {
            let id = row.get::<String, _>("id");
            let is_new = current_id.as_deref() != Some(id.as_str());
            if is_new {
                let revoked = row.get::<Option<i64>, _>("revoked_at").is_some();
                let last_seen_at = row.get::<Option<i64>, _>("last_seen_at");
                let online = agent_online(revoked, last_seen_at, now);
                let desired_snapshot_id =
                    row.get::<Option<String>, _>("desired_config_snapshot_id");
                let applied_snapshot_id =
                    row.get::<Option<String>, _>("applied_config_snapshot_id");
                let last_error_kind = row.get::<Option<String>, _>("last_config_sync_error_kind");
                let last_error = row.get::<Option<String>, _>("last_config_sync_error");
                let sync_state = config_sync_state(
                    online,
                    desired_snapshot_id.as_deref(),
                    applied_snapshot_id.as_deref(),
                    last_error_kind.as_deref(),
                );

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
                        last_attempt_at: row.get::<Option<i64>, _>("last_config_sync_attempt_at"),
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
    }

    Ok(Json(FleetListResponse {
        summary,
        onboarding: FleetOnboarding {
            public_base_url: state.hub_runtime_config.public_base_url.clone(),
            command_generation_ready: state.hub_runtime_config.public_base_url.is_some(),
        },
        items,
        page: if pagination_requested { page } else { 1 },
        page_size: if pagination_requested {
            page_size
        } else {
            total
        },
        total,
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
