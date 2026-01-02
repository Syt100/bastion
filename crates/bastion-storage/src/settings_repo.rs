use sqlx::Row;
use sqlx::SqlitePool;
use time::OffsetDateTime;

pub async fn get_value_json(db: &SqlitePool, key: &str) -> Result<Option<String>, anyhow::Error> {
    let row = sqlx::query("SELECT value_json FROM settings WHERE key = ? LIMIT 1")
        .bind(key)
        .fetch_optional(db)
        .await?;

    Ok(row.map(|r| r.get::<String, _>("value_json")))
}

pub async fn upsert_value_json(
    db: &SqlitePool,
    key: &str,
    value_json: &str,
) -> Result<(), anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    sqlx::query(
        r#"
        INSERT INTO settings (key, value_json, updated_at)
        VALUES (?, ?, ?)
        ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at
        "#,
    )
    .bind(key)
    .bind(value_json)
    .bind(now)
    .execute(db)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;

    use super::{get_value_json, upsert_value_json};

    #[tokio::test]
    async fn settings_round_trip() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        assert_eq!(get_value_json(&pool, "k").await.unwrap(), None);

        upsert_value_json(&pool, "k", r#"{"a":1}"#)
            .await
            .expect("upsert");
        assert_eq!(
            get_value_json(&pool, "k").await.unwrap(),
            Some(r#"{"a":1}"#.to_string())
        );

        upsert_value_json(&pool, "k", r#"{"a":2}"#)
            .await
            .expect("upsert2");
        assert_eq!(
            get_value_json(&pool, "k").await.unwrap(),
            Some(r#"{"a":2}"#.to_string())
        );
    }
}

