use sqlx::SqlitePool;

use crate::run_events_bus::RunEventsBus;
use bastion_storage::runs_repo;

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

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use bastion_storage::db;

    use super::append_and_broadcast;
    use crate::run_events_bus::RunEventsBus;

    #[tokio::test]
    async fn append_and_broadcast_persists_event_and_notifies_subscribers() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();

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
        .unwrap();

        let run = bastion_storage::runs_repo::create_run(
            &pool,
            "job1",
            bastion_storage::runs_repo::RunStatus::Queued,
            1000,
            None,
            None,
            None,
        )
        .await
        .unwrap();

        let bus = RunEventsBus::new_with_options(8, 60, 1);
        let mut rx = bus.subscribe(&run.id);

        let event = append_and_broadcast(
            &pool,
            &bus,
            &run.id,
            "info",
            "test",
            "hello",
            Some(serde_json::json!({ "k": 1 })),
        )
        .await
        .unwrap();
        assert_eq!(event.seq, 1);
        assert_eq!(event.message, "hello");

        let got = rx.recv().await.unwrap();
        assert_eq!(got.seq, event.seq);
        assert_eq!(got.message, event.message);
        assert_eq!(got.fields, event.fields);
    }
}
