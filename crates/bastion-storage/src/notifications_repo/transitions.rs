use sqlx::SqlitePool;

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

pub async fn retry_now_by_id(db: &SqlitePool, id: &str, now: i64) -> Result<bool, anyhow::Error> {
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
