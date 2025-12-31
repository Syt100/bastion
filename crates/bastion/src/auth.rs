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
    #[allow(dead_code)]
    pub user_id: i64,
    pub csrf_token: String,
    #[allow(dead_code)]
    pub created_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: i64,
    #[allow(dead_code)]
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

const LOGIN_WINDOW_SECONDS: i64 = 10 * 60;
const LOGIN_MAX_FAILURES: i64 = 10;
const LOGIN_LOCK_SECONDS: i64 = 15 * 60;

pub async fn login_throttle_retry_after_seconds(
    db: &SqlitePool,
    ip: &str,
    now: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let row = sqlx::query("SELECT locked_until FROM login_throttle WHERE ip = ? LIMIT 1")
        .bind(ip)
        .fetch_optional(db)
        .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let locked_until = row.get::<Option<i64>, _>("locked_until");
    let Some(locked_until) = locked_until else {
        return Ok(None);
    };

    if locked_until > now {
        return Ok(Some(locked_until.saturating_sub(now)));
    }

    Ok(None)
}

pub async fn clear_login_throttle(db: &SqlitePool, ip: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM login_throttle WHERE ip = ?")
        .bind(ip)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn record_login_failure(db: &SqlitePool, ip: &str, now: i64) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    let row = sqlx::query(
        "SELECT failures, first_failed_at, locked_until FROM login_throttle WHERE ip = ? LIMIT 1",
    )
    .bind(ip)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(row) = row {
        let failures = row.get::<i64, _>("failures");
        let first_failed_at = row.get::<i64, _>("first_failed_at");
        let locked_until = row.get::<Option<i64>, _>("locked_until");

        if locked_until.is_some_and(|t| t > now) {
            sqlx::query("UPDATE login_throttle SET last_failed_at = ? WHERE ip = ?")
                .bind(now)
                .bind(ip)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            return Ok(());
        }

        let window_expired = now.saturating_sub(first_failed_at) > LOGIN_WINDOW_SECONDS;
        if window_expired {
            sqlx::query(
                "UPDATE login_throttle SET failures = 1, first_failed_at = ?, last_failed_at = ?, locked_until = NULL WHERE ip = ?",
            )
            .bind(now)
            .bind(now)
            .bind(ip)
            .execute(&mut *tx)
            .await?;
        } else {
            let new_failures = failures.saturating_add(1);
            let locked_until = if new_failures >= LOGIN_MAX_FAILURES {
                Some(now.saturating_add(LOGIN_LOCK_SECONDS))
            } else {
                None
            };

            sqlx::query(
                "UPDATE login_throttle SET failures = ?, last_failed_at = ?, locked_until = ? WHERE ip = ?",
            )
            .bind(new_failures)
            .bind(now)
            .bind(locked_until)
            .bind(ip)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        return Ok(());
    }

    sqlx::query(
        "INSERT INTO login_throttle (ip, failures, first_failed_at, last_failed_at, locked_until) VALUES (?, ?, ?, ?, NULL)",
    )
    .bind(ip)
    .bind(1_i64)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
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

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;

    use super::{
        LOGIN_LOCK_SECONDS, LOGIN_MAX_FAILURES, clear_login_throttle,
        login_throttle_retry_after_seconds, record_login_failure,
    };

    #[tokio::test]
    async fn login_throttle_locks_after_too_many_failures() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        let ip = "203.0.113.10";
        let now = 1000;

        for _ in 0..LOGIN_MAX_FAILURES {
            record_login_failure(&pool, ip, now).await.expect("record");
        }

        let retry = login_throttle_retry_after_seconds(&pool, ip, now)
            .await
            .expect("retry");
        assert!(retry.is_some());

        let retry = login_throttle_retry_after_seconds(&pool, ip, now + LOGIN_LOCK_SECONDS)
            .await
            .expect("retry2");
        assert!(retry.is_none());

        clear_login_throttle(&pool, ip).await.expect("clear");
        let retry = login_throttle_retry_after_seconds(&pool, ip, now)
            .await
            .expect("retry3");
        assert!(retry.is_none());
    }
}
