use base64::Engine as _;
use rand::RngCore;
use sqlx::Row;
use sqlx::SqlitePool;
use time::Duration;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SessionRow {
    pub id: String,
    #[allow(dead_code)]
    pub user_id: i64,
    pub csrf_token: String,
    #[allow(dead_code)]
    pub created_at: i64,
    pub expires_at: i64,
}

pub async fn create_session(db: &SqlitePool, user_id: i64) -> Result<SessionRow, anyhow::Error> {
    let now = OffsetDateTime::now_utc();
    let created_at = now.unix_timestamp();
    let expires_at = (now + Duration::days(7)).unix_timestamp();

    let id = Uuid::new_v4().to_string();
    let csrf_token = random_token_b64_urlsafe(32);

    sqlx::query(
        "INSERT INTO sessions (id, user_id, csrf_token, created_at, expires_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&csrf_token)
    .bind(created_at)
    .bind(expires_at)
    .execute(db)
    .await?;

    Ok(SessionRow {
        id,
        user_id,
        csrf_token,
        created_at,
        expires_at,
    })
}

pub async fn get_session(
    db: &SqlitePool,
    session_id: &str,
) -> Result<Option<SessionRow>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, user_id, csrf_token, created_at, expires_at FROM sessions WHERE id = ? LIMIT 1",
    )
    .bind(session_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let session = SessionRow {
        id: row.get::<String, _>("id"),
        user_id: row.get::<i64, _>("user_id"),
        csrf_token: row.get::<String, _>("csrf_token"),
        created_at: row.get::<i64, _>("created_at"),
        expires_at: row.get::<i64, _>("expires_at"),
    };

    let now = OffsetDateTime::now_utc().unix_timestamp();
    if session.expires_at <= now {
        return Ok(None);
    }

    Ok(Some(session))
}

pub async fn delete_session(db: &SqlitePool, session_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(session_id)
        .execute(db)
        .await?;
    Ok(())
}

fn random_token_b64_urlsafe(size: usize) -> String {
    let mut bytes = vec![0_u8; size];
    rand::rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}
