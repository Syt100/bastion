use sqlx::{Row, SqlitePool};

use super::{IncompleteCleanupRun, RunStatus};

pub async fn prune_runs_ended_before(
    db: &SqlitePool,
    cutoff_ts: i64,
) -> Result<u64, anyhow::Error> {
    // Snapshot-aware pruning: keep run history as long as a "live" snapshot exists.
    //
    // We use a correlated subquery so SQLite can use the `run_artifacts.run_id` primary key
    // instead of scanning `run_artifacts` by status.
    let result = sqlx::query(
        r#"
        DELETE FROM runs
        WHERE ended_at IS NOT NULL
          AND ended_at < ?
          AND NOT EXISTS (
            SELECT 1 FROM run_artifacts a
            WHERE a.run_id = runs.id
              AND a.status IN ('present', 'deleting', 'error')
          )
        "#,
    )
    .bind(cutoff_ts)
    .execute(db)
    .await?;
    Ok(result.rows_affected())
}

pub async fn list_incomplete_cleanup_candidates(
    db: &SqlitePool,
    cutoff_started_at: i64,
    limit: u32,
) -> Result<Vec<IncompleteCleanupRun>, anyhow::Error> {
    let rows = sqlx::query(
        r#"
        SELECT r.id, r.job_id, r.status, r.started_at
        FROM runs r
        LEFT JOIN incomplete_cleanup_tasks t ON t.run_id = r.id
        WHERE t.run_id IS NULL
          AND r.status != 'success'
          AND r.started_at < ?
        ORDER BY r.started_at ASC
        LIMIT ?
        "#,
    )
    .bind(cutoff_started_at)
    .bind(limit as i64)
    .fetch_all(db)
    .await?;

    let mut runs = Vec::with_capacity(rows.len());
    for row in rows {
        let status = row.get::<String, _>("status").parse::<RunStatus>()?;
        runs.push(IncompleteCleanupRun {
            id: row.get::<String, _>("id"),
            job_id: row.get::<String, _>("job_id"),
            status,
            started_at: row.get::<i64, _>("started_at"),
        });
    }

    Ok(runs)
}
