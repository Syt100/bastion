use sqlx::Row;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use uuid::Uuid;

pub const CHANNEL_WECOM_BOT: &str = "wecom_bot";

#[derive(Debug, Clone)]
pub struct NotificationRow {
    pub id: String,
    pub run_id: String,
    pub channel: String,
    pub secret_name: String,
    pub attempts: i64,
}

pub async fn enqueue_wecom_bots_for_run(
    db: &SqlitePool,
    run_id: &str,
) -> Result<i64, anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let bots = sqlx::query("SELECT name FROM secrets WHERE kind = ? ORDER BY updated_at DESC")
        .bind("wecom_bot")
        .fetch_all(db)
        .await?;

    if bots.is_empty() {
        return Ok(0);
    }

    let mut inserted = 0_i64;
    for row in bots {
        let name = row.get::<String, _>("name");
        let id = Uuid::new_v4().to_string();
        let result = sqlx::query(
            "INSERT OR IGNORE INTO notifications (id, run_id, channel, secret_name, status, attempts, next_attempt_at, created_at, updated_at) VALUES (?, ?, ?, ?, 'queued', 0, ?, ?, ?)",
        )
        .bind(id)
        .bind(run_id)
        .bind(CHANNEL_WECOM_BOT)
        .bind(name)
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(db)
        .await?;
        inserted += result.rows_affected() as i64;
    }

    Ok(inserted)
}

pub async fn claim_next_due(
    db: &SqlitePool,
    now: i64,
) -> Result<Option<NotificationRow>, anyhow::Error> {
    let row = sqlx::query(
        "SELECT id, run_id, channel, secret_name, attempts FROM notifications WHERE status = 'queued' AND next_attempt_at <= ? ORDER BY next_attempt_at ASC LIMIT 1",
    )
    .bind(now)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(NotificationRow {
        id: row.get::<String, _>("id"),
        run_id: row.get::<String, _>("run_id"),
        channel: row.get::<String, _>("channel"),
        secret_name: row.get::<String, _>("secret_name"),
        attempts: row.get::<i64, _>("attempts"),
    }))
}

pub async fn mark_sent(db: &SqlitePool, id: &str, now: i64) -> Result<(), anyhow::Error> {
    sqlx::query(
        "UPDATE notifications SET status = 'sent', updated_at = ?, last_error = NULL WHERE id = ?",
    )
    .bind(now)
    .bind(id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn mark_failed(
    db: &SqlitePool,
    id: &str,
    attempts: i64,
    last_error: &str,
    now: i64,
) -> Result<(), anyhow::Error> {
    sqlx::query("UPDATE notifications SET status = 'failed', attempts = ?, updated_at = ?, last_error = ? WHERE id = ?")
        .bind(attempts)
        .bind(now)
        .bind(last_error)
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn reschedule(
    db: &SqlitePool,
    id: &str,
    attempts: i64,
    next_attempt_at: i64,
    last_error: &str,
    now: i64,
) -> Result<(), anyhow::Error> {
    sqlx::query("UPDATE notifications SET status = 'queued', attempts = ?, next_attempt_at = ?, updated_at = ?, last_error = ? WHERE id = ?")
        .bind(attempts)
        .bind(next_attempt_at)
        .bind(now)
        .bind(last_error)
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;

    use super::enqueue_wecom_bots_for_run;

    #[tokio::test]
    async fn enqueue_dedupes_per_run_per_bot() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        // Seed two wecom bots.
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        for name in ["a", "b"] {
            sqlx::query(
                "INSERT INTO secrets (id, kind, name, kid, nonce, ciphertext, created_at, updated_at) VALUES (?, 'wecom_bot', ?, 1, X'00', X'00', ?, ?)",
            )
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(name)
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await
            .expect("insert secret");
        }

        // Seed a run row, as notifications has a FK.
        sqlx::query(
            "INSERT INTO jobs (id, name, schedule, overlap_policy, spec_json, created_at, updated_at) VALUES (?, ?, NULL, 'queue', ?, ?, ?)",
        )
        .bind("job1")
        .bind("job1")
        .bind(r#"{"v":1,"type":"filesystem","source":{"root":"/"},"target":{"type":"local_dir","base_dir":"/tmp"}}"#)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert job");

        sqlx::query(
            "INSERT INTO runs (id, job_id, status, started_at, ended_at) VALUES (?, ?, 'success', ?, ?)",
        )
        .bind("run1")
        .bind("job1")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert run");

        let inserted1 = enqueue_wecom_bots_for_run(&pool, "run1").await.unwrap();
        assert_eq!(inserted1, 2);

        let inserted2 = enqueue_wecom_bots_for_run(&pool, "run1").await.unwrap();
        assert_eq!(inserted2, 0);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM notifications")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 2);
    }
}
