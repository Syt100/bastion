use sqlx::Row;
use sqlx::SqlitePool;

use super::types::{CleanupTaskDetail, CleanupTaskListItem};

pub async fn get_task(
    db: &SqlitePool,
    run_id: &str,
) -> Result<Option<CleanupTaskDetail>, anyhow::Error> {
    let row = sqlx::query(
        r#"
        SELECT
          t.run_id,
          t.job_id,
          j.name AS job_name,
          t.node_id,
          t.target_type,
          t.target_snapshot_json,
          t.status,
          t.attempts,
          t.created_at,
          t.updated_at,
          t.last_attempt_at,
          t.next_attempt_at,
          t.last_error_kind,
          t.last_error,
          t.ignored_at,
          t.ignored_by_user_id,
          t.ignore_reason
        FROM incomplete_cleanup_tasks t
        JOIN jobs j ON j.id = t.job_id
        WHERE t.run_id = ?
        LIMIT 1
        "#,
    )
    .bind(run_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let snapshot_json = row.get::<String, _>("target_snapshot_json");
    let snapshot = serde_json::from_str::<serde_json::Value>(&snapshot_json)?;

    Ok(Some(CleanupTaskDetail {
        run_id: row.get::<String, _>("run_id"),
        job_id: row.get::<String, _>("job_id"),
        job_name: row.get::<String, _>("job_name"),
        node_id: row.get::<String, _>("node_id"),
        target_type: row.get::<String, _>("target_type"),
        target_snapshot: snapshot,
        status: row.get::<String, _>("status"),
        attempts: row.get::<i64, _>("attempts"),
        created_at: row.get::<i64, _>("created_at"),
        updated_at: row.get::<i64, _>("updated_at"),
        last_attempt_at: row.get::<Option<i64>, _>("last_attempt_at"),
        next_attempt_at: row.get::<i64, _>("next_attempt_at"),
        last_error_kind: row.get::<Option<String>, _>("last_error_kind"),
        last_error: row.get::<Option<String>, _>("last_error"),
        ignored_at: row.get::<Option<i64>, _>("ignored_at"),
        ignored_by_user_id: row.get::<Option<i64>, _>("ignored_by_user_id"),
        ignore_reason: row.get::<Option<String>, _>("ignore_reason"),
    }))
}

pub async fn list_tasks(
    db: &SqlitePool,
    status: Option<&str>,
    target_type: Option<&str>,
    node_id: Option<&str>,
    job_id: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<CleanupTaskListItem>, anyhow::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
          t.run_id,
          t.job_id,
          j.name AS job_name,
          t.node_id,
          t.target_type,
          t.status,
          t.attempts,
          t.last_attempt_at,
          t.next_attempt_at,
          t.created_at,
          t.updated_at,
          t.last_error_kind,
          t.last_error
        FROM incomplete_cleanup_tasks t
        JOIN jobs j ON j.id = t.job_id
        WHERE (? IS NULL OR t.status = ?)
          AND (? IS NULL OR t.target_type = ?)
          AND (? IS NULL OR t.node_id = ?)
          AND (? IS NULL OR t.job_id = ?)
        ORDER BY t.next_attempt_at ASC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(status)
    .bind(status)
    .bind(target_type)
    .bind(target_type)
    .bind(node_id)
    .bind(node_id)
    .bind(job_id)
    .bind(job_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(CleanupTaskListItem {
            run_id: row.get::<String, _>("run_id"),
            job_id: row.get::<String, _>("job_id"),
            job_name: row.get::<String, _>("job_name"),
            node_id: row.get::<String, _>("node_id"),
            target_type: row.get::<String, _>("target_type"),
            status: row.get::<String, _>("status"),
            attempts: row.get::<i64, _>("attempts"),
            last_attempt_at: row.get::<Option<i64>, _>("last_attempt_at"),
            next_attempt_at: row.get::<i64, _>("next_attempt_at"),
            created_at: row.get::<i64, _>("created_at"),
            updated_at: row.get::<i64, _>("updated_at"),
            last_error_kind: row.get::<Option<String>, _>("last_error_kind"),
            last_error: row.get::<Option<String>, _>("last_error"),
        });
    }

    Ok(out)
}

pub async fn count_tasks(
    db: &SqlitePool,
    status: Option<&str>,
    target_type: Option<&str>,
    node_id: Option<&str>,
    job_id: Option<&str>,
) -> Result<i64, anyhow::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(1) FROM incomplete_cleanup_tasks
        WHERE (? IS NULL OR status = ?)
          AND (? IS NULL OR target_type = ?)
          AND (? IS NULL OR node_id = ?)
          AND (? IS NULL OR job_id = ?)
        "#,
    )
    .bind(status)
    .bind(status)
    .bind(target_type)
    .bind(target_type)
    .bind(node_id)
    .bind(node_id)
    .bind(job_id)
    .bind(job_id)
    .fetch_one(db)
    .await?;
    Ok(count)
}
