use tempfile::TempDir;

use crate::db;

use super::{
    OperationKind, OperationStatus, append_event, complete_operation, create_operation,
    get_operation, list_events, list_operations_by_subject,
};

#[tokio::test]
async fn operations_and_events_round_trip() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let op = create_operation(&pool, OperationKind::Verify, None)
        .await
        .expect("create");
    append_event(&pool, &op.id, "info", "start", "start", None)
        .await
        .expect("event1");
    append_event(
        &pool,
        &op.id,
        "info",
        "step",
        "step",
        Some(serde_json::json!({"n": 1})),
    )
    .await
    .expect("event2");

    complete_operation(&pool, &op.id, OperationStatus::Success, None, None)
        .await
        .expect("complete");

    let fetched = get_operation(&pool, &op.id)
        .await
        .expect("get")
        .expect("present");
    assert_eq!(fetched.kind, OperationKind::Verify);
    assert_eq!(fetched.status, OperationStatus::Success);

    let events = list_events(&pool, &op.id, 100).await.expect("list");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].seq, 1);
    assert_eq!(events[1].seq, 2);
}

#[tokio::test]
async fn list_operations_by_subject_filters_by_kind_and_id() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let op1 = create_operation(&pool, OperationKind::Restore, Some(("run", "r1")))
        .await
        .expect("op1");
    let op2 = create_operation(&pool, OperationKind::Verify, Some(("run", "r1")))
        .await
        .expect("op2");
    let _op_other = create_operation(&pool, OperationKind::Verify, Some(("run", "r2")))
        .await
        .expect("op_other");

    let ops = list_operations_by_subject(&pool, "run", "r1", 10)
        .await
        .expect("list");
    let ids = ops.iter().map(|o| o.id.as_str()).collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&op1.id.as_str()));
    assert!(ids.contains(&op2.id.as_str()));
}
