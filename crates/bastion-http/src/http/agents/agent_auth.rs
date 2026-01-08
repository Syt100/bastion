use axum::http::HeaderMap;
use sqlx::{Row, SqlitePool};

use bastion_core::agent;

use super::super::AppError;

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let header = headers.get("authorization")?.to_str().ok()?;
    let token = header.strip_prefix("Bearer ")?;
    Some(token.trim().to_string())
}

pub(super) async fn authenticate_agent(
    db: &SqlitePool,
    headers: &HeaderMap,
) -> Result<String, AppError> {
    let agent_key = bearer_token(headers)
        .ok_or_else(|| AppError::unauthorized("unauthorized", "Unauthorized"))?;
    let key_hash = agent::sha256_urlsafe_token(&agent_key)
        .map_err(|_| AppError::unauthorized("unauthorized", "Unauthorized"))?;

    let row = sqlx::query("SELECT id, revoked_at FROM agents WHERE key_hash = ? LIMIT 1")
        .bind(key_hash)
        .fetch_optional(db)
        .await?;

    let Some(row) = row else {
        return Err(AppError::unauthorized("unauthorized", "Unauthorized"));
    };
    if row.get::<Option<i64>, _>("revoked_at").is_some() {
        return Err(AppError::unauthorized("revoked", "Agent revoked"));
    }

    Ok(row.get::<String, _>("id"))
}
