use sqlx::SqlitePool;

pub async fn upsert_task_if_missing(
    db: &SqlitePool,
    run_id: &str,
    job_id: &str,
    node_id: &str,
    target_type: &str,
    target_snapshot_json: &str,
    now: i64,
) -> Result<bool, anyhow::Error> {
    let result = sqlx::query(
        r#"
        INSERT OR IGNORE INTO incomplete_cleanup_tasks (
          run_id, job_id, node_id, target_type, target_snapshot_json,
          status, attempts, created_at, updated_at, next_attempt_at
        )
        VALUES (?, ?, ?, ?, ?, 'queued', 0, ?, ?, ?)
        "#,
    )
    .bind(run_id)
    .bind(job_id)
    .bind(node_id)
    .bind(target_type)
    .bind(target_snapshot_json)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn mark_done(db: &SqlitePool, run_id: &str, now: i64) -> Result<(), anyhow::Error> {
    sqlx::query(
        "UPDATE incomplete_cleanup_tasks SET status = 'done', updated_at = ?, next_attempt_at = ?, last_error_kind = NULL, last_error = NULL WHERE run_id = ?",
    )
    .bind(now)
    .bind(now)
    .bind(run_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn mark_retrying(
    db: &SqlitePool,
    run_id: &str,
    next_attempt_at: i64,
    last_error_kind: &str,
    last_error: &str,
    now: i64,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        "UPDATE incomplete_cleanup_tasks SET status = 'retrying', updated_at = ?, next_attempt_at = ?, last_error_kind = ?, last_error = ? WHERE run_id = ?",
    )
    .bind(now)
    .bind(next_attempt_at)
    .bind(last_error_kind)
    .bind(last_error)
    .bind(run_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn mark_blocked(
    db: &SqlitePool,
    run_id: &str,
    next_attempt_at: i64,
    last_error_kind: &str,
    last_error: &str,
    now: i64,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        "UPDATE incomplete_cleanup_tasks SET status = 'blocked', updated_at = ?, next_attempt_at = ?, last_error_kind = ?, last_error = ? WHERE run_id = ?",
    )
    .bind(now)
    .bind(next_attempt_at)
    .bind(last_error_kind)
    .bind(last_error)
    .bind(run_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn mark_abandoned(
    db: &SqlitePool,
    run_id: &str,
    last_error_kind: &str,
    last_error: &str,
    now: i64,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        "UPDATE incomplete_cleanup_tasks SET status = 'abandoned', updated_at = ?, next_attempt_at = ?, last_error_kind = ?, last_error = ? WHERE run_id = ?",
    )
    .bind(now)
    .bind(now)
    .bind(last_error_kind)
    .bind(last_error)
    .bind(run_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn retry_now(db: &SqlitePool, run_id: &str, now: i64) -> Result<bool, anyhow::Error> {
    let result = sqlx::query(
        "UPDATE incomplete_cleanup_tasks SET status = 'queued', attempts = 0, updated_at = ?, next_attempt_at = ?, last_error_kind = NULL, last_error = NULL WHERE run_id = ? AND status IN ('retrying', 'blocked', 'ignored', 'abandoned')",
    )
    .bind(now)
    .bind(now)
    .bind(run_id)
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn ignore_task(
    db: &SqlitePool,
    run_id: &str,
    ignored_by_user_id: Option<i64>,
    reason: Option<&str>,
    now: i64,
) -> Result<bool, anyhow::Error> {
    let result = sqlx::query(
        "UPDATE incomplete_cleanup_tasks SET status = 'ignored', ignored_at = ?, ignored_by_user_id = ?, ignore_reason = ?, updated_at = ? WHERE run_id = ? AND status NOT IN ('done')",
    )
    .bind(now)
    .bind(ignored_by_user_id)
    .bind(reason)
    .bind(now)
    .bind(run_id)
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn unignore_task(db: &SqlitePool, run_id: &str, now: i64) -> Result<bool, anyhow::Error> {
    let result = sqlx::query(
        "UPDATE incomplete_cleanup_tasks SET status = 'queued', ignored_at = NULL, ignored_by_user_id = NULL, ignore_reason = NULL, updated_at = ?, next_attempt_at = ? WHERE run_id = ? AND status = 'ignored'",
    )
    .bind(now)
    .bind(now)
    .bind(run_id)
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}
