use sqlx::{QueryBuilder, Row, SqlitePool};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct AgentLabelCount {
    pub label: String,
    pub count: i64,
}

pub async fn list_labels_for_agent(
    db: &SqlitePool,
    agent_id: &str,
) -> Result<Vec<String>, anyhow::Error> {
    let rows = sqlx::query("SELECT label FROM agent_labels WHERE agent_id = ? ORDER BY label ASC")
        .bind(agent_id)
        .fetch_all(db)
        .await?;

    Ok(rows
        .into_iter()
        .map(|r| r.get::<String, _>("label"))
        .collect())
}

pub async fn list_label_counts(db: &SqlitePool) -> Result<Vec<AgentLabelCount>, anyhow::Error> {
    let rows = sqlx::query(
        r#"
        SELECT label, COUNT(*) AS cnt
        FROM agent_labels
        GROUP BY label
        ORDER BY cnt DESC, label ASC
        "#,
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| AgentLabelCount {
            label: r.get::<String, _>("label"),
            count: r.get::<i64, _>("cnt"),
        })
        .collect())
}

pub async fn add_labels(
    db: &SqlitePool,
    agent_id: &str,
    labels: &[String],
) -> Result<(), anyhow::Error> {
    if labels.is_empty() {
        return Ok(());
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    for label in labels {
        sqlx::query(
            r#"
            INSERT INTO agent_labels (agent_id, label, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(agent_id, label) DO UPDATE SET
              updated_at = excluded.updated_at
            "#,
        )
        .bind(agent_id)
        .bind(label)
        .bind(now)
        .bind(now)
        .execute(db)
        .await?;
    }

    Ok(())
}

pub async fn remove_labels(
    db: &SqlitePool,
    agent_id: &str,
    labels: &[String],
) -> Result<(), anyhow::Error> {
    if labels.is_empty() {
        return Ok(());
    }

    for label in labels {
        sqlx::query("DELETE FROM agent_labels WHERE agent_id = ? AND label = ?")
            .bind(agent_id)
            .bind(label)
            .execute(db)
            .await?;
    }

    Ok(())
}

pub async fn set_labels(
    db: &SqlitePool,
    agent_id: &str,
    labels: &[String],
) -> Result<(), anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let mut tx = db.begin().await?;

    if labels.is_empty() {
        sqlx::query("DELETE FROM agent_labels WHERE agent_id = ?")
            .bind(agent_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Ok(());
    }

    // Delete labels that are no longer present.
    let mut qb: QueryBuilder<sqlx::Sqlite> =
        QueryBuilder::new("DELETE FROM agent_labels WHERE agent_id = ");
    qb.push_bind(agent_id);
    qb.push(" AND label NOT IN (");
    let mut separated = qb.separated(", ");
    for label in labels {
        separated.push_bind(label);
    }
    separated.push_unseparated(")");
    qb.build().execute(&mut *tx).await?;

    // Upsert all desired labels.
    for label in labels {
        sqlx::query(
            r#"
            INSERT INTO agent_labels (agent_id, label, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(agent_id, label) DO UPDATE SET
              updated_at = excluded.updated_at
            "#,
        )
        .bind(agent_id)
        .bind(label)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;

    use super::{add_labels, list_label_counts, list_labels_for_agent, remove_labels, set_labels};

    async fn seed_agent(db: &SqlitePool, id: &str) {
        sqlx::query("INSERT INTO agents (id, name, key_hash, created_at) VALUES (?, NULL, ?, ?)")
            .bind(id)
            .bind(vec![0u8; 32])
            .bind(1i64)
            .execute(db)
            .await
            .unwrap();
    }

    use sqlx::SqlitePool;

    #[tokio::test]
    async fn labels_round_trip_and_counts() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        seed_agent(&pool, "a1").await;
        seed_agent(&pool, "a2").await;

        add_labels(&pool, "a1", &["prod".to_string(), "shanghai".to_string()])
            .await
            .unwrap();
        add_labels(&pool, "a2", &["prod".to_string()])
            .await
            .unwrap();

        let a1 = list_labels_for_agent(&pool, "a1").await.unwrap();
        assert_eq!(a1, vec!["prod".to_string(), "shanghai".to_string()]);

        let counts = list_label_counts(&pool).await.unwrap();
        assert_eq!(counts[0].label, "prod");
        assert_eq!(counts[0].count, 2);

        remove_labels(&pool, "a1", &["prod".to_string()])
            .await
            .unwrap();
        let a1 = list_labels_for_agent(&pool, "a1").await.unwrap();
        assert_eq!(a1, vec!["shanghai".to_string()]);

        set_labels(&pool, "a1", &["prod".to_string(), "db".to_string()])
            .await
            .unwrap();
        let a1 = list_labels_for_agent(&pool, "a1").await.unwrap();
        assert_eq!(a1, vec!["db".to_string(), "prod".to_string()]);
    }
}
