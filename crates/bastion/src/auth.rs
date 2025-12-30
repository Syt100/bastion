use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
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
    pub user_id: i64,
    pub csrf_token: String,
    pub created_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

pub async fn users_count(db: &SqlitePool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(1) FROM users")
        .fetch_one(db)
        .await
}

pub async fn create_user(
    db: &SqlitePool,
    username: &str,
    password: &str,
) -> Result<(), anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let password_hash = hash_password(password)?;
    sqlx::query("INSERT INTO users (username, password_hash, created_at) VALUES (?, ?, ?)")
        .bind(username)
        .bind(password_hash)
        .bind(now)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn find_user_by_username(
    db: &SqlitePool,
    username: &str,
) -> Result<Option<UserRow>, sqlx::Error> {
    let row =
        sqlx::query("SELECT id, username, password_hash FROM users WHERE username = ? LIMIT 1")
            .bind(username)
            .fetch_optional(db)
            .await?;

    Ok(row.map(|r| UserRow {
        id: r.get::<i64, _>("id"),
        username: r.get::<String, _>("username"),
        password_hash: r.get::<String, _>("password_hash"),
    }))
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

pub fn hash_password(password: &str) -> Result<String, anyhow::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password_hash: &str, password: &str) -> Result<bool, anyhow::Error> {
    let parsed = PasswordHash::new(password_hash).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

fn random_token_b64_urlsafe(size: usize) -> String {
    let mut bytes = vec![0_u8; size];
    rand::rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}
