use sqlx::Row;
use sqlx::SqlitePool;
use time::OffsetDateTime;

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
    let password_hash = super::hash_password(password)?;
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
