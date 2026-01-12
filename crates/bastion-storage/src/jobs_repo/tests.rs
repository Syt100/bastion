use tempfile::TempDir;

use crate::db;

use super::{OverlapPolicy, create_job, get_job, list_jobs, update_job};

#[tokio::test]
async fn jobs_crud_round_trip() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let spec = serde_json::json!({ "v": 1, "type": "filesystem" });
    let job = create_job(
        &pool,
        "job1",
        None,
        Some("0 */6 * * *"),
        Some("UTC"),
        OverlapPolicy::Queue,
        spec,
    )
    .await
    .expect("create");

    let fetched = get_job(&pool, &job.id)
        .await
        .expect("get")
        .expect("present");
    assert_eq!(fetched.name, "job1");
    assert_eq!(fetched.overlap_policy, OverlapPolicy::Queue);
    assert_eq!(fetched.schedule_timezone, "UTC");

    let listed = list_jobs(&pool).await.expect("list");
    assert_eq!(listed.len(), 1);

    let updated_spec = serde_json::json!({ "v": 1, "type": "sqlite" });
    let updated = update_job(
        &pool,
        &job.id,
        "job2",
        Some("agent-1"),
        None,
        Some("Asia/Shanghai"),
        OverlapPolicy::Reject,
        updated_spec,
    )
    .await
    .expect("update");
    assert!(updated);

    let fetched = get_job(&pool, &job.id)
        .await
        .expect("get2")
        .expect("present2");
    assert_eq!(fetched.name, "job2");
    assert_eq!(fetched.agent_id.as_deref(), Some("agent-1"));
    assert_eq!(fetched.overlap_policy, OverlapPolicy::Reject);
    assert!(fetched.schedule.is_none());
    assert_eq!(fetched.schedule_timezone, "Asia/Shanghai");
}
