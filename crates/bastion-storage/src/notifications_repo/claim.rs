use sqlx::Row;
use sqlx::SqlitePool;

use super::NotificationRow;

pub async fn claim_next_due(
    db: &SqlitePool,
    now: i64,
) -> Result<Option<NotificationRow>, anyhow::Error> {
    let row = sqlx::query(
        "UPDATE notifications SET status = 'sending', updated_at = ? WHERE id = (SELECT id FROM notifications WHERE status = 'queued' AND next_attempt_at <= ? ORDER BY next_attempt_at ASC LIMIT 1) RETURNING id, run_id, channel, secret_name, attempts",
    )
    .bind(now)
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

pub async fn next_due_at(db: &SqlitePool) -> Result<Option<i64>, anyhow::Error> {
    let row = sqlx::query(
        "SELECT next_attempt_at FROM notifications WHERE status = 'queued' ORDER BY next_attempt_at ASC LIMIT 1",
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| r.get::<i64, _>("next_attempt_at")))
}
