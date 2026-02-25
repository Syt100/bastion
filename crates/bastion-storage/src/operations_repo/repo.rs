use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::types::{Operation, OperationEvent, OperationKind, OperationStatus};

fn parse_operation_row(row: &sqlx::sqlite::SqliteRow) -> Result<Operation, anyhow::Error> {
    let kind = row.get::<String, _>("kind").parse::<OperationKind>()?;
    let status = row.get::<String, _>("status").parse::<OperationStatus>()?;
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

    Ok(Operation {
        id: row.get::<String, _>("id"),
        kind,
        status,
        created_at: row.get::<i64, _>("created_at"),
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

pub async fn create_operation(
    db: &SqlitePool,
    kind: OperationKind,
    subject: Option<(&str, &str)>,
) -> Result<Operation, anyhow::Error> {
    let id = Uuid::new_v4().to_string();
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let subject = subject.and_then(|(k, id)| {
        let k = k.trim();
        let id = id.trim();
        if k.is_empty() || id.is_empty() {
            None
        } else {
            Some((k, id))
        }
    });
    let (subject_kind, subject_id) = match subject {
        Some((k, id)) => (Some(k), Some(id)),
        None => (None, None),
    };

    sqlx::query(
        "INSERT INTO operations (id, kind, status, created_at, started_at, subject_kind, subject_id) VALUES (?, ?, 'running', ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(kind.as_str())
    .bind(now)
    .bind(now)
    .bind(subject_kind)
    .bind(subject_id)
    .execute(db)
    .await?;

    Ok(Operation {
        id,
        kind,
        status: OperationStatus::Running,
        created_at: now,
        started_at: now,
        ended_at: None,
        cancel_requested_at: None,
        cancel_requested_by_user_id: None,
        cancel_reason: None,
        progress: None,
        summary: None,
        error: None,
    })
}

pub async fn get_operation(
    db: &SqlitePool,
    op_id: &str,
) -> Result<Option<Operation>, anyhow::Error> {
    let row = sqlx::query(
        "SELECT id, kind, status, created_at, started_at, ended_at, cancel_requested_at, cancel_requested_by_user_id, cancel_reason, progress_json, summary_json, error FROM operations WHERE id = ? LIMIT 1",
    )
    .bind(op_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    Ok(Some(parse_operation_row(&row)?))
}

pub async fn list_operations_by_subject(
    db: &SqlitePool,
    subject_kind: &str,
    subject_id: &str,
    limit: u32,
) -> Result<Vec<Operation>, anyhow::Error> {
    let rows = sqlx::query(
        "SELECT id, kind, status, created_at, started_at, ended_at, cancel_requested_at, cancel_requested_by_user_id, cancel_reason, progress_json, summary_json, error FROM operations WHERE subject_kind = ? AND subject_id = ? ORDER BY started_at DESC, id DESC LIMIT ?",
    )
    .bind(subject_kind)
    .bind(subject_id)
    .bind(limit as i64)
    .fetch_all(db)
    .await?;

    let mut ops = Vec::with_capacity(rows.len());
    for row in rows {
        ops.push(parse_operation_row(&row)?);
    }
    Ok(ops)
}

pub async fn set_operation_progress(
    db: &SqlitePool,
    op_id: &str,
    progress: Option<serde_json::Value>,
) -> Result<(), anyhow::Error> {
    let progress_json = match progress {
        Some(v) => Some(serde_json::to_string(&v)?),
        None => None,
    };

    sqlx::query("UPDATE operations SET progress_json = ? WHERE id = ?")
        .bind(progress_json)
        .bind(op_id)
        .execute(db)
        .await?;

    Ok(())
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
) -> Result<bool, anyhow::Error> {
    let ended_at = OffsetDateTime::now_utc().unix_timestamp();
    let summary_json = match summary {
        Some(v) => Some(serde_json::to_string(&v)?),
        None => None,
    };

    let result = sqlx::query(
        "UPDATE operations
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
         WHERE id = ? AND status = 'running'",
    )
    .bind(status.as_str())
    .bind(ended_at)
    .bind(summary_json)
    .bind(error)
    .bind(op_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn request_operation_cancel(
    db: &SqlitePool,
    op_id: &str,
    requested_by_user_id: i64,
    reason: Option<&str>,
) -> Result<Option<Operation>, anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let reason = reason.map(str::trim).filter(|v| !v.is_empty());

    let _ = sqlx::query(
        "UPDATE operations
         SET cancel_requested_at = COALESCE(cancel_requested_at, ?),
             cancel_requested_by_user_id = COALESCE(cancel_requested_by_user_id, ?),
             cancel_reason = COALESCE(cancel_reason, ?)
         WHERE id = ?
           AND status = 'running'",
    )
    .bind(now)
    .bind(requested_by_user_id)
    .bind(reason)
    .bind(op_id)
    .execute(db)
    .await?;

    get_operation(db, op_id).await
}
