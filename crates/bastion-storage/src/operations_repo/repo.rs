use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::types::{Operation, OperationEvent, OperationKind, OperationStatus};

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
