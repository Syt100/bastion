use super::super::storage::offline_run_dir;
use super::sync_offline_runs;

#[tokio::test]
async fn sync_offline_runs_ingests_and_removes_dir() {
    use axum::routing::post;
    use axum::{Json, Router};
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct IngestReq {
        run: IngestRun,
    }

    #[derive(Debug, Deserialize)]
    struct IngestRun {
        id: String,
        job_id: String,
        status: String,
        started_at: i64,
        ended_at: i64,
        summary: Option<serde_json::Value>,
        error: Option<String>,
        events: Vec<super::OfflineRunEventV1>,
    }

    let tmp = tempfile::tempdir().unwrap();
    let data_dir = tmp.path();

    let run_id = "run1";
    let run_dir = offline_run_dir(data_dir, run_id);
    tokio::fs::create_dir_all(&run_dir).await.unwrap();

    let run_file = super::OfflineRunFileV1 {
        v: 1,
        id: run_id.to_string(),
        job_id: "job1".to_string(),
        job_name: "job1".to_string(),
        status: super::OfflineRunStatusV1::Success,
        started_at: 1,
        ended_at: Some(2),
        summary: Some(serde_json::json!({ "k": "v" })),
        error: None,
    };
    tokio::fs::write(
        run_dir.join("run.json"),
        serde_json::to_vec(&run_file).unwrap(),
    )
    .await
    .unwrap();

    let ev1 = super::OfflineRunEventV1 {
        seq: 1,
        ts: 1,
        level: "info".to_string(),
        kind: "start".to_string(),
        message: "start".to_string(),
        fields: None,
    };
    let ev2 = super::OfflineRunEventV1 {
        seq: 2,
        ts: 2,
        level: "info".to_string(),
        kind: "done".to_string(),
        message: "done".to_string(),
        fields: Some(serde_json::json!({ "n": 1 })),
    };
    let events_jsonl = format!(
        "{}\n{}\n",
        serde_json::to_string(&ev1).unwrap(),
        serde_json::to_string(&ev2).unwrap()
    );
    tokio::fs::write(run_dir.join("events.jsonl"), events_jsonl)
        .await
        .unwrap();

    let captured = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<IngestReq>::new()));
    let captured_clone = captured.clone();
    let agent_key = "agent-key";

    let app = Router::new().route(
        "/agent/runs/ingest",
        post(
            move |headers: axum::http::HeaderMap, Json(payload): Json<IngestReq>| {
                let captured = captured_clone.clone();
                async move {
                    let auth = headers
                        .get(axum::http::header::AUTHORIZATION)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or_default();
                    assert_eq!(auth, format!("Bearer {agent_key}"));
                    captured.lock().await.push(payload);
                    axum::http::StatusCode::NO_CONTENT
                }
            },
        ),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    let base_url = url::Url::parse(&format!("http://{addr}/")).unwrap();
    sync_offline_runs(&base_url, agent_key, data_dir)
        .await
        .unwrap();
    let _ = shutdown_tx.send(());

    assert!(!run_dir.exists());
    let captured = captured.lock().await;
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].run.id, "run1");
    assert_eq!(captured[0].run.job_id, "job1");
    assert_eq!(captured[0].run.status, "success");
    assert_eq!(captured[0].run.started_at, 1);
    assert_eq!(captured[0].run.ended_at, 2);
    assert_eq!(
        captured[0].run.summary.as_ref().and_then(|v| v.get("k")),
        Some(&serde_json::Value::String("v".to_string()))
    );
    assert!(captured[0].run.error.is_none());
    assert_eq!(captured[0].run.events.len(), 2);
}

#[tokio::test]
async fn sync_offline_runs_returns_error_and_keeps_dir_when_ingest_fails() {
    use axum::Router;
    use axum::http::StatusCode;
    use axum::routing::post;

    let tmp = tempfile::tempdir().unwrap();
    let data_dir = tmp.path();

    let run_id = "run1";
    let run_dir = offline_run_dir(data_dir, run_id);
    tokio::fs::create_dir_all(&run_dir).await.unwrap();

    let run_file = super::OfflineRunFileV1 {
        v: 1,
        id: run_id.to_string(),
        job_id: "job1".to_string(),
        job_name: "job1".to_string(),
        status: super::OfflineRunStatusV1::Success,
        started_at: 1,
        ended_at: Some(2),
        summary: None,
        error: None,
    };
    tokio::fs::write(
        run_dir.join("run.json"),
        serde_json::to_vec(&run_file).unwrap(),
    )
    .await
    .unwrap();

    tokio::fs::write(run_dir.join("events.jsonl"), "")
        .await
        .unwrap();

    let app = Router::new().route(
        "/agent/runs/ingest",
        post(|| async { (StatusCode::INTERNAL_SERVER_ERROR, "nope") }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    let base_url = url::Url::parse(&format!("http://{addr}/")).unwrap();
    let err = sync_offline_runs(&base_url, "agent-key", data_dir)
        .await
        .expect_err("expected ingest failure");
    let _ = shutdown_tx.send(());

    assert!(run_dir.exists());
    let msg = err.to_string();
    assert!(msg.contains("ingest failed"));
    assert!(msg.contains("HTTP 500"));
    assert!(msg.contains("nope"));
}

#[tokio::test]
async fn sync_offline_runs_returns_error_and_keeps_dir_when_events_invalid() {
    use axum::Router;
    use axum::routing::post;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let tmp = tempfile::tempdir().unwrap();
    let data_dir = tmp.path();

    let run_id = "run1";
    let run_dir = offline_run_dir(data_dir, run_id);
    tokio::fs::create_dir_all(&run_dir).await.unwrap();

    let run_file = super::OfflineRunFileV1 {
        v: 1,
        id: run_id.to_string(),
        job_id: "job1".to_string(),
        job_name: "job1".to_string(),
        status: super::OfflineRunStatusV1::Success,
        started_at: 1,
        ended_at: Some(2),
        summary: None,
        error: None,
    };
    tokio::fs::write(
        run_dir.join("run.json"),
        serde_json::to_vec(&run_file).unwrap(),
    )
    .await
    .unwrap();

    tokio::fs::write(run_dir.join("events.jsonl"), "not json\n")
        .await
        .unwrap();

    let called = std::sync::Arc::new(AtomicUsize::new(0));
    let called_clone = called.clone();
    let app = Router::new().route(
        "/agent/runs/ingest",
        post(move |_body: axum::body::Bytes| async move {
            called_clone.fetch_add(1, Ordering::Relaxed);
            axum::http::StatusCode::NO_CONTENT
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    let base_url = url::Url::parse(&format!("http://{addr}/")).unwrap();
    let _ = sync_offline_runs(&base_url, "agent-key", data_dir)
        .await
        .expect_err("expected invalid events failure");
    let _ = shutdown_tx.send(());

    assert!(run_dir.exists());
    assert_eq!(called.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn sync_offline_runs_skips_running_runs_and_keeps_dir() {
    use axum::Router;
    use axum::routing::post;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let tmp = tempfile::tempdir().unwrap();
    let data_dir = tmp.path();

    let run_id = "run1";
    let run_dir = offline_run_dir(data_dir, run_id);
    tokio::fs::create_dir_all(&run_dir).await.unwrap();

    let run_file = super::OfflineRunFileV1 {
        v: 1,
        id: run_id.to_string(),
        job_id: "job1".to_string(),
        job_name: "job1".to_string(),
        status: super::OfflineRunStatusV1::Running,
        started_at: 1,
        ended_at: None,
        summary: None,
        error: None,
    };
    tokio::fs::write(
        run_dir.join("run.json"),
        serde_json::to_vec(&run_file).unwrap(),
    )
    .await
    .unwrap();

    // If ingest is called at all, bump the counter.
    let called = std::sync::Arc::new(AtomicUsize::new(0));
    let called_clone = called.clone();
    let app = Router::new().route(
        "/agent/runs/ingest",
        post(move |_body: axum::body::Bytes| async move {
            called_clone.fetch_add(1, Ordering::Relaxed);
            axum::http::StatusCode::NO_CONTENT
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    let base_url = url::Url::parse(&format!("http://{addr}/")).unwrap();
    sync_offline_runs(&base_url, "agent-key", data_dir)
        .await
        .unwrap();
    let _ = shutdown_tx.send(());

    assert!(run_dir.exists());
    assert_eq!(called.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn sync_offline_runs_defaults_ended_at_to_started_at_and_allows_missing_events() {
    use axum::routing::post;
    use axum::{Json, Router};
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct IngestReq {
        run: IngestRun,
    }

    #[derive(Debug, Deserialize)]
    struct IngestRun {
        id: String,
        status: String,
        started_at: i64,
        ended_at: i64,
        events: Vec<super::OfflineRunEventV1>,
    }

    let tmp = tempfile::tempdir().unwrap();
    let data_dir = tmp.path();

    let run_id = "run1";
    let run_dir = offline_run_dir(data_dir, run_id);
    tokio::fs::create_dir_all(&run_dir).await.unwrap();

    let run_file = super::OfflineRunFileV1 {
        v: 1,
        id: run_id.to_string(),
        job_id: "job1".to_string(),
        job_name: "job1".to_string(),
        status: super::OfflineRunStatusV1::Success,
        started_at: 123,
        ended_at: None,
        summary: None,
        error: None,
    };
    tokio::fs::write(
        run_dir.join("run.json"),
        serde_json::to_vec(&run_file).unwrap(),
    )
    .await
    .unwrap();

    // Intentionally do not create events.jsonl.

    let captured = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<IngestReq>::new()));
    let captured_clone = captured.clone();
    let agent_key = "agent-key";

    let app = Router::new().route(
        "/agent/runs/ingest",
        post(
            move |headers: axum::http::HeaderMap, Json(payload): Json<IngestReq>| {
                let captured = captured_clone.clone();
                async move {
                    let auth = headers
                        .get(axum::http::header::AUTHORIZATION)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or_default();
                    assert_eq!(auth, format!("Bearer {agent_key}"));
                    captured.lock().await.push(payload);
                    axum::http::StatusCode::NO_CONTENT
                }
            },
        ),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await;
    });

    let base_url = url::Url::parse(&format!("http://{addr}/")).unwrap();
    sync_offline_runs(&base_url, agent_key, data_dir)
        .await
        .unwrap();
    let _ = shutdown_tx.send(());

    assert!(!run_dir.exists());
    let captured = captured.lock().await;
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].run.id, "run1");
    assert_eq!(captured[0].run.status, "success");
    assert_eq!(captured[0].run.started_at, 123);
    assert_eq!(captured[0].run.ended_at, 123);
    assert!(captured[0].run.events.is_empty());
}
