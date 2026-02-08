use axum::Json;
use axum::extract::{Path, RawQuery};
use axum::http::{HeaderMap, StatusCode};
use serde::Serialize;
use sqlx::{QueryBuilder, Row};
use tower_cookies::Cookies;

use bastion_engine::agent_snapshots::{
    SendConfigSnapshotOutcome, send_node_config_snapshot_with_outcome,
};
use bastion_storage::agent_labels_repo;
use bastion_storage::agents_repo;

use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};
use super::labels::{LabelsMode, normalize_labels, parse_labels_mode};

#[derive(Debug, Serialize)]
pub(in crate::http) struct AgentListItem {
    id: String,
    name: Option<String>,
    revoked: bool,
    last_seen_at: Option<i64>,
    online: bool,
    labels: Vec<String>,
    desired_config_snapshot_id: Option<String>,
    applied_config_snapshot_id: Option<String>,
    config_sync_status: String,
    last_config_sync_attempt_at: Option<i64>,
    last_config_sync_error_kind: Option<String>,
    last_config_sync_error: Option<String>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct AgentListResponse {
    items: Vec<AgentListItem>,
    page: i64,
    page_size: i64,
    total: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentStatusFilter {
    All,
    Online,
    Offline,
    Revoked,
}

fn parse_status_filter(value: Option<&str>) -> Result<AgentStatusFilter, AppError> {
    let value = value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("all");

    match value {
        "all" => Ok(AgentStatusFilter::All),
        "online" => Ok(AgentStatusFilter::Online),
        "offline" => Ok(AgentStatusFilter::Offline),
        "revoked" => Ok(AgentStatusFilter::Revoked),
        _ => Err(AppError::bad_request("invalid_status", "Invalid status")),
    }
}

fn normalize_search_query(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn push_agent_list_filters(
    qb: &mut QueryBuilder<sqlx::Sqlite>,
    labels: &[String],
    mode: LabelsMode,
    status: AgentStatusFilter,
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
        AgentStatusFilter::All => {}
        AgentStatusFilter::Revoked => {
            push_next(qb);
            qb.push("a.revoked_at IS NOT NULL");
        }
        AgentStatusFilter::Online => {
            push_next(qb);
            qb.push("a.revoked_at IS NULL AND a.last_seen_at IS NOT NULL AND a.last_seen_at >= ");
            qb.push_bind(online_cutoff);
        }
        AgentStatusFilter::Offline => {
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

fn agent_online(revoked: bool, last_seen_at: Option<i64>, now: i64) -> bool {
    if revoked {
        return false;
    }

    let cutoff = now.saturating_sub(60);
    last_seen_at.is_some_and(|ts| ts >= cutoff)
}

fn config_sync_status(
    online: bool,
    desired: Option<&str>,
    applied: Option<&str>,
    last_error_kind: Option<&str>,
) -> &'static str {
    if !online {
        return "offline";
    }
    if last_error_kind.is_some() {
        return "error";
    }
    if let Some(desired) = desired
        && applied == Some(desired)
    {
        return "synced";
    }
    "pending"
}

pub(in crate::http) async fn list_agents(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    RawQuery(raw): RawQuery,
) -> Result<Json<AgentListResponse>, AppError> {
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
                        .map_err(|_| AppError::bad_request("invalid_page", "Invalid page"))?;
                    page = Some(parsed);
                }
                "page_size" => {
                    let parsed = value.parse::<i64>().map_err(|_| {
                        AppError::bad_request("invalid_page_size", "Invalid page_size")
                    })?;
                    page_size = Some(parsed);
                }
                _ => {}
            }
        }
    }

    let labels = normalize_labels(labels)?;
    let mode = parse_labels_mode(labels_mode.as_deref())?;
    let status = parse_status_filter(status.as_deref())?;
    let search = normalize_search_query(search);

    let mut total_qb: QueryBuilder<sqlx::Sqlite> =
        QueryBuilder::new("SELECT COUNT(*) AS total FROM agents a");
    push_agent_list_filters(
        &mut total_qb,
        &labels,
        mode,
        status,
        online_cutoff,
        search.as_deref(),
    );
    let total_row = total_qb.build().fetch_one(&state.db).await?;
    let total = total_row.get::<i64, _>("total");

    let pagination_requested = page.is_some() || page_size.is_some();
    let page = page.unwrap_or(1);
    if page < 1 {
        return Err(AppError::bad_request("invalid_page", "Invalid page"));
    }

    let page_size = if pagination_requested {
        let page_size = page_size.unwrap_or(20);
        if page_size < 1 {
            return Err(AppError::bad_request(
                "invalid_page_size",
                "Invalid page_size",
            ));
        }
        page_size.clamp(1, 100)
    } else {
        total
    };

    let mut ids_qb: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new("SELECT a.id FROM agents a");
    push_agent_list_filters(
        &mut ids_qb,
        &labels,
        mode,
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

    let mut agents: Vec<AgentListItem> = Vec::new();
    if !ids.is_empty() {
        let mut rows_qb: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            r#"
            SELECT
              a.id, a.name, a.revoked_at, a.last_seen_at,
              a.desired_config_snapshot_id, a.applied_config_snapshot_id,
              a.last_config_sync_attempt_at, a.last_config_sync_error_kind, a.last_config_sync_error,
              al.label
            FROM agents a
            LEFT JOIN agent_labels al ON al.agent_id = a.id
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
            let is_new = current_id.as_deref() != Some(&id);
            if is_new {
                let revoked = row.get::<Option<i64>, _>("revoked_at").is_some();
                let last_seen_at = row.get::<Option<i64>, _>("last_seen_at");
                let online = agent_online(revoked, last_seen_at, now);

                let desired_config_snapshot_id =
                    row.get::<Option<String>, _>("desired_config_snapshot_id");
                let applied_config_snapshot_id =
                    row.get::<Option<String>, _>("applied_config_snapshot_id");
                let last_config_sync_error_kind =
                    row.get::<Option<String>, _>("last_config_sync_error_kind");
                let status = config_sync_status(
                    online,
                    desired_config_snapshot_id.as_deref(),
                    applied_config_snapshot_id.as_deref(),
                    last_config_sync_error_kind.as_deref(),
                )
                .to_string();

                agents.push(AgentListItem {
                    id: id.clone(),
                    name: row.get::<Option<String>, _>("name"),
                    revoked,
                    last_seen_at,
                    online,
                    labels: Vec::new(),
                    desired_config_snapshot_id,
                    applied_config_snapshot_id,
                    config_sync_status: status,
                    last_config_sync_attempt_at: row
                        .get::<Option<i64>, _>("last_config_sync_attempt_at"),
                    last_config_sync_error_kind,
                    last_config_sync_error: row.get::<Option<String>, _>("last_config_sync_error"),
                });
                current_id = Some(id);
            }

            if let Some(label) = row.get::<Option<String>, _>("label")
                && let Some(last) = agents.last_mut()
            {
                last.labels.push(label);
            }
        }
    }

    Ok(Json(AgentListResponse {
        items: agents,
        page: if pagination_requested { page } else { 1 },
        page_size: if pagination_requested {
            page_size
        } else {
            total
        },
        total,
    }))
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct AgentDetail {
    id: String,
    name: Option<String>,
    revoked: bool,
    created_at: i64,
    last_seen_at: Option<i64>,
    online: bool,
    capabilities_json: Option<String>,
    labels: Vec<String>,

    desired_config_snapshot_id: Option<String>,
    desired_config_snapshot_at: Option<i64>,
    applied_config_snapshot_id: Option<String>,
    applied_config_snapshot_at: Option<i64>,
    config_sync_status: String,
    last_config_sync_attempt_at: Option<i64>,
    last_config_sync_error_kind: Option<String>,
    last_config_sync_error: Option<String>,
    last_config_sync_error_at: Option<i64>,
}

pub(in crate::http) async fn get_agent(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(agent_id): Path<String>,
) -> Result<Json<AgentDetail>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let row = sqlx::query(
        r#"
        SELECT
          id, name, revoked_at, created_at, last_seen_at, capabilities_json,
          desired_config_snapshot_id, desired_config_snapshot_at,
          applied_config_snapshot_id, applied_config_snapshot_at,
          last_config_sync_attempt_at, last_config_sync_error_kind, last_config_sync_error, last_config_sync_error_at
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

    let desired_config_snapshot_id = row.get::<Option<String>, _>("desired_config_snapshot_id");
    let applied_config_snapshot_id = row.get::<Option<String>, _>("applied_config_snapshot_id");
    let last_config_sync_error_kind = row.get::<Option<String>, _>("last_config_sync_error_kind");
    let status = config_sync_status(
        online,
        desired_config_snapshot_id.as_deref(),
        applied_config_snapshot_id.as_deref(),
        last_config_sync_error_kind.as_deref(),
    )
    .to_string();

    let labels = agent_labels_repo::list_labels_for_agent(&state.db, &agent_id).await?;

    Ok(Json(AgentDetail {
        id: row.get::<String, _>("id"),
        name: row.get::<Option<String>, _>("name"),
        revoked,
        created_at: row.get::<i64, _>("created_at"),
        last_seen_at,
        online,
        capabilities_json: row.get::<Option<String>, _>("capabilities_json"),
        labels,
        desired_config_snapshot_id,
        desired_config_snapshot_at: row.get::<Option<i64>, _>("desired_config_snapshot_at"),
        applied_config_snapshot_id,
        applied_config_snapshot_at: row.get::<Option<i64>, _>("applied_config_snapshot_at"),
        config_sync_status: status,
        last_config_sync_attempt_at: row.get::<Option<i64>, _>("last_config_sync_attempt_at"),
        last_config_sync_error_kind,
        last_config_sync_error: row.get::<Option<String>, _>("last_config_sync_error"),
        last_config_sync_error_at: row.get::<Option<i64>, _>("last_config_sync_error_at"),
    }))
}

pub(in crate::http) async fn revoke_agent(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    sqlx::query("UPDATE agents SET revoked_at = ? WHERE id = ? AND revoked_at IS NULL")
        .bind(now)
        .bind(agent_id)
        .execute(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct RotateAgentKeyResponse {
    agent_id: String,
    agent_key: String,
}

pub(in crate::http) async fn rotate_agent_key(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
) -> Result<Json<RotateAgentKeyResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let agent_key = agents_repo::rotate_agent_key(&state.db, &agent_id)
        .await?
        .ok_or_else(|| AppError::not_found("agent_not_found", "Agent not found"))?;

    Ok(Json(RotateAgentKeyResponse {
        agent_id,
        agent_key,
    }))
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct SyncConfigNowResponse {
    outcome: String,
}

pub(in crate::http) async fn sync_config_now(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(agent_id): Path<String>,
) -> Result<Json<SyncConfigNowResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let exists = sqlx::query("SELECT 1 FROM agents WHERE id = ? LIMIT 1")
        .bind(&agent_id)
        .fetch_optional(&state.db)
        .await?;
    if exists.is_none() {
        return Err(AppError::not_found("agent_not_found", "Agent not found"));
    }

    let outcome = send_node_config_snapshot_with_outcome(
        &state.db,
        &state.secrets,
        &state.agent_manager,
        &agent_id,
    )
    .await?;
    let outcome = match outcome {
        SendConfigSnapshotOutcome::Sent => "sent",
        SendConfigSnapshotOutcome::Unchanged => "unchanged",
        SendConfigSnapshotOutcome::PendingOffline => "pending_offline",
    };

    Ok(Json(SyncConfigNowResponse {
        outcome: outcome.to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::agent_online;

    #[test]
    fn agent_online_false_when_revoked() {
        assert!(!agent_online(true, Some(1000), 1000));
    }

    #[test]
    fn agent_online_false_when_never_seen() {
        assert!(!agent_online(false, None, 1000));
    }

    #[test]
    fn agent_online_false_when_stale() {
        assert!(!agent_online(false, Some(900), 1000));
    }

    #[test]
    fn agent_online_true_when_recent() {
        assert!(agent_online(false, Some(950), 1000));
    }
}
