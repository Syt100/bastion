use tempfile::TempDir;

use crate::db;
use crate::jobs_repo::{OverlapPolicy, create_job};
use crate::runs_repo::{RunStatus, create_run};

use super::{
    CleanupTaskStatus, claim_next_due, count_tasks, get_task, ignore_task, list_tasks, mark_done,
    mark_retrying, retry_now, unignore_task, upsert_task_if_missing,
};

#[tokio::test]
async fn cleanup_tasks_round_trip() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let job = create_job(
        &pool,
        "job1",
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
        RunStatus::Failed,
        1000,
        Some(1001),
        None,
        Some("x"),
    )
    .await
    .expect("create run");

    let now = 2000;
    let snapshot = serde_json::json!({
        "node_id": "hub",
        "target": { "type": "local_dir", "base_dir": "/tmp" }
    });
    let inserted = upsert_task_if_missing(
        &pool,
        &run.id,
        &job.id,
        "hub",
        "local_dir",
        &serde_json::to_string(&snapshot).unwrap(),
        now,
    )
    .await
    .expect("upsert");
    assert!(inserted);

    let total = count_tasks(&pool, None, None, None, None)
        .await
        .expect("count");
    assert_eq!(total, 1);

    let tasks = list_tasks(&pool, None, None, None, None, 50, 0)
        .await
        .expect("list");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].job_id, job.id);

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
    assert_eq!(detail.status, CleanupTaskStatus::Running.as_str());

    mark_retrying(&pool, &run.id, now + 60, "network", "connect failed", now)
        .await
        .expect("retrying");

    let ok = retry_now(&pool, &run.id, now).await.expect("retry now");
    assert!(ok);

    let ok = ignore_task(&pool, &run.id, None, Some("ignore"), now)
        .await
        .expect("ignore");
    assert!(ok);

    let ok = unignore_task(&pool, &run.id, now).await.expect("unignore");
    assert!(ok);

    mark_done(&pool, &run.id, now).await.expect("done");
    let detail = get_task(&pool, &run.id)
        .await
        .expect("get2")
        .expect("present2");
    assert_eq!(detail.status, CleanupTaskStatus::Done.as_str());
}

#[tokio::test]
async fn list_and_count_support_multi_value_filters() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let job = create_job(
        &pool,
        "job1",
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

    let run1 = create_run(
        &pool,
        &job.id,
        RunStatus::Failed,
        1000,
        Some(1001),
        None,
        Some("x"),
    )
    .await
    .expect("create run1");
    let run2 = create_run(
        &pool,
        &job.id,
        RunStatus::Failed,
        1002,
        Some(1003),
        None,
        Some("y"),
    )
    .await
    .expect("create run2");

    let now = 2000;
    let snapshot1 = serde_json::json!({
        "node_id": "hub",
        "target": { "type": "local_dir", "base_dir": "/tmp" }
    });
    upsert_task_if_missing(
        &pool,
        &run1.id,
        &job.id,
        "hub",
        "local_dir",
        &serde_json::to_string(&snapshot1).unwrap(),
        now,
    )
    .await
    .expect("upsert1");

    let snapshot2 = serde_json::json!({
        "node_id": "hub",
        "target": { "type": "webdav", "base_url": "http://example", "secret_name": "s" }
    });
    upsert_task_if_missing(
        &pool,
        &run2.id,
        &job.id,
        "hub",
        "webdav",
        &serde_json::to_string(&snapshot2).unwrap(),
        now,
    )
    .await
    .expect("upsert2");

    mark_done(&pool, &run2.id, now).await.expect("done2");

    let statuses = vec![
        CleanupTaskStatus::Queued.as_str().to_string(),
        CleanupTaskStatus::Done.as_str().to_string(),
    ];
    let total = count_tasks(&pool, Some(statuses.as_slice()), None, None, None)
        .await
        .expect("count statuses");
    assert_eq!(total, 2);

    let statuses = vec![CleanupTaskStatus::Done.as_str().to_string()];
    let total = count_tasks(&pool, Some(statuses.as_slice()), None, None, None)
        .await
        .expect("count done");
    assert_eq!(total, 1);

    let target_types = vec!["webdav".to_string()];
    let total = count_tasks(&pool, None, Some(target_types.as_slice()), None, None)
        .await
        .expect("count targets");
    assert_eq!(total, 1);

    let target_types = vec!["local_dir".to_string(), "webdav".to_string()];
    let tasks = list_tasks(&pool, None, Some(target_types.as_slice()), None, None, 50, 0)
        .await
        .expect("list targets");
    assert_eq!(tasks.len(), 2);
}
