use std::path::Path;
use std::path::PathBuf;

use tracing::warn;

use bastion_core::run_failure::RunFailedWithSummary;

use super::super::storage::OfflineRunWriterHandle;
use super::sink::{OfflineSink, mark_summary_executed_offline};
use super::types::{InFlightCounts, OfflineRunTask};

pub(super) async fn offline_worker_loop(
    data_dir: PathBuf,
    agent_id: String,
    run_lock: std::sync::Arc<tokio::sync::Mutex<()>>,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<OfflineRunTask>,
    inflight: std::sync::Arc<tokio::sync::Mutex<InFlightCounts>>,
) {
    while let Some(task) = rx.recv().await {
        let job_id = task.job_id.clone();
        let run_id = task.run_id.clone();

        let _guard = run_lock.lock().await;
        if let Err(error) = execute_offline_run_task(&data_dir, &agent_id, &task).await {
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
