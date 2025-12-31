use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Queued,
    Running,
    Success,
    Failed,
    Rejected,
}

impl RunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Rejected => "rejected",
        }
    }
}

impl std::str::FromStr for RunStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "success" => Ok(Self::Success),
            "failed" => Ok(Self::Failed),
            "rejected" => Ok(Self::Rejected),
            _ => Err(anyhow::anyhow!("invalid run status")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Run {
    pub id: String,
    pub job_id: String,
    pub status: RunStatus,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub summary: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunEvent {
    pub run_id: String,
    pub seq: i64,
    pub ts: i64,
    pub level: String,
    pub kind: String,
    pub message: String,
    pub fields: Option<serde_json::Value>,
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

pub async fn append_run_event(
    db: &SqlitePool,
    run_id: &str,
    level: &str,
    kind: &str,
    message: &str,
    fields: Option<serde_json::Value>,
) -> Result<i64, anyhow::Error> {
    let fields_json = match fields {
        Some(v) => Some(serde_json::to_string(&v)?),
        None => None,
    };
    let ts = OffsetDateTime::now_utc().unix_timestamp();

    let mut tx = db.begin().await?;
    let row =
        sqlx::query("SELECT COALESCE(MAX(seq), 0) AS max_seq FROM run_events WHERE run_id = ?")
            .bind(run_id)
            .fetch_one(&mut *tx)
            .await?;

    let max_seq = row.get::<i64, _>("max_seq");
    let seq = max_seq + 1;

    sqlx::query(
        r#"
        INSERT INTO run_events (run_id, seq, ts, level, kind, message, fields_json)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(run_id)
    .bind(seq)
    .bind(ts)
    .bind(level)
    .bind(kind)
    .bind(message)
    .bind(fields_json)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(seq)
}

pub async fn list_run_events(
    db: &SqlitePool,
    run_id: &str,
    limit: u32,
) -> Result<Vec<RunEvent>, anyhow::Error> {
    let rows = sqlx::query(
        "SELECT run_id, seq, ts, level, kind, message, fields_json FROM run_events WHERE run_id = ? ORDER BY seq ASC LIMIT ?",
    )
    .bind(run_id)
    .bind(limit as i64)
    .fetch_all(db)
    .await?;

    let mut events = Vec::with_capacity(rows.len());
    for row in rows {
        let fields_json = row.get::<Option<String>, _>("fields_json");
        let fields = match fields_json {
            Some(s) => Some(serde_json::from_str::<serde_json::Value>(&s)?),
            None => None,
        };

        events.push(RunEvent {
            run_id: row.get::<String, _>("run_id"),
            seq: row.get::<i64, _>("seq"),
            ts: row.get::<i64, _>("ts"),
            level: row.get::<String, _>("level"),
            kind: row.get::<String, _>("kind"),
            message: row.get::<String, _>("message"),
            fields,
        });
    }

    Ok(events)
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

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;

    use super::{
        RunStatus, append_run_event, claim_next_queued_run, complete_run, create_run,
        list_run_events, list_runs_for_job, prune_runs_ended_before,
    };

    #[tokio::test]
    async fn runs_and_events_round_trip() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        sqlx::query(
            "INSERT INTO jobs (id, name, schedule, overlap_policy, spec_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("job1")
        .bind("job1")
        .bind(None::<String>)
        .bind("queue")
        .bind(r#"{"v":1,"type":"filesystem"}"#)
        .bind(1000)
        .bind(1000)
        .execute(&pool)
        .await
        .expect("insert job");

        let run = create_run(&pool, "job1", RunStatus::Queued, 1000, None, None, None)
            .await
            .expect("create run");

        append_run_event(&pool, &run.id, "info", "queued", "queued", None)
            .await
            .expect("event1");
        append_run_event(&pool, &run.id, "info", "start", "start", None)
            .await
            .expect("event2");

        let runs = list_runs_for_job(&pool, "job1", 10)
            .await
            .expect("list runs");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status, RunStatus::Queued);

        let claimed = claim_next_queued_run(&pool)
            .await
            .expect("claim")
            .expect("claimed");
        assert_eq!(claimed.status, RunStatus::Running);

        complete_run(&pool, &claimed.id, RunStatus::Success, None, None)
            .await
            .expect("complete");

        let events = list_run_events(&pool, &run.id, 100)
            .await
            .expect("list events");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].seq, 1);
        assert_eq!(events[1].seq, 2);

        let pruned = prune_runs_ended_before(&pool, 0).await.expect("prune");
        assert_eq!(pruned, 0);
    }
}
