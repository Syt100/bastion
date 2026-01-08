use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use serde::Serialize;
use sqlx::Row;
use tower_cookies::Cookies;

use bastion_storage::agents_repo;

use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};

#[derive(Debug, Serialize)]
pub(in crate::http) struct AgentListItem {
    id: String,
    name: Option<String>,
    revoked: bool,
    last_seen_at: Option<i64>,
    online: bool,
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
) -> Result<Json<Vec<AgentListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();

    let rows = sqlx::query(
        "SELECT id, name, revoked_at, last_seen_at FROM agents ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    let agents = rows
        .into_iter()
        .map(|r| {
            let revoked = r.get::<Option<i64>, _>("revoked_at").is_some();
            let last_seen_at = r.get::<Option<i64>, _>("last_seen_at");
            let online = agent_online(revoked, last_seen_at, now);

            AgentListItem {
                id: r.get::<String, _>("id"),
                name: r.get::<Option<String>, _>("name"),
                revoked,
                last_seen_at,
                online,
            }
        })
        .collect();

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
