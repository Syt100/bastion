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
) -> Result<Json<Vec<AgentListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();

    let mut labels = Vec::new();
    let mut labels_mode: Option<String> = None;
    if let Some(raw) = raw {
        for (key, value) in url::form_urlencoded::parse(raw.as_bytes()) {
            match key.as_ref() {
                "labels" | "labels[]" => labels.push(value.into_owned()),
                "labels_mode" => labels_mode = Some(value.into_owned()),
                _ => {}
            }
        }
    }
    let labels = normalize_labels(labels)?;
    let mode = parse_labels_mode(labels_mode.as_deref())?;

    let mut qb: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
        r#"
        SELECT
          a.id, a.name, a.revoked_at, a.last_seen_at,
          a.desired_config_snapshot_id, a.applied_config_snapshot_id,
          a.last_config_sync_attempt_at, a.last_config_sync_error_kind, a.last_config_sync_error,
          al.label
        FROM agents a
        LEFT JOIN agent_labels al ON al.agent_id = a.id
        "#,
    );

    if !labels.is_empty() {
        qb.push(" WHERE a.id IN (");
        match mode {
            LabelsMode::And => {
                qb.push("SELECT al2.agent_id FROM agent_labels al2 WHERE al2.label IN (");
                let mut separated = qb.separated(", ");
                for label in &labels {
                    separated.push_bind(label);
                }
                separated.push_unseparated(")");
                qb.push(" GROUP BY al2.agent_id HAVING COUNT(DISTINCT al2.label) = ");
                qb.push_bind(labels.len() as i64);
            }
            LabelsMode::Or => {
                qb.push("SELECT DISTINCT al2.agent_id FROM agent_labels al2 WHERE al2.label IN (");
                let mut separated = qb.separated(", ");
                for label in &labels {
                    separated.push_bind(label);
                }
                separated.push_unseparated(")");
            }
        }
        qb.push(")");
    }

    qb.push(" ORDER BY a.created_at DESC, a.id ASC, al.label ASC");

    let rows = qb.build().fetch_all(&state.db).await?;

    let mut agents: Vec<AgentListItem> = Vec::new();
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

    Ok(Json(agents))
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
