use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;

use tracing::warn;

use bastion_core::run_failure::RunFailedWithSummary;

use super::super::storage::OfflineRunWriterHandle;
use super::sink::{OfflineSink, mark_summary_executed_offline};
use super::types::{InFlightCounts, OfflineRunTask};

type BoxFuture<'a> =
    Pin<Box<dyn std::future::Future<Output = Result<(), anyhow::Error>> + Send + 'a>>;

trait OfflineTaskExecutor: Send + Sync {
    fn execute<'a>(
        &'a self,
        data_dir: &'a Path,
        agent_id: &'a str,
        task: &'a OfflineRunTask,
    ) -> BoxFuture<'a>;
}

impl<F> OfflineTaskExecutor for F
where
    F: for<'a> Fn(&'a Path, &'a str, &'a OfflineRunTask) -> BoxFuture<'a> + Send + Sync,
{
    fn execute<'a>(
        &'a self,
        data_dir: &'a Path,
        agent_id: &'a str,
        task: &'a OfflineRunTask,
    ) -> BoxFuture<'a> {
        (self)(data_dir, agent_id, task)
    }
}

fn execute_offline_run_task_boxed<'a>(
    data_dir: &'a Path,
    agent_id: &'a str,
    task: &'a OfflineRunTask,
) -> BoxFuture<'a> {
    Box::pin(execute_offline_run_task(data_dir, agent_id, task))
}

pub(super) async fn offline_worker_loop(
    data_dir: PathBuf,
    agent_id: String,
    run_lock: std::sync::Arc<tokio::sync::Mutex<()>>,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<OfflineRunTask>,
    inflight: std::sync::Arc<tokio::sync::Mutex<InFlightCounts>>,
) {
    offline_worker_loop_with_executor(
        data_dir,
        agent_id,
        run_lock,
        &mut rx,
        inflight,
        &execute_offline_run_task_boxed,
    )
    .await;
}

async fn offline_worker_loop_with_executor(
    data_dir: PathBuf,
    agent_id: String,
    run_lock: std::sync::Arc<tokio::sync::Mutex<()>>,
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<OfflineRunTask>,
    inflight: std::sync::Arc<tokio::sync::Mutex<InFlightCounts>>,
    executor: &dyn OfflineTaskExecutor,
) {
    while let Some(task) = rx.recv().await {
        let job_id = task.job_id.clone();
        let run_id = task.run_id.clone();

        let _guard = run_lock.lock().await;
        if let Err(error) = executor.execute(&data_dir, &agent_id, &task).await {
            warn!(
                agent_id = %agent_id,
                job_id = %job_id,
                run_id = %run_id,
                error = %error,
                "offline run failed"
            );
        }

        let mut state = inflight.lock().await;
        state.dec_job(&job_id);
    }
}

async fn execute_offline_run_task(
    data_dir: &Path,
    agent_id: &str,
    task: &OfflineRunTask,
) -> Result<(), anyhow::Error> {
    let started_at = time::OffsetDateTime::now_utc();
    let writer = OfflineRunWriterHandle::start(
        data_dir,
        &task.run_id,
        &task.job_id,
        &task.job_name,
        started_at.unix_timestamp(),
    )
    .await?;

    let mut sink = OfflineSink::new(writer);

    let _ = sink.writer().append_event(
        "info",
        "queued",
        "queued",
        Some(serde_json::json!({ "source": "schedule", "executed_offline": true })),
    );

    let run_task = bastion_core::agent_protocol::BackupRunTaskV1 {
        run_id: task.run_id.clone(),
        job_id: task.job_id.clone(),
        started_at: started_at.unix_timestamp(),
        spec: task.spec.clone(),
    };

    let outcome =
        super::super::super::handle_backup_task(data_dir, &mut sink, &task.run_id, run_task).await;

    let (writer, task_summary) = sink.into_parts();
    match outcome {
        Ok(()) => {
            let mut summary = task_summary.unwrap_or_else(|| serde_json::json!({}));
            mark_summary_executed_offline(&mut summary);
            writer.finish_success(summary).await?;
        }
        Err(error) => {
            let soft = error.downcast_ref::<RunFailedWithSummary>();
            let error_code = soft.map(|e| e.code).unwrap_or("run_failed");

            let mut summary = soft
                .map(|e| e.summary.clone())
                .unwrap_or_else(|| serde_json::json!({}));
            summary.as_object_mut().map(|o| {
                o.insert(
                    "error_code".to_string(),
                    serde_json::Value::String(error_code.to_string()),
                )
            });
            mark_summary_executed_offline(&mut summary);

            let message = format!("failed: {error}");
            let _ = writer.append_event(
                "error",
                "failed",
                &message,
                Some(serde_json::json!({ "agent_id": agent_id })),
            );
            writer.finish_failed(error_code, summary).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use bastion_core::agent_protocol::{JobSpecResolvedV1, TargetResolvedV1};
    use bastion_core::job_spec::{
        FilesystemSource, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy,
    };

    use super::super::types::InFlightCounts;
    use super::{
        BoxFuture, OfflineRunTask, OfflineTaskExecutor, offline_worker_loop_with_executor,
    };

    fn task(run_id: &str, job_id: &str) -> OfflineRunTask {
        OfflineRunTask {
            run_id: run_id.to_string(),
            job_id: job_id.to_string(),
            job_name: job_id.to_string(),
            spec: JobSpecResolvedV1::Filesystem {
                v: 1,
                pipeline: Default::default(),
                source: FilesystemSource {
                    pre_scan: true,
                    paths: vec![],
                    root: String::new(),
                    include: vec![],
                    exclude: vec![],
                    symlink_policy: FsSymlinkPolicy::Skip,
                    hardlink_policy: FsHardlinkPolicy::Copy,
                    error_policy: FsErrorPolicy::FailFast,
                },
                target: TargetResolvedV1::LocalDir {
                    base_dir: "/tmp".to_string(),
                    part_size_bytes: 1024,
                },
            },
        }
    }

    #[tokio::test]
    async fn offline_worker_loop_decrements_inflight_counts_on_success() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_path_buf();

        let run_lock = std::sync::Arc::new(tokio::sync::Mutex::new(()));
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<OfflineRunTask>();
        let inflight = std::sync::Arc::new(tokio::sync::Mutex::new(InFlightCounts::default()));

        {
            let mut state = inflight.lock().await;
            state.inc_job("job1");
            state.inc_job("job1");
        }

        struct OkExecutor {
            called: std::sync::Arc<AtomicUsize>,
            run_lock: std::sync::Arc<tokio::sync::Mutex<()>>,
        }

        impl OfflineTaskExecutor for OkExecutor {
            fn execute<'a>(
                &'a self,
                _data_dir: &'a std::path::Path,
                _agent_id: &'a str,
                _task: &'a OfflineRunTask,
            ) -> BoxFuture<'a> {
                let called = self.called.clone();
                let run_lock = self.run_lock.clone();
                Box::pin(async move {
                    // Worker loop should hold the run lock while executing.
                    assert!(run_lock.try_lock().is_err());
                    called.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })
            }
        }

        let called = std::sync::Arc::new(AtomicUsize::new(0));
        let exec = OkExecutor {
            called: called.clone(),
            run_lock: run_lock.clone(),
        };

        tx.send(task("run1", "job1")).unwrap();
        tx.send(task("run2", "job1")).unwrap();
        drop(tx);

        offline_worker_loop_with_executor(
            data_dir,
            "agent1".to_string(),
            run_lock.clone(),
            &mut rx,
            inflight.clone(),
            &exec,
        )
        .await;

        assert_eq!(called.load(Ordering::Relaxed), 2);
        let state = inflight.lock().await;
        assert_eq!(state.inflight_for_job("job1"), 0);
    }

    #[tokio::test]
    async fn offline_worker_loop_continues_after_executor_error() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_path_buf();

        let run_lock = std::sync::Arc::new(tokio::sync::Mutex::new(()));
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<OfflineRunTask>();
        let inflight = std::sync::Arc::new(tokio::sync::Mutex::new(InFlightCounts::default()));

        {
            let mut state = inflight.lock().await;
            state.inc_job("job1");
            state.inc_job("job1");
        }

        struct FailFirstExecutor {
            calls: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
        }

        impl OfflineTaskExecutor for FailFirstExecutor {
            fn execute<'a>(
                &'a self,
                _data_dir: &'a std::path::Path,
                _agent_id: &'a str,
                task: &'a OfflineRunTask,
            ) -> BoxFuture<'a> {
                let calls = self.calls.clone();
                let run_id = task.run_id.clone();
                Box::pin(async move {
                    calls.lock().unwrap().push(run_id.clone());
                    if run_id == "run1" {
                        Err(anyhow::anyhow!("boom"))
                    } else {
                        Ok(())
                    }
                })
            }
        }

        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let exec = FailFirstExecutor {
            calls: calls.clone(),
        };

        tx.send(task("run1", "job1")).unwrap();
        tx.send(task("run2", "job1")).unwrap();
        drop(tx);

        offline_worker_loop_with_executor(
            data_dir,
            "agent1".to_string(),
            run_lock,
            &mut rx,
            inflight.clone(),
            &exec,
        )
        .await;

        assert_eq!(
            calls.lock().unwrap().as_slice(),
            &["run1".to_string(), "run2".to_string()]
        );
        let state = inflight.lock().await;
        assert_eq!(state.inflight_for_job("job1"), 0);
    }
}
