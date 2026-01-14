use axum::Json;
use axum::extract::{Path, RawQuery};
use axum::http::{HeaderMap, StatusCode};
use serde::Serialize;
use sqlx::{QueryBuilder, Row};
use tower_cookies::Cookies;

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
}

fn agent_online(revoked: bool, last_seen_at: Option<i64>, now: i64) -> bool {
    if revoked {
        return false;
    }

    let cutoff = now.saturating_sub(60);
    last_seen_at.is_some_and(|ts| ts >= cutoff)
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
        SELECT a.id, a.name, a.revoked_at, a.last_seen_at, al.label
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

            agents.push(AgentListItem {
                id: id.clone(),
                name: row.get::<Option<String>, _>("name"),
                revoked,
                last_seen_at,
                online,
                labels: Vec::new(),
            });
            current_id = Some(id);
        }

        if let Some(label) = row.get::<Option<String>, _>("label") {
            if let Some(last) = agents.last_mut() {
                last.labels.push(label);
            }
        }
    }

    Ok(Json(agents))
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
