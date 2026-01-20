use tempfile::TempDir;

use crate::db;
use crate::incomplete_cleanup_repo;

use super::{
    IncompleteCleanupRun, RunStatus, append_run_event, claim_next_queued_run, complete_run,
    create_run, get_run_progress, list_incomplete_cleanup_candidates, list_run_events,
    list_runs_for_job, prune_runs_ended_before, requeue_run, set_run_progress,
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

    requeue_run(&pool, &claimed.id).await.expect("requeue");
    let claimed2 = claim_next_queued_run(&pool)
        .await
        .expect("claim2")
        .expect("claimed2");
    assert_eq!(claimed2.id, claimed.id);

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

#[tokio::test]
async fn list_incomplete_cleanup_candidates_filters_and_orders() {
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

    // Old queued -> included.
    let r1 = create_run(&pool, "job1", RunStatus::Queued, 10, None, None, None)
        .await
        .expect("run1");
    // Old running -> included.
    let r2 = create_run(&pool, "job1", RunStatus::Running, 20, None, None, None)
        .await
        .expect("run2");
    // Old success -> excluded.
    let _ = create_run(&pool, "job1", RunStatus::Success, 30, Some(31), None, None)
        .await
        .expect("run3");
    // New failed -> excluded by cutoff.
    let _ = create_run(
        &pool,
        "job1",
        RunStatus::Failed,
        999,
        Some(1000),
        None,
        None,
    )
    .await
    .expect("run4");

    let got = list_incomplete_cleanup_candidates(&pool, 100, 10)
        .await
        .expect("list");
    let ids: Vec<String> = got.iter().map(|r| r.id.clone()).collect();
    assert_eq!(ids, vec![r1.id.clone(), r2.id.clone()]);

    // Ensure struct fields are populated.
    assert!(matches!(
        got[0],
        IncompleteCleanupRun {
            status: RunStatus::Queued,
            ..
        }
    ));

    // Runs that already have a cleanup task are excluded.
    let snapshot = serde_json::json!({
        "node_id": "hub",
        "target": { "type": "local_dir", "base_dir": "/tmp" }
    });
    incomplete_cleanup_repo::upsert_task_if_missing(
        &pool,
        &r1.id,
        "job1",
        "hub",
        "local_dir",
        &serde_json::to_string(&snapshot).unwrap(),
        2000,
    )
    .await
    .expect("upsert");

    let got = list_incomplete_cleanup_candidates(&pool, 100, 10)
        .await
        .expect("list2");
    let ids: Vec<String> = got.iter().map(|r| r.id.clone()).collect();
    assert_eq!(ids, vec![r2.id.clone()]);
}

#[tokio::test]
async fn run_progress_round_trips_and_can_be_cleared() {
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

    let run = create_run(&pool, "job1", RunStatus::Queued, 1000, None, None, None)
        .await
        .expect("create run");

    let progress = serde_json::json!({"v":1,"kind":"backup","stage":"scan","done":{"files":1,"dirs":0,"bytes":123},"total":{"files":10,"dirs":2,"bytes":456}});
    let ok = set_run_progress(&pool, &run.id, Some(progress.clone()))
        .await
        .expect("set progress");
    assert!(ok);

    let got = get_run_progress(&pool, &run.id)
        .await
        .expect("get progress")
        .expect("present");
    assert_eq!(got, progress);

    let ok = set_run_progress(&pool, &run.id, None)
        .await
        .expect("clear progress");
    assert!(ok);

    let got = get_run_progress(&pool, &run.id)
        .await
        .expect("get progress 2");
    assert!(got.is_none());
}
