use sqlx::{Row, SqlitePool};

use super::{IncompleteCleanupRun, RunStatus};

pub async fn prune_runs_ended_before(
    db: &SqlitePool,
    cutoff_ts: i64,
) -> Result<u64, anyhow::Error> {
    let result = sqlx::query("DELETE FROM runs WHERE ended_at IS NOT NULL AND ended_at < ?")
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
        "SELECT id, job_id, status, started_at FROM runs WHERE status != 'success' AND started_at < ? ORDER BY started_at ASC LIMIT ?",
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
