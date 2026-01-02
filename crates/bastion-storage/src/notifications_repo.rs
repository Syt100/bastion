use sqlx::Row;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use uuid::Uuid;

pub const CHANNEL_WECOM_BOT: &str = "wecom_bot";
pub const CHANNEL_EMAIL: &str = "email";

pub const STATUS_QUEUED: &str = "queued";
pub const STATUS_SENDING: &str = "sending";
pub const STATUS_SENT: &str = "sent";
pub const STATUS_FAILED: &str = "failed";
pub const STATUS_CANCELED: &str = "canceled";

#[derive(Debug, Clone)]
pub struct NotificationRow {
    pub id: String,
    pub run_id: String,
    pub channel: String,
    pub secret_name: String,
    pub attempts: i64,
}

#[derive(Debug, Clone)]
pub struct NotificationListItem {
    pub id: String,
    pub run_id: String,
    pub job_id: String,
    pub job_name: String,
    pub channel: String,
    pub secret_name: String,
    pub status: String,
    pub attempts: i64,
    pub next_attempt_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_error: Option<String>,
    pub destination_deleted: bool,
    pub destination_enabled: bool,
}

pub async fn enqueue_wecom_bots_for_run(
    db: &SqlitePool,
    run_id: &str,
) -> Result<i64, anyhow::Error> {
    let bots = sqlx::query("SELECT name FROM secrets WHERE kind = ? ORDER BY updated_at DESC")
        .bind("wecom_bot")
        .fetch_all(db)
        .await?;

    if bots.is_empty() {
        return Ok(0);
    }

    let names = bots.into_iter().map(|r| r.get::<String, _>("name")).collect::<Vec<_>>();
    enqueue_for_run(db, run_id, CHANNEL_WECOM_BOT, &names).await
}

pub async fn enqueue_emails_for_run(db: &SqlitePool, run_id: &str) -> Result<i64, anyhow::Error> {
    let destinations =
        sqlx::query("SELECT name FROM secrets WHERE kind = ? ORDER BY updated_at DESC")
            .bind("smtp")
            .fetch_all(db)
            .await?;

    if destinations.is_empty() {
        return Ok(0);
    }

    let names = destinations
        .into_iter()
        .map(|r| r.get::<String, _>("name"))
        .collect::<Vec<_>>();
    enqueue_for_run(db, run_id, CHANNEL_EMAIL, &names).await
}

pub async fn enqueue_for_run(
    db: &SqlitePool,
    run_id: &str,
    channel: &str,
    secret_names: &[String],
) -> Result<i64, anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let mut inserted = 0_i64;
    for name in secret_names {
        let name = name.trim();
        if name.is_empty() {
            continue;
        }

        let id = Uuid::new_v4().to_string();
        let result = sqlx::query(
            "INSERT OR IGNORE INTO notifications (id, run_id, channel, secret_name, status, attempts, next_attempt_at, created_at, updated_at) VALUES (?, ?, ?, ?, 'queued', 0, ?, ?, ?)",
        )
        .bind(id)
        .bind(run_id)
        .bind(channel)
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

pub async fn mark_sent(db: &SqlitePool, id: &str, now: i64) -> Result<(), anyhow::Error> {
    sqlx::query(
        "UPDATE notifications SET status = 'sent', updated_at = ?, last_error = NULL WHERE id = ? AND status = 'sending'",
    )
    .bind(now)
    .bind(id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn mark_canceled(
    db: &SqlitePool,
    id: &str,
    reason: &str,
    now: i64,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        "UPDATE notifications SET status = 'canceled', updated_at = ?, last_error = ? WHERE id = ? AND status = 'sending'",
    )
    .bind(now)
    .bind(reason)
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
    sqlx::query("UPDATE notifications SET status = 'failed', attempts = ?, updated_at = ?, last_error = ? WHERE id = ? AND status = 'sending'")
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
    sqlx::query("UPDATE notifications SET status = 'queued', attempts = ?, next_attempt_at = ?, updated_at = ?, last_error = ? WHERE id = ? AND status = 'sending'")
        .bind(attempts)
        .bind(next_attempt_at)
        .bind(now)
        .bind(last_error)
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

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

pub async fn cancel_queued_by_id(
    db: &SqlitePool,
    id: &str,
    reason: &str,
    now: i64,
) -> Result<bool, anyhow::Error> {
    let result = sqlx::query(
        "UPDATE notifications SET status = 'canceled', updated_at = ?, last_error = ? WHERE id = ? AND status = 'queued'",
    )
    .bind(now)
    .bind(reason)
    .bind(id)
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn retry_now_by_id(
    db: &SqlitePool,
    id: &str,
    now: i64,
) -> Result<bool, anyhow::Error> {
    let result = sqlx::query(
        "UPDATE notifications SET status = 'queued', attempts = 0, next_attempt_at = ?, updated_at = ? WHERE id = ? AND status IN ('failed', 'canceled')",
    )
    .bind(now)
    .bind(now)
    .bind(id)
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn cancel_queued_for_destination(
    db: &SqlitePool,
    channel: &str,
    secret_name: &str,
    reason: &str,
    now: i64,
) -> Result<i64, anyhow::Error> {
    let result = sqlx::query(
        "UPDATE notifications SET status = 'canceled', updated_at = ?, last_error = ? WHERE status = 'queued' AND channel = ? AND secret_name = ?",
    )
    .bind(now)
    .bind(reason)
    .bind(channel)
    .bind(secret_name)
    .execute(db)
    .await?;
    Ok(result.rows_affected() as i64)
}

pub async fn cancel_queued_for_channel(
    db: &SqlitePool,
    channel: &str,
    reason: &str,
    now: i64,
) -> Result<i64, anyhow::Error> {
    let result = sqlx::query(
        "UPDATE notifications SET status = 'canceled', updated_at = ?, last_error = ? WHERE status = 'queued' AND channel = ?",
    )
    .bind(now)
    .bind(reason)
    .bind(channel)
    .execute(db)
    .await?;
    Ok(result.rows_affected() as i64)
}

pub async fn cancel_all_queued(
    db: &SqlitePool,
    reason: &str,
    now: i64,
) -> Result<i64, anyhow::Error> {
    let result = sqlx::query(
        "UPDATE notifications SET status = 'canceled', updated_at = ?, last_error = ? WHERE status = 'queued'",
    )
    .bind(now)
    .bind(reason)
    .execute(db)
    .await?;
    Ok(result.rows_affected() as i64)
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

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;

    use super::enqueue_emails_for_run;
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

    #[tokio::test]
    async fn enqueue_email_dedupes_per_run_per_destination() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        // Seed two smtp destinations.
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        for name in ["a", "b"] {
            sqlx::query(
                "INSERT INTO secrets (id, kind, name, kid, nonce, ciphertext, created_at, updated_at) VALUES (?, 'smtp', ?, 1, X'00', X'00', ?, ?)",
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

        let inserted1 = enqueue_emails_for_run(&pool, "run1").await.unwrap();
        assert_eq!(inserted1, 2);

        let inserted2 = enqueue_emails_for_run(&pool, "run1").await.unwrap();
        assert_eq!(inserted2, 0);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM notifications")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn cancel_and_retry_now_change_status() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        // Seed smtp destination.
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        sqlx::query(
            "INSERT INTO secrets (id, kind, name, kid, nonce, ciphertext, created_at, updated_at) VALUES (?, 'smtp', ?, 1, X'00', X'00', ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind("smtp1")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert secret");

        // Seed run + job.
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

        let inserted = enqueue_emails_for_run(&pool, "run1").await.unwrap();
        assert_eq!(inserted, 1);

        let id: String = sqlx::query_scalar("SELECT id FROM notifications LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        let canceled = super::cancel_queued_by_id(&pool, &id, "canceled", now)
            .await
            .unwrap();
        assert!(canceled);

        // Pretend it was failed, then retry-now should work.
        sqlx::query("UPDATE notifications SET status = 'failed' WHERE id = ?")
            .bind(&id)
            .execute(&pool)
            .await
            .unwrap();

        let retried = super::retry_now_by_id(&pool, &id, now).await.unwrap();
        assert!(retried);
        let status: String = sqlx::query_scalar("SELECT status FROM notifications WHERE id = ?")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, "queued");
    }
}
