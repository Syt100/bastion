use sqlx::Row;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::{CHANNEL_EMAIL, CHANNEL_WECOM_BOT};

pub async fn enqueue_wecom_bots_for_run(
    db: &SqlitePool,
    run_id: &str,
) -> Result<i64, anyhow::Error> {
    let bots = sqlx::query(
        "SELECT name FROM secrets WHERE node_id = 'hub' AND kind = ? ORDER BY updated_at DESC",
    )
    .bind("wecom_bot")
    .fetch_all(db)
    .await?;

    if bots.is_empty() {
        return Ok(0);
    }

    let names = bots
        .into_iter()
        .map(|r| r.get::<String, _>("name"))
        .collect::<Vec<_>>();
    enqueue_for_run(db, run_id, CHANNEL_WECOM_BOT, &names).await
}

pub async fn enqueue_emails_for_run(db: &SqlitePool, run_id: &str) -> Result<i64, anyhow::Error> {
    let destinations = sqlx::query(
        "SELECT name FROM secrets WHERE node_id = 'hub' AND kind = ? ORDER BY updated_at DESC",
    )
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
