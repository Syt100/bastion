use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Restore,
    Verify,
}

impl OperationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Restore => "restore",
            Self::Verify => "verify",
        }
    }
}

impl std::str::FromStr for OperationKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "restore" => Ok(Self::Restore),
            "verify" => Ok(Self::Verify),
            _ => Err(anyhow::anyhow!("invalid operation kind")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    Running,
    Success,
    Failed,
}

impl OperationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Success => "success",
            Self::Failed => "failed",
        }
    }
}

impl std::str::FromStr for OperationStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "running" => Ok(Self::Running),
            "success" => Ok(Self::Success),
            "failed" => Ok(Self::Failed),
            _ => Err(anyhow::anyhow!("invalid operation status")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Operation {
    pub id: String,
    pub kind: OperationKind,
    pub status: OperationStatus,
    pub created_at: i64,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub summary: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperationEvent {
    pub op_id: String,
    pub seq: i64,
    pub ts: i64,
    pub level: String,
    pub kind: String,
    pub message: String,
    pub fields: Option<serde_json::Value>,
}

pub async fn create_operation(
    db: &SqlitePool,
    kind: OperationKind,
) -> Result<Operation, anyhow::Error> {
    let id = Uuid::new_v4().to_string();
    let now = OffsetDateTime::now_utc().unix_timestamp();
    sqlx::query(
        "INSERT INTO operations (id, kind, status, created_at, started_at) VALUES (?, ?, 'running', ?, ?)",
    )
    .bind(&id)
    .bind(kind.as_str())
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    Ok(Operation {
        id,
        kind,
        status: OperationStatus::Running,
        created_at: now,
        started_at: now,
        ended_at: None,
        summary: None,
        error: None,
    })
}

pub async fn get_operation(
    db: &SqlitePool,
    op_id: &str,
) -> Result<Option<Operation>, anyhow::Error> {
    let row = sqlx::query(
        "SELECT id, kind, status, created_at, started_at, ended_at, summary_json, error FROM operations WHERE id = ? LIMIT 1",
    )
    .bind(op_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let kind = row.get::<String, _>("kind").parse::<OperationKind>()?;
    let status = row.get::<String, _>("status").parse::<OperationStatus>()?;
    let summary_json = row.get::<Option<String>, _>("summary_json");
    let summary = match summary_json {
        Some(s) => Some(serde_json::from_str::<serde_json::Value>(&s)?),
        None => None,
    };

    Ok(Some(Operation {
        id: row.get::<String, _>("id"),
        kind,
        status,
        created_at: row.get::<i64, _>("created_at"),
        started_at: row.get::<i64, _>("started_at"),
        ended_at: row.get::<Option<i64>, _>("ended_at"),
        summary,
        error: row.get::<Option<String>, _>("error"),
    }))
}

pub async fn append_event(
    db: &SqlitePool,
    op_id: &str,
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
    let row = sqlx::query(
        "SELECT COALESCE(MAX(seq), 0) AS max_seq FROM operation_events WHERE op_id = ?",
    )
    .bind(op_id)
    .fetch_one(&mut *tx)
    .await?;

    let max_seq = row.get::<i64, _>("max_seq");
    let seq = max_seq + 1;

    sqlx::query(
        "INSERT INTO operation_events (op_id, seq, ts, level, kind, message, fields_json) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(op_id)
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

pub async fn list_events(
    db: &SqlitePool,
    op_id: &str,
    limit: u32,
) -> Result<Vec<OperationEvent>, anyhow::Error> {
    let rows = sqlx::query(
        "SELECT op_id, seq, ts, level, kind, message, fields_json FROM operation_events WHERE op_id = ? ORDER BY seq ASC LIMIT ?",
    )
    .bind(op_id)
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

        events.push(OperationEvent {
            op_id: row.get::<String, _>("op_id"),
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

pub async fn complete_operation(
    db: &SqlitePool,
    op_id: &str,
    status: OperationStatus,
    summary: Option<serde_json::Value>,
    error: Option<&str>,
) -> Result<(), anyhow::Error> {
    let ended_at = OffsetDateTime::now_utc().unix_timestamp();
    let summary_json = match summary {
        Some(v) => Some(serde_json::to_string(&v)?),
        None => None,
    };

    sqlx::query(
        "UPDATE operations SET status = ?, ended_at = ?, summary_json = ?, error = ? WHERE id = ?",
    )
    .bind(status.as_str())
    .bind(ended_at)
    .bind(summary_json)
    .bind(error)
    .bind(op_id)
    .execute(db)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;

    use super::{
        OperationKind, OperationStatus, append_event, complete_operation, create_operation,
        get_operation, list_events,
    };

    #[tokio::test]
    async fn operations_and_events_round_trip() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        let op = create_operation(&pool, OperationKind::Verify)
            .await
            .expect("create");
        append_event(&pool, &op.id, "info", "start", "start", None)
            .await
            .expect("event1");
        append_event(
            &pool,
            &op.id,
            "info",
            "step",
            "step",
            Some(serde_json::json!({"n": 1})),
        )
        .await
        .expect("event2");

        complete_operation(&pool, &op.id, OperationStatus::Success, None, None)
            .await
            .expect("complete");

        let fetched = get_operation(&pool, &op.id)
            .await
            .expect("get")
            .expect("present");
        assert_eq!(fetched.kind, OperationKind::Verify);
        assert_eq!(fetched.status, OperationStatus::Success);

        let events = list_events(&pool, &op.id, 100).await.expect("list");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].seq, 1);
        assert_eq!(events[1].seq, 2);
    }
}
