use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{Run, RunStatus};

fn parse_run_row(row: &sqlx::sqlite::SqliteRow) -> Result<Run, anyhow::Error> {
    let status = row.get::<String, _>("status").parse::<RunStatus>()?;
    let progress_json = row.get::<Option<String>, _>("progress_json");
    let progress = match progress_json {
        Some(s) => Some(serde_json::from_str::<serde_json::Value>(&s)?),
        None => None,
    };
    let summary_json = row.get::<Option<String>, _>("summary_json");
    let summary = match summary_json {
        Some(s) => Some(serde_json::from_str::<serde_json::Value>(&s)?),
        None => None,
    };

    Ok(Run {
        id: row.get::<String, _>("id"),
        job_id: row.get::<String, _>("job_id"),
        status,
        started_at: row.get::<i64, _>("started_at"),
        ended_at: row.get::<Option<i64>, _>("ended_at"),
        cancel_requested_at: row.get::<Option<i64>, _>("cancel_requested_at"),
        cancel_requested_by_user_id: row.get::<Option<i64>, _>("cancel_requested_by_user_id"),
        cancel_reason: row.get::<Option<String>, _>("cancel_reason"),
        progress,
        summary,
        error: row.get::<Option<String>, _>("error"),
    })
}

pub async fn create_run(
    db: &SqlitePool,
    job_id: &str,
    status: RunStatus,
    started_at: i64,
    ended_at: Option<i64>,
    summary: Option<serde_json::Value>,
    error: Option<&str>,
) -> Result<Run, anyhow::Error> {
    let id = Uuid::new_v4().to_string();
    let summary_json = match &summary {
        Some(v) => Some(serde_json::to_string(v)?),
        None => None,
    };

    sqlx::query(
        r#"
        INSERT INTO runs (id, job_id, status, started_at, ended_at, summary_json, error)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(job_id)
    .bind(status.as_str())
    .bind(started_at)
    .bind(ended_at)
    .bind(summary_json)
    .bind(error)
    .execute(db)
    .await?;

    Ok(Run {
        id,
        job_id: job_id.to_string(),
        status,
        started_at,
        ended_at,
        cancel_requested_at: None,
        cancel_requested_by_user_id: None,
        cancel_reason: None,
        progress: None,
        summary,
        error: error.map(|s| s.to_string()),
    })
}

pub async fn list_runs_for_job(
    db: &SqlitePool,
    job_id: &str,
    limit: u32,
) -> Result<Vec<Run>, anyhow::Error> {
    let rows = sqlx::query(
        "SELECT id, job_id, status, started_at, ended_at, cancel_requested_at, cancel_requested_by_user_id, cancel_reason, progress_json, summary_json, error FROM runs WHERE job_id = ? ORDER BY started_at DESC LIMIT ?",
    )
    .bind(job_id)
    .bind(limit as i64)
    .fetch_all(db)
    .await?;

    let mut runs = Vec::with_capacity(rows.len());
    for row in rows {
        runs.push(parse_run_row(&row)?);
    }

    Ok(runs)
}

pub async fn get_run(db: &SqlitePool, run_id: &str) -> Result<Option<Run>, anyhow::Error> {
    let row = sqlx::query(
        "SELECT id, job_id, status, started_at, ended_at, cancel_requested_at, cancel_requested_by_user_id, cancel_reason, progress_json, summary_json, error FROM runs WHERE id = ? LIMIT 1",
    )
    .bind(run_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(parse_run_row(&row)?))
}

pub async fn get_run_target_snapshot(
    db: &SqlitePool,
    run_id: &str,
) -> Result<Option<serde_json::Value>, anyhow::Error> {
    let row = sqlx::query("SELECT target_snapshot_json FROM runs WHERE id = ? LIMIT 1")
        .bind(run_id)
        .fetch_optional(db)
        .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let snapshot_json = row.get::<Option<String>, _>("target_snapshot_json");
    let snapshot = snapshot_json
        .map(|s| serde_json::from_str::<serde_json::Value>(&s))
        .transpose()?;
    Ok(snapshot)
}

pub async fn get_run_progress(
    db: &SqlitePool,
    run_id: &str,
) -> Result<Option<serde_json::Value>, anyhow::Error> {
    let row = sqlx::query("SELECT progress_json FROM runs WHERE id = ? LIMIT 1")
        .bind(run_id)
        .fetch_optional(db)
        .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let progress_json = row.get::<Option<String>, _>("progress_json");
    let progress = progress_json
        .map(|s| serde_json::from_str::<serde_json::Value>(&s))
        .transpose()?;
    Ok(progress)
}

pub async fn set_run_progress(
    db: &SqlitePool,
    run_id: &str,
    progress: Option<serde_json::Value>,
) -> Result<bool, anyhow::Error> {
    let progress_json = match progress {
        Some(v) => Some(serde_json::to_string(&v)?),
        None => None,
    };

    let result = sqlx::query("UPDATE runs SET progress_json = ? WHERE id = ?")
        .bind(progress_json)
        .bind(run_id)
        .execute(db)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn set_run_target_snapshot(
    db: &SqlitePool,
    run_id: &str,
    snapshot: serde_json::Value,
) -> Result<bool, anyhow::Error> {
    let snapshot_json = serde_json::to_string(&snapshot)?;

    let result = sqlx::query(
        "UPDATE runs SET target_snapshot_json = ? WHERE id = ? AND target_snapshot_json IS NULL",
    )
    .bind(snapshot_json)
    .bind(run_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn claim_next_queued_run(db: &SqlitePool) -> Result<Option<Run>, anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let mut tx = db.begin().await?;
    loop {
        let row = sqlx::query(
            "SELECT id, job_id FROM runs WHERE status = 'queued' AND cancel_requested_at IS NULL ORDER BY started_at ASC LIMIT 1",
        )
        .fetch_optional(&mut *tx)
        .await?;

        let Some(row) = row else {
            tx.commit().await?;
            return Ok(None);
        };

        let run_id = row.get::<String, _>("id");
        let job_id = row.get::<String, _>("job_id");

        let result = sqlx::query(
            "UPDATE runs SET status = 'running', started_at = ? WHERE id = ? AND status = 'queued' AND cancel_requested_at IS NULL",
        )
        .bind(now)
        .bind(&run_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            continue;
        }

        tx.commit().await?;

        return Ok(Some(Run {
            id: run_id,
            job_id,
            status: RunStatus::Running,
            started_at: now,
            ended_at: None,
            cancel_requested_at: None,
            cancel_requested_by_user_id: None,
            cancel_reason: None,
            progress: None,
            summary: None,
            error: None,
        }));
    }
}

pub async fn complete_run(
    db: &SqlitePool,
    run_id: &str,
    status: RunStatus,
    summary: Option<serde_json::Value>,
    error: Option<&str>,
) -> Result<bool, anyhow::Error> {
    let ended_at = OffsetDateTime::now_utc().unix_timestamp();
    let summary_json = match summary {
        Some(v) => Some(serde_json::to_string(&v)?),
        None => None,
    };

    let result = sqlx::query(
        "UPDATE runs
         SET status = CASE
             WHEN cancel_requested_at IS NOT NULL THEN 'canceled'
             ELSE ?
         END,
             ended_at = ?,
             summary_json = CASE
                 WHEN cancel_requested_at IS NOT NULL THEN NULL
                 ELSE ?
             END,
             error = CASE
                 WHEN cancel_requested_at IS NOT NULL THEN COALESCE(error, 'canceled')
                 ELSE ?
             END
         WHERE id = ? AND status IN ('running', 'queued')",
    )
    .bind(status.as_str())
    .bind(ended_at)
    .bind(summary_json)
    .bind(error)
    .bind(run_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn request_run_cancel(
    db: &SqlitePool,
    run_id: &str,
    requested_by_user_id: i64,
    reason: Option<&str>,
) -> Result<Option<Run>, anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let reason = reason.map(str::trim).filter(|v| !v.is_empty());

    let _ = sqlx::query(
        "UPDATE runs
         SET cancel_requested_at = COALESCE(cancel_requested_at, ?),
             cancel_requested_by_user_id = COALESCE(cancel_requested_by_user_id, ?),
             cancel_reason = COALESCE(cancel_reason, ?),
             status = CASE
                 WHEN status = 'queued' THEN 'canceled'
                 ELSE status
             END,
             ended_at = CASE
                 WHEN status = 'queued' THEN COALESCE(ended_at, ?)
                 ELSE ended_at
             END,
             error = CASE
                 WHEN status = 'queued' THEN COALESCE(error, 'canceled')
                 ELSE error
             END
         WHERE id = ?
           AND status IN ('queued', 'running')",
    )
    .bind(now)
    .bind(requested_by_user_id)
    .bind(reason)
    .bind(now)
    .bind(run_id)
    .execute(db)
    .await?;

    get_run(db, run_id).await
}

pub async fn requeue_run(db: &SqlitePool, run_id: &str) -> Result<(), anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();

    sqlx::query(
        "UPDATE runs
         SET status = 'queued',
             started_at = ?,
             ended_at = NULL,
             cancel_requested_at = NULL,
             cancel_requested_by_user_id = NULL,
             cancel_reason = NULL,
             progress_json = NULL,
             summary_json = NULL,
             error = NULL
         WHERE id = ?",
    )
    .bind(now)
    .bind(run_id)
    .execute(db)
    .await?;

    Ok(())
}
