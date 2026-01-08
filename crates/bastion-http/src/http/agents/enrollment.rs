use axum::Json;
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use tower_cookies::Cookies;
use uuid::Uuid;

use bastion_core::agent;

use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};

#[derive(Debug, Deserialize)]
pub(in crate::http) struct CreateEnrollmentTokenRequest {
    #[serde(default = "default_enroll_ttl_seconds")]
    ttl_seconds: i64,
    remaining_uses: Option<i64>,
}

fn default_enroll_ttl_seconds() -> i64 {
    60 * 60
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct CreateEnrollmentTokenResponse {
    token: String,
    expires_at: i64,
    remaining_uses: Option<i64>,
}

pub(in crate::http) async fn create_enrollment_token(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Json(req): Json<CreateEnrollmentTokenRequest>,
) -> Result<Json<CreateEnrollmentTokenResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let token = agent::generate_token_b64_urlsafe(32);
    let token_hash = agent::sha256_urlsafe_token(&token)?;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let expires_at = now + req.ttl_seconds;

    sqlx::query(
        "INSERT INTO enrollment_tokens (token_hash, created_at, expires_at, remaining_uses) VALUES (?, ?, ?, ?)",
    )
    .bind(token_hash)
    .bind(now)
    .bind(expires_at)
    .bind(req.remaining_uses)
    .execute(&state.db)
    .await?;

    Ok(Json(CreateEnrollmentTokenResponse {
        token,
        expires_at,
        remaining_uses: req.remaining_uses,
    }))
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct AgentEnrollRequest {
    token: String,
    name: Option<String>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct AgentEnrollResponse {
    agent_id: String,
    agent_key: String,
}

pub(in crate::http) async fn agent_enroll(
    state: axum::extract::State<AppState>,
    Json(req): Json<AgentEnrollRequest>,
) -> Result<Json<AgentEnrollResponse>, AppError> {
    let token_hash = agent::sha256_urlsafe_token(&req.token).map_err(|_| {
        AppError::unauthorized("invalid_token", "Invalid enrollment token")
            .with_details(json!({ "field": "token" }))
    })?;
    let now = time::OffsetDateTime::now_utc().unix_timestamp();

    let mut tx = state.db.begin().await?;
    let row = sqlx::query(
        "SELECT expires_at, remaining_uses FROM enrollment_tokens WHERE token_hash = ? LIMIT 1",
    )
    .bind(&token_hash)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(row) = row else {
        return Err(AppError::unauthorized(
            "invalid_token",
            "Invalid enrollment token",
        ));
    };

    let expires_at = row.get::<i64, _>("expires_at");
    let remaining_uses = row.get::<Option<i64>, _>("remaining_uses");

    if expires_at <= now {
        sqlx::query("DELETE FROM enrollment_tokens WHERE token_hash = ?")
            .bind(&token_hash)
            .execute(&mut *tx)
            .await?;
        return Err(AppError::unauthorized(
            "expired_token",
            "Enrollment token expired",
        ));
    }

    if let Some(uses) = remaining_uses {
        if uses <= 0 {
            return Err(AppError::unauthorized(
                "invalid_token",
                "Invalid enrollment token",
            ));
        }
        let new_uses = uses - 1;
        if new_uses == 0 {
            sqlx::query("DELETE FROM enrollment_tokens WHERE token_hash = ?")
                .bind(&token_hash)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query("UPDATE enrollment_tokens SET remaining_uses = ? WHERE token_hash = ?")
                .bind(new_uses)
                .bind(&token_hash)
                .execute(&mut *tx)
                .await?;
        }
    }

    let agent_id = Uuid::new_v4().to_string();
    let agent_key = agent::generate_token_b64_urlsafe(32);
    let agent_key_hash = agent::sha256_urlsafe_token(&agent_key)?;

    sqlx::query("INSERT INTO agents (id, name, key_hash, created_at) VALUES (?, ?, ?, ?)")
        .bind(&agent_id)
        .bind(req.name)
        .bind(agent_key_hash)
        .bind(now)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(AgentEnrollResponse {
        agent_id,
        agent_key,
    }))
}
