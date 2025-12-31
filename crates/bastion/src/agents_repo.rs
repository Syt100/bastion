use sqlx::SqlitePool;

use crate::agent;

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

#[cfg(test)]
mod tests {
    use sqlx::Row;
    use tempfile::TempDir;

    use crate::agent;
    use crate::db;

    use super::rotate_agent_key;

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
}
