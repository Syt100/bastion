use sqlx::Row;
use sqlx::SqlitePool;

use super::types::{ArtifactDeleteTaskDetail, ArtifactDeleteTaskSummary};

pub async fn get_task(
    db: &SqlitePool,
    run_id: &str,
) -> Result<Option<ArtifactDeleteTaskDetail>, anyhow::Error> {
    let row = sqlx::query(
        r#"
        SELECT
          run_id,
          job_id,
          node_id,
          target_type,
          target_snapshot_json,
          status,
          attempts,
          created_at,
          updated_at,
          last_attempt_at,
          next_attempt_at,
          last_error_kind,
          last_error,
          ignored_at,
          ignored_by_user_id,
          ignore_reason
        FROM artifact_delete_tasks
        WHERE run_id = ?
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

    Ok(Some(ArtifactDeleteTaskDetail {
        run_id: row.get::<String, _>("run_id"),
        job_id: row.get::<String, _>("job_id"),
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

pub async fn list_tasks_by_run_ids(
    db: &SqlitePool,
    run_ids: &[String],
) -> Result<Vec<ArtifactDeleteTaskSummary>, anyhow::Error> {
    if run_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut qb = sqlx::QueryBuilder::new(
        r#"
        SELECT
          run_id,
          status,
          attempts,
          last_attempt_at,
          next_attempt_at,
          last_error_kind,
          last_error,
          ignored_at
        FROM artifact_delete_tasks
        WHERE run_id IN (
        "#,
    );
    {
        let mut separated = qb.separated(", ");
        for run_id in run_ids {
            separated.push_bind(run_id);
        }
    }
    qb.push(")");

    let rows = qb.build().fetch_all(db).await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(ArtifactDeleteTaskSummary {
            run_id: row.get::<String, _>("run_id"),
            status: row.get::<String, _>("status"),
            attempts: row.get::<i64, _>("attempts"),
            last_attempt_at: row.get::<Option<i64>, _>("last_attempt_at"),
            next_attempt_at: row.get::<i64, _>("next_attempt_at"),
            last_error_kind: row.get::<Option<String>, _>("last_error_kind"),
            last_error: row.get::<Option<String>, _>("last_error"),
            ignored_at: row.get::<Option<i64>, _>("ignored_at"),
        });
    }

    Ok(out)
}

pub async fn count_retention_enqueues_for_job_since(
    db: &SqlitePool,
    job_id: &str,
    since_ts: i64,
) -> Result<u64, anyhow::Error> {
    let row = sqlx::query(
        r#"
        SELECT COUNT(*) AS cnt
        FROM artifact_delete_events e
        JOIN artifact_delete_tasks t ON t.run_id = e.run_id
        WHERE t.job_id = ?
          AND e.kind = 'retention_queued'
          AND e.ts >= ?
        "#,
    )
    .bind(job_id)
    .bind(since_ts)
    .fetch_one(db)
    .await?;

    let cnt = row.get::<i64, _>("cnt");
    Ok(u64::try_from(cnt).unwrap_or(0))
}
