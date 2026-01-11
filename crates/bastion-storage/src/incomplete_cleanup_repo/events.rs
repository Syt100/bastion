use sqlx::{Row, SqlitePool};

use super::types::CleanupEvent;

pub async fn append_event(
    db: &SqlitePool,
    run_id: &str,
    level: &str,
    kind: &str,
    message: &str,
    fields: Option<serde_json::Value>,
    ts: i64,
) -> Result<CleanupEvent, anyhow::Error> {
    let fields_json = fields.as_ref().map(serde_json::to_string).transpose()?;

    let mut tx = db.begin().await?;
    let row = sqlx::query(
        "SELECT COALESCE(MAX(seq), 0) AS max_seq FROM incomplete_cleanup_events WHERE run_id = ?",
    )
    .bind(run_id)
    .fetch_one(&mut *tx)
    .await?;

    let max_seq = row.get::<i64, _>("max_seq");
    let seq = max_seq + 1;

    sqlx::query(
        r#"
        INSERT INTO incomplete_cleanup_events (run_id, seq, ts, level, kind, message, fields_json)
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
    Ok(CleanupEvent {
        run_id: run_id.to_string(),
        seq,
        ts,
        level: level.to_string(),
        kind: kind.to_string(),
        message: message.to_string(),
        fields,
    })
}

pub async fn list_events(
    db: &SqlitePool,
    run_id: &str,
    limit: u32,
) -> Result<Vec<CleanupEvent>, anyhow::Error> {
    let rows = sqlx::query(
        "SELECT run_id, seq, ts, level, kind, message, fields_json FROM incomplete_cleanup_events WHERE run_id = ? ORDER BY seq ASC LIMIT ?",
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

        events.push(CleanupEvent {
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
