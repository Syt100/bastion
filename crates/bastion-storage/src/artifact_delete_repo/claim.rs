use sqlx::Row;
use sqlx::SqlitePool;

use super::types::{ArtifactDeleteTaskRow, DeleteTargetType};

pub async fn claim_next_due(
    db: &SqlitePool,
    now: i64,
) -> Result<Option<ArtifactDeleteTaskRow>, anyhow::Error> {
    let row = sqlx::query(
        r#"
        UPDATE artifact_delete_tasks
        SET status = 'running', attempts = attempts + 1, last_attempt_at = ?, updated_at = ?
        WHERE run_id = (
          SELECT run_id FROM artifact_delete_tasks
          WHERE status IN ('queued', 'retrying', 'blocked') AND next_attempt_at <= ?
          ORDER BY next_attempt_at ASC
          LIMIT 1
        )
        RETURNING run_id, job_id, node_id, target_type, target_snapshot_json, attempts, created_at
        "#,
    )
    .bind(now)
    .bind(now)
    .bind(now)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let target_type = row
        .get::<String, _>("target_type")
        .parse::<DeleteTargetType>()?;
    let target_snapshot_json = row.get::<String, _>("target_snapshot_json");
    let target_snapshot = serde_json::from_str::<serde_json::Value>(&target_snapshot_json)?;

    Ok(Some(ArtifactDeleteTaskRow {
        run_id: row.get::<String, _>("run_id"),
        job_id: row.get::<String, _>("job_id"),
        node_id: row.get::<String, _>("node_id"),
        target_type,
        target_snapshot,
        attempts: row.get::<i64, _>("attempts"),
        created_at: row.get::<i64, _>("created_at"),
    }))
}

pub async fn next_due_at(db: &SqlitePool) -> Result<Option<i64>, anyhow::Error> {
    let row = sqlx::query(
        "SELECT next_attempt_at FROM artifact_delete_tasks WHERE status IN ('queued', 'retrying', 'blocked') ORDER BY next_attempt_at ASC LIMIT 1",
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| r.get::<i64, _>("next_attempt_at")))
}

