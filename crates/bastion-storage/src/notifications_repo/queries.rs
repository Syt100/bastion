use sqlx::Row;
use sqlx::SqlitePool;

use super::{NotificationListItem, NotificationRow};

pub async fn get_notification(
    db: &SqlitePool,
    id: &str,
) -> Result<Option<NotificationRow>, anyhow::Error> {
    let row = sqlx::query(
        "SELECT id, run_id, channel, secret_name, attempts FROM notifications WHERE id = ? LIMIT 1",
    )
    .bind(id)
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

pub async fn list_queue(
    db: &SqlitePool,
    status: Option<&str>,
    channel: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<NotificationListItem>, anyhow::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
          n.id,
          n.run_id,
          r.job_id,
          j.name AS job_name,
          n.channel,
          n.secret_name,
          n.status,
          n.attempts,
          n.next_attempt_at,
          n.created_at,
          n.updated_at,
          n.last_error,
          CASE WHEN s.id IS NULL THEN 1 ELSE 0 END AS destination_deleted,
          CASE WHEN s.id IS NULL THEN 0 ELSE COALESCE(d.enabled, 1) END AS destination_enabled
        FROM notifications n
        JOIN runs r ON r.id = n.run_id
        JOIN jobs j ON j.id = r.job_id
        LEFT JOIN secrets s ON (
          (n.channel = 'wecom_bot' AND s.kind = 'wecom_bot' AND s.name = n.secret_name) OR
          (n.channel = 'email' AND s.kind = 'smtp' AND s.name = n.secret_name)
        )
        LEFT JOIN notification_destinations d ON d.secret_kind = s.kind AND d.secret_name = s.name
        WHERE (? IS NULL OR n.status = ?) AND (? IS NULL OR n.channel = ?)
        ORDER BY n.created_at DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(status)
    .bind(status)
    .bind(channel)
    .bind(channel)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(NotificationListItem {
            id: row.get::<String, _>("id"),
            run_id: row.get::<String, _>("run_id"),
            job_id: row.get::<String, _>("job_id"),
            job_name: row.get::<String, _>("job_name"),
            channel: row.get::<String, _>("channel"),
            secret_name: row.get::<String, _>("secret_name"),
            status: row.get::<String, _>("status"),
            attempts: row.get::<i64, _>("attempts"),
            next_attempt_at: row.get::<i64, _>("next_attempt_at"),
            created_at: row.get::<i64, _>("created_at"),
            updated_at: row.get::<i64, _>("updated_at"),
            last_error: row.get::<Option<String>, _>("last_error"),
            destination_deleted: row.get::<i64, _>("destination_deleted") != 0,
            destination_enabled: row.get::<i64, _>("destination_enabled") != 0,
        });
    }
    Ok(out)
}

pub async fn count_queue(
    db: &SqlitePool,
    status: Option<&str>,
    channel: Option<&str>,
) -> Result<i64, anyhow::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(1) FROM notifications WHERE (? IS NULL OR status = ?) AND (? IS NULL OR channel = ?)",
    )
    .bind(status)
    .bind(status)
    .bind(channel)
    .bind(channel)
    .fetch_one(db)
    .await?;
    Ok(count)
}
