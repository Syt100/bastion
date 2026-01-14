use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;

use bastion_core::agent;

pub async fn rotate_agent_key(
    db: &SqlitePool,
    agent_id: &str,
) -> Result<Option<String>, anyhow::Error> {
    let agent_key = agent::generate_token_b64_urlsafe(32);
    let key_hash = agent::sha256_urlsafe_token(&agent_key)?;

    let result = sqlx::query("UPDATE agents SET key_hash = ? WHERE id = ? AND revoked_at IS NULL")
        .bind(key_hash)
        .bind(agent_id)
        .execute(db)
        .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    Ok(Some(agent_key))
}

pub async fn set_desired_config_snapshot(
    db: &SqlitePool,
    agent_id: &str,
    snapshot_id: &str,
) -> Result<(), anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    sqlx::query(
        r#"
        UPDATE agents
        SET desired_config_snapshot_id = ?,
            desired_config_snapshot_at = ?,
            last_config_sync_attempt_at = ?
        WHERE id = ?
        "#,
    )
    .bind(snapshot_id)
    .bind(now)
    .bind(now)
    .bind(agent_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn record_applied_config_snapshot(
    db: &SqlitePool,
    agent_id: &str,
    snapshot_id: &str,
) -> Result<(), anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let desired = sqlx::query("SELECT desired_config_snapshot_id FROM agents WHERE id = ? LIMIT 1")
        .bind(agent_id)
        .fetch_optional(db)
        .await?
        .and_then(|r| {
            r.try_get::<Option<String>, _>("desired_config_snapshot_id")
                .ok()
        })
        .flatten();

    if desired.as_deref() == Some(snapshot_id) {
        sqlx::query(
            r#"
            UPDATE agents
            SET applied_config_snapshot_id = ?,
                applied_config_snapshot_at = ?,
                last_config_sync_error_kind = NULL,
                last_config_sync_error = NULL,
                last_config_sync_error_at = NULL
            WHERE id = ?
            "#,
        )
        .bind(snapshot_id)
        .bind(now)
        .bind(agent_id)
        .execute(db)
        .await?;
    } else {
        sqlx::query(
            r#"
            UPDATE agents
            SET applied_config_snapshot_id = ?,
                applied_config_snapshot_at = ?
            WHERE id = ?
            "#,
        )
        .bind(snapshot_id)
        .bind(now)
        .bind(agent_id)
        .execute(db)
        .await?;
    }

    Ok(())
}

pub async fn record_config_sync_error(
    db: &SqlitePool,
    agent_id: &str,
    error_kind: &str,
    error: &str,
) -> Result<(), anyhow::Error> {
    const MAX_CHARS: usize = 200;

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let msg = if error.chars().count() <= MAX_CHARS {
        error.to_string()
    } else {
        let mut cutoff = error.len();
        for (i, (idx, _)) in error.char_indices().enumerate() {
            if i == MAX_CHARS {
                cutoff = idx;
                break;
            }
        }
        format!("{}…", &error[..cutoff])
    };

    sqlx::query(
        r#"
        UPDATE agents
        SET last_config_sync_attempt_at = ?,
            last_config_sync_error_kind = ?,
            last_config_sync_error = ?,
            last_config_sync_error_at = ?
        WHERE id = ?
        "#,
    )
    .bind(now)
    .bind(error_kind)
    .bind(msg)
    .bind(now)
    .bind(agent_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn clear_config_sync_error(db: &SqlitePool, agent_id: &str) -> Result<(), anyhow::Error> {
    sqlx::query(
        r#"
        UPDATE agents
        SET last_config_sync_error_kind = NULL,
            last_config_sync_error = NULL,
            last_config_sync_error_at = NULL
        WHERE id = ?
        "#,
    )
    .bind(agent_id)
    .execute(db)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::Row;
    use tempfile::TempDir;

    use crate::db;
    use bastion_core::agent;

    use super::{
        record_applied_config_snapshot, record_config_sync_error, rotate_agent_key,
        set_desired_config_snapshot,
    };

    #[tokio::test]
    async fn rotate_updates_hash() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();

        let agent_id = "agent1";
        let old_key = agent::generate_token_b64_urlsafe(32);
        let old_hash = agent::sha256_urlsafe_token(&old_key).unwrap();

        sqlx::query("INSERT INTO agents (id, name, key_hash, created_at) VALUES (?, NULL, ?, ?)")
            .bind(agent_id)
            .bind(old_hash)
            .bind(1i64)
            .execute(&pool)
            .await
            .unwrap();

        let new_key = rotate_agent_key(&pool, agent_id).await.unwrap().unwrap();
        assert_ne!(new_key, old_key);
        let new_hash = agent::sha256_urlsafe_token(&new_key).unwrap();

        let row = sqlx::query("SELECT key_hash FROM agents WHERE id = ? LIMIT 1")
            .bind(agent_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        let stored = row.get::<Vec<u8>, _>("key_hash");
        assert_eq!(stored, new_hash);
    }

    #[tokio::test]
    async fn rotate_is_noop_for_revoked_agent() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();

        let agent_id = "agent1";
        let old_key = agent::generate_token_b64_urlsafe(32);
        let old_hash = agent::sha256_urlsafe_token(&old_key).unwrap();

        sqlx::query(
            "INSERT INTO agents (id, name, key_hash, created_at, revoked_at) VALUES (?, NULL, ?, ?, ?)",
        )
        .bind(agent_id)
        .bind(old_hash)
        .bind(1i64)
        .bind(2i64)
        .execute(&pool)
        .await
        .unwrap();

        assert!(rotate_agent_key(&pool, agent_id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn desired_snapshot_updates_desired_and_attempt_timestamps() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();

        let agent_id = "agent1";
        let agent_key = agent::generate_token_b64_urlsafe(32);
        let key_hash = agent::sha256_urlsafe_token(&agent_key).unwrap();

        sqlx::query("INSERT INTO agents (id, name, key_hash, created_at) VALUES (?, NULL, ?, ?)")
            .bind(agent_id)
            .bind(key_hash)
            .bind(1i64)
            .execute(&pool)
            .await
            .unwrap();

        set_desired_config_snapshot(&pool, agent_id, "snap1")
            .await
            .unwrap();

        let row = sqlx::query(
            "SELECT desired_config_snapshot_id, desired_config_snapshot_at, last_config_sync_attempt_at FROM agents WHERE id = ? LIMIT 1",
        )
        .bind(agent_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(
            row.get::<Option<String>, _>("desired_config_snapshot_id")
                .as_deref(),
            Some("snap1")
        );
        let desired_at = row.get::<Option<i64>, _>("desired_config_snapshot_at");
        let attempt_at = row.get::<Option<i64>, _>("last_config_sync_attempt_at");
        assert!(desired_at.is_some());
        assert_eq!(desired_at, attempt_at);
    }

    #[tokio::test]
    async fn records_and_truncates_last_sync_error_safely() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();

        let agent_id = "agent1";
        let agent_key = agent::generate_token_b64_urlsafe(32);
        let key_hash = agent::sha256_urlsafe_token(&agent_key).unwrap();

        sqlx::query("INSERT INTO agents (id, name, key_hash, created_at) VALUES (?, NULL, ?, ?)")
            .bind(agent_id)
            .bind(key_hash)
            .bind(1i64)
            .execute(&pool)
            .await
            .unwrap();

        let long = "错".repeat(1000);
        record_config_sync_error(&pool, agent_id, "send_failed", &long)
            .await
            .unwrap();

        let row = sqlx::query(
            "SELECT last_config_sync_error_kind, last_config_sync_error, last_config_sync_error_at, last_config_sync_attempt_at FROM agents WHERE id = ? LIMIT 1",
        )
        .bind(agent_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(
            row.get::<Option<String>, _>("last_config_sync_error_kind")
                .as_deref(),
            Some("send_failed")
        );
        let msg = row
            .get::<Option<String>, _>("last_config_sync_error")
            .unwrap();
        assert!(msg.ends_with('…'));
        assert!(msg.chars().count() <= 201);
        assert!(
            row.get::<Option<i64>, _>("last_config_sync_error_at")
                .is_some()
        );
        assert!(
            row.get::<Option<i64>, _>("last_config_sync_attempt_at")
                .is_some()
        );
    }

    #[tokio::test]
    async fn config_ack_clears_error_when_matches_desired_snapshot() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();

        let agent_id = "agent1";
        let agent_key = agent::generate_token_b64_urlsafe(32);
        let key_hash = agent::sha256_urlsafe_token(&agent_key).unwrap();

        sqlx::query(
            r#"
            INSERT INTO agents (
              id, name, key_hash, created_at,
              desired_config_snapshot_id,
              last_config_sync_error_kind, last_config_sync_error, last_config_sync_error_at
            )
            VALUES (?, NULL, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(agent_id)
        .bind(key_hash)
        .bind(1i64)
        .bind("snap1")
        .bind("send_failed")
        .bind("oops")
        .bind(123i64)
        .execute(&pool)
        .await
        .unwrap();

        record_applied_config_snapshot(&pool, agent_id, "snap1")
            .await
            .unwrap();

        let row = sqlx::query(
            "SELECT applied_config_snapshot_id, last_config_sync_error_kind, last_config_sync_error, last_config_sync_error_at FROM agents WHERE id = ? LIMIT 1",
        )
        .bind(agent_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(
            row.get::<Option<String>, _>("applied_config_snapshot_id")
                .as_deref(),
            Some("snap1")
        );
        assert!(
            row.get::<Option<String>, _>("last_config_sync_error_kind")
                .is_none()
        );
        assert!(
            row.get::<Option<String>, _>("last_config_sync_error")
                .is_none()
        );
        assert!(
            row.get::<Option<i64>, _>("last_config_sync_error_at")
                .is_none()
        );
    }

    #[tokio::test]
    async fn config_ack_does_not_clear_error_when_snapshot_does_not_match_desired() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();

        let agent_id = "agent1";
        let agent_key = agent::generate_token_b64_urlsafe(32);
        let key_hash = agent::sha256_urlsafe_token(&agent_key).unwrap();

        sqlx::query(
            r#"
            INSERT INTO agents (
              id, name, key_hash, created_at,
              desired_config_snapshot_id,
              last_config_sync_error_kind, last_config_sync_error, last_config_sync_error_at
            )
            VALUES (?, NULL, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(agent_id)
        .bind(key_hash)
        .bind(1i64)
        .bind("desired")
        .bind("send_failed")
        .bind("oops")
        .bind(123i64)
        .execute(&pool)
        .await
        .unwrap();

        record_applied_config_snapshot(&pool, agent_id, "other")
            .await
            .unwrap();

        let row = sqlx::query(
            "SELECT applied_config_snapshot_id, last_config_sync_error_kind, last_config_sync_error, last_config_sync_error_at FROM agents WHERE id = ? LIMIT 1",
        )
        .bind(agent_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(
            row.get::<Option<String>, _>("applied_config_snapshot_id")
                .as_deref(),
            Some("other")
        );
        assert_eq!(
            row.get::<Option<String>, _>("last_config_sync_error_kind")
                .as_deref(),
            Some("send_failed")
        );
        assert_eq!(
            row.get::<Option<String>, _>("last_config_sync_error")
                .as_deref(),
            Some("oops")
        );
        assert_eq!(
            row.get::<Option<i64>, _>("last_config_sync_error_at"),
            Some(123i64)
        );
    }
}
