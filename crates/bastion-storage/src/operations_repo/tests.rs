use tempfile::TempDir;

use crate::db;

use super::{
    OperationKind, OperationStatus, append_event, complete_operation, create_operation,
    get_operation, list_events,
};

#[tokio::test]
async fn operations_and_events_round_trip() {
    let temp = TempDir::new().expect("tempdir");
    let pool = db::init(temp.path()).await.expect("db init");

    let op = create_operation(&pool, OperationKind::Verify)
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
