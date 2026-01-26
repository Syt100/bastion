use sqlx::{Row, SqlitePool};

use super::types::ArtifactDeleteEvent;

pub async fn append_event(
    db: &SqlitePool,
    run_id: &str,
    level: &str,
    kind: &str,
    message: &str,
    fields: Option<serde_json::Value>,
    ts: i64,
) -> Result<ArtifactDeleteEvent, anyhow::Error> {
    let fields_json = fields.as_ref().map(serde_json::to_string).transpose()?;

    // NOTE: `artifact_delete_events` uses a per-run `(run_id, seq)` primary key. Multiple producers
    // (Hub, delete worker, agent callbacks) can append concurrently, so we must tolerate races.
    //
    // We compute the next seq via a subquery and retry on UNIQUE/busy errors.
    const MAX_ATTEMPTS: usize = 10;
    for attempt in 0..MAX_ATTEMPTS {
        let mut tx = db.begin().await?;
        let res = sqlx::query(
            r#"
            INSERT INTO artifact_delete_events (run_id, seq, ts, level, kind, message, fields_json)
            VALUES (
              ?,
              COALESCE((SELECT MAX(seq) FROM artifact_delete_events WHERE run_id = ?), 0) + 1,
              ?, ?, ?, ?, ?
            )
            RETURNING seq
            "#,
        )
        .bind(run_id)
        .bind(run_id)
        .bind(ts)
        .bind(level)
        .bind(kind)
        .bind(message)
        .bind(fields_json.clone())
        .fetch_one(&mut *tx)
        .await;

        match res {
            Ok(row) => {
                let seq = row.get::<i64, _>("seq");
                tx.commit().await?;
                return Ok(ArtifactDeleteEvent {
                    run_id: run_id.to_string(),
                    seq,
                    ts,
                    level: level.to_string(),
                    kind: kind.to_string(),
                    message: message.to_string(),
                    fields,
                });
            }
            Err(error) => {
                let _ = tx.rollback().await;
                let msg = error.to_string();
                let retryable = msg.contains("UNIQUE constraint failed")
                    || msg.contains("database is locked")
                    || msg.contains("SQLITE_BUSY");
                if retryable && attempt + 1 < MAX_ATTEMPTS {
                    continue;
                }
                return Err(error.into());
            }
        }
    }

    unreachable!("append_event retry loop should always return");
}

pub async fn list_events(
    db: &SqlitePool,
    run_id: &str,
    limit: u32,
) -> Result<Vec<ArtifactDeleteEvent>, anyhow::Error> {
    let rows = sqlx::query(
        "SELECT run_id, seq, ts, level, kind, message, fields_json FROM artifact_delete_events WHERE run_id = ? ORDER BY seq ASC LIMIT ?",
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

        events.push(ArtifactDeleteEvent {
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
