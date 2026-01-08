use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{Run, RunStatus};

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
        "SELECT id, job_id, status, started_at, ended_at, summary_json, error FROM runs WHERE job_id = ? ORDER BY started_at DESC LIMIT ?",
    )
    .bind(job_id)
    .bind(limit as i64)
    .fetch_all(db)
    .await?;

    let mut runs = Vec::with_capacity(rows.len());
    for row in rows {
        let status = row.get::<String, _>("status").parse::<RunStatus>()?;
        let summary_json = row.get::<Option<String>, _>("summary_json");
        let summary = match summary_json {
            Some(s) => Some(serde_json::from_str::<serde_json::Value>(&s)?),
            None => None,
        };

        runs.push(Run {
            id: row.get::<String, _>("id"),
            job_id: row.get::<String, _>("job_id"),
            status,
            started_at: row.get::<i64, _>("started_at"),
            ended_at: row.get::<Option<i64>, _>("ended_at"),
            summary,
            error: row.get::<Option<String>, _>("error"),
        });
    }

    Ok(runs)
}

pub async fn get_run(db: &SqlitePool, run_id: &str) -> Result<Option<Run>, anyhow::Error> {
    let row = sqlx::query(
        "SELECT id, job_id, status, started_at, ended_at, summary_json, error FROM runs WHERE id = ? LIMIT 1",
    )
    .bind(run_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let status = row.get::<String, _>("status").parse::<RunStatus>()?;
    let summary_json = row.get::<Option<String>, _>("summary_json");
    let summary = match summary_json {
        Some(s) => Some(serde_json::from_str::<serde_json::Value>(&s)?),
        None => None,
    };

    Ok(Some(Run {
        id: row.get::<String, _>("id"),
        job_id: row.get::<String, _>("job_id"),
        status,
        started_at: row.get::<i64, _>("started_at"),
        ended_at: row.get::<Option<i64>, _>("ended_at"),
        summary,
        error: row.get::<Option<String>, _>("error"),
    }))
}

pub async fn claim_next_queued_run(db: &SqlitePool) -> Result<Option<Run>, anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let mut tx = db.begin().await?;
    let row = sqlx::query(
        "SELECT id, job_id FROM runs WHERE status = 'queued' ORDER BY started_at ASC LIMIT 1",
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(row) = row else {
        tx.commit().await?;
        return Ok(None);
    };

    let run_id = row.get::<String, _>("id");
    let job_id = row.get::<String, _>("job_id");

    sqlx::query("UPDATE runs SET status = 'running', started_at = ? WHERE id = ?")
        .bind(now)
        .bind(&run_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Some(Run {
        id: run_id,
        job_id,
        status: RunStatus::Running,
        started_at: now,
        ended_at: None,
        summary: None,
        error: None,
    }))
}

pub async fn complete_run(
    db: &SqlitePool,
    run_id: &str,
    status: RunStatus,
    summary: Option<serde_json::Value>,
    error: Option<&str>,
) -> Result<(), anyhow::Error> {
    let ended_at = OffsetDateTime::now_utc().unix_timestamp();
    let summary_json = match summary {
        Some(v) => Some(serde_json::to_string(&v)?),
        None => None,
    };

    sqlx::query(
        "UPDATE runs SET status = ?, ended_at = ?, summary_json = ?, error = ? WHERE id = ?",
    )
    .bind(status.as_str())
    .bind(ended_at)
    .bind(summary_json)
    .bind(error)
    .bind(run_id)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn requeue_run(db: &SqlitePool, run_id: &str) -> Result<(), anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();

    sqlx::query(
        "UPDATE runs SET status = 'queued', started_at = ?, ended_at = NULL, summary_json = NULL, error = NULL WHERE id = ?",
    )
    .bind(now)
    .bind(run_id)
    .execute(db)
    .await?;

    Ok(())
}
