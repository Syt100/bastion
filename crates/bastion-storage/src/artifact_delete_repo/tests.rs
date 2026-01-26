use tempfile::TempDir;

use crate::db;
use crate::jobs_repo::{OverlapPolicy, create_job};
use crate::runs_repo::{RunStatus, create_run};

use super::{
    append_event, claim_next_due, get_task, ignore_task, list_events, list_tasks_by_run_ids,
    mark_done, mark_retrying, retry_now, upsert_task_if_missing,
};

#[tokio::test]
async fn delete_tasks_round_trip_and_idempotent_enqueue() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let job = create_job(
        &pool,
        "job1",
        None,
        None,
        None,
        OverlapPolicy::Queue,
        serde_json::json!({
            "v": 1,
            "type": "filesystem",
            "source": { "root": "/" },
            "target": { "type": "local_dir", "base_dir": "/tmp" }
        }),
    )
    .await
    .expect("create job");

    let run = create_run(
        &pool,
        &job.id,
        RunStatus::Success,
        1000,
        Some(1001),
        None,
        None,
    )
    .await
    .expect("create run");

    let now = 2000;
    let snapshot = serde_json::json!({
        "node_id": "hub",
        "target": { "type": "local_dir", "base_dir": "/tmp" }
    });
    let snapshot_json = serde_json::to_string(&snapshot).unwrap();

    let inserted = upsert_task_if_missing(
        &pool,
        &run.id,
        &job.id,
        "hub",
        "local_dir",
        &snapshot_json,
        now,
    )
    .await
    .expect("upsert");
    assert!(inserted);

    let inserted = upsert_task_if_missing(
        &pool,
        &run.id,
        &job.id,
        "hub",
        "local_dir",
        &snapshot_json,
        now,
    )
    .await
    .expect("upsert2");
    assert!(!inserted);

    let summary = list_tasks_by_run_ids(&pool, std::slice::from_ref(&run.id))
        .await
        .expect("list by ids");
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].run_id, run.id);

    let claimed = claim_next_due(&pool, now)
        .await
        .expect("claim")
        .expect("claimed");
    assert_eq!(claimed.run_id, run.id);
    assert_eq!(claimed.attempts, 1);

    let detail = get_task(&pool, &run.id)
        .await
        .expect("get")
        .expect("present");
    assert_eq!(detail.status, "running");

    mark_retrying(&pool, &run.id, now + 60, "network", "connect failed", now)
        .await
        .expect("retrying");

    let ok = retry_now(&pool, &run.id, now).await.expect("retry now");
    assert!(ok);

    let ok = ignore_task(&pool, &run.id, None, Some("ignore"), now)
        .await
        .expect("ignore");
    assert!(ok);

    mark_done(&pool, &run.id, now).await.expect("done");
    let detail = get_task(&pool, &run.id)
        .await
        .expect("get2")
        .expect("present2");
    assert_eq!(detail.status, "done");

    let events = list_events(&pool, &run.id, 100).await.expect("events");
    assert_eq!(events.len(), 0);
}

#[tokio::test]
async fn delete_events_append_is_concurrency_safe() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let job = create_job(
        &pool,
        "job1",
        None,
        None,
        None,
        OverlapPolicy::Queue,
        serde_json::json!({
            "v": 1,
            "type": "filesystem",
            "source": { "root": "/" },
            "target": { "type": "local_dir", "base_dir": "/tmp" }
        }),
    )
    .await
    .expect("create job");

    let run = create_run(
        &pool,
        &job.id,
        RunStatus::Success,
        1000,
        Some(1001),
        None,
        None,
    )
    .await
    .expect("create run");

    let now = 2000;
    let snapshot = serde_json::json!({
        "node_id": "hub",
        "target": { "type": "local_dir", "base_dir": "/tmp" }
    });
    let snapshot_json = serde_json::to_string(&snapshot).unwrap();

    let inserted = upsert_task_if_missing(
        &pool,
        &run.id,
        &job.id,
        "hub",
        "local_dir",
        &snapshot_json,
        now,
    )
    .await
    .expect("upsert");
    assert!(inserted);

    const N: usize = 20;
    let mut handles = Vec::new();
    for i in 0..N {
        let pool = pool.clone();
        let run_id = run.id.clone();
        handles.push(tokio::spawn(async move {
            append_event(
                &pool,
                &run_id,
                "info",
                "test",
                &format!("msg {i}"),
                Some(serde_json::json!({ "i": i })),
                now + i as i64,
            )
            .await
        }));
    }

    let mut seqs = Vec::new();
    for handle in handles {
        let evt = handle.await.expect("join").expect("append");
        seqs.push(evt.seq);
    }
    seqs.sort();
    assert_eq!(seqs, (1..=N as i64).collect::<Vec<_>>());

    let events = list_events(&pool, &run.id, 200).await.expect("events");
    assert_eq!(events.len(), N);
    assert_eq!(events.first().map(|e| e.seq), Some(1));
    assert_eq!(events.last().map(|e| e.seq), Some(N as i64));
}
