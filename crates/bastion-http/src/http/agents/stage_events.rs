use std::collections::HashMap;

use sqlx::SqlitePool;

use bastion_engine::run_events;
use bastion_engine::run_events_bus::RunEventsBus;

const STAGE_KINDS: [&str; 3] = ["scan", "packaging", "upload"];

fn normalize_stage(stage: &str) -> Option<&str> {
    let s = stage.trim();
    if STAGE_KINDS.contains(&s) {
        Some(s)
    } else {
        None
    }
}

pub(crate) async fn maybe_append_run_stage_event(
    db: &SqlitePool,
    bus: &RunEventsBus,
    last_by_run_id: &mut HashMap<String, String>,
    run_id: &str,
    stage: &str,
) {
    let Some(stage) = normalize_stage(stage) else {
        return;
    };

    // Initialize cache from DB to avoid duplicates after reconnect.
    if !last_by_run_id.contains_key(run_id) {
        let q = r#"
            SELECT kind
            FROM run_events
            WHERE run_id = ?
              AND kind IN ('scan', 'packaging', 'upload')
            ORDER BY seq DESC
            LIMIT 1
        "#;
        if let Ok(Some(kind)) = sqlx::query_scalar::<_, String>(q)
            .bind(run_id)
            .fetch_optional(db)
            .await
        {
            last_by_run_id.insert(run_id.to_string(), kind);
        }
    }

    if last_by_run_id.get(run_id).is_some_and(|v| v == stage) {
        return;
    }

    last_by_run_id.insert(run_id.to_string(), stage.to_string());
    let _ = run_events::append_and_broadcast(db, bus, run_id, "info", stage, stage, None).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    use bastion_storage::db;
    use bastion_storage::runs_repo::{RunStatus, create_run, list_run_events};

    #[tokio::test]
    async fn stage_events_are_deduped_and_reconnect_safe() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        sqlx::query(
            "INSERT INTO jobs (id, name, schedule, overlap_policy, spec_json, created_at, updated_at) VALUES (?, ?, NULL, 'queue', ?, ?, ?)",
        )
        .bind("job1")
        .bind("job1")
        .bind(r#"{"v":1,"type":"filesystem","source":{"root":"/"},"target":{"type":"local_dir","base_dir":"/tmp"}}"#)
        .bind(1000)
        .bind(1000)
        .execute(&pool)
        .await
        .expect("insert job");

        let run = create_run(&pool, "job1", RunStatus::Running, 1000, None, None, None)
            .await
            .expect("create run");

        let bus = RunEventsBus::new_with_options(8, 60, 1);

        let mut cache = HashMap::new();
        maybe_append_run_stage_event(&pool, &bus, &mut cache, &run.id, "scan").await;
        maybe_append_run_stage_event(&pool, &bus, &mut cache, &run.id, "scan").await;
        maybe_append_run_stage_event(&pool, &bus, &mut cache, &run.id, "packaging").await;
        maybe_append_run_stage_event(&pool, &bus, &mut cache, &run.id, "packaging").await;

        let events = list_run_events(&pool, &run.id, 100)
            .await
            .expect("list events");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, "scan");
        assert_eq!(events[1].kind, "packaging");

        // Reconnect: new empty cache should not duplicate the last known stage.
        let mut cache2 = HashMap::new();
        maybe_append_run_stage_event(&pool, &bus, &mut cache2, &run.id, "packaging").await;
        maybe_append_run_stage_event(&pool, &bus, &mut cache2, &run.id, "upload").await;

        let events2 = list_run_events(&pool, &run.id, 100)
            .await
            .expect("list events 2");
        assert_eq!(events2.len(), 3);
        assert_eq!(events2[2].kind, "upload");
    }
}
