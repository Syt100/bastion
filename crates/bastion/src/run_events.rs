use sqlx::SqlitePool;

use crate::run_events_bus::RunEventsBus;
use crate::runs_repo;

pub async fn append_and_broadcast(
    db: &SqlitePool,
    bus: &RunEventsBus,
    run_id: &str,
    level: &str,
    kind: &str,
    message: &str,
    fields: Option<serde_json::Value>,
) -> Result<runs_repo::RunEvent, anyhow::Error> {
    let event = runs_repo::append_run_event(db, run_id, level, kind, message, fields).await?;
    bus.publish(&event);
    Ok(event)
}
