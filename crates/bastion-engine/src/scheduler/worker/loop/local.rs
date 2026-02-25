use time::OffsetDateTime;
use tracing::{debug, info, warn};

use bastion_core::job_spec;
use bastion_core::run_failure::RunFailedWithSummary;
use bastion_storage::jobs_repo;
use bastion_storage::run_artifacts_repo;
use bastion_storage::runs_repo::{self, RunStatus};
use bastion_targets::WebdavPutError;

use crate::cancel_registry::global_cancel_registry;
use crate::run_events;

use super::super::execute::{ExecuteRunArgs, RunCanceled, execute_run};
use super::WorkerLoopCtx;
use super::notifications;

fn part_name_from_url(url: &str) -> Option<String> {
    let path = url.split('?').next().unwrap_or(url);
    path.rsplit('/')
        .find(|seg| !seg.is_empty())
        .map(str::to_string)
}

fn fallback_error_hint(error: &anyhow::Error) -> (String, String) {
    let text = error.to_string().to_lowercase();
    if text.contains("http 413") || text.contains("payload too large") {
        return (
            "payload_too_large".to_string(),
            "upload payload may exceed gateway/storage limits; reduce target.part_size_bytes or increase proxy upload limits".to_string(),
        );
    }
    if text.contains("http 429") || text.contains("too many requests") {
        return (
            "rate_limited".to_string(),
            "remote throttled requests; lower request rate/concurrency or increase retry backoff"
                .to_string(),
        );
    }
    if text.contains("http 401") || text.contains("unauthorized") {
        return (
            "auth".to_string(),
            "check WebDAV credentials and token validity (401)".to_string(),
        );
    }
    if text.contains("http 403") || text.contains("forbidden") {
        return (
            "permission".to_string(),
            "check WebDAV account permissions for target path (403)".to_string(),
        );
    }
    if text.contains("timed out")
        || text.contains("timeout")
        || text.contains("connection reset")
        || text.contains("broken pipe")
    {
        return (
            "network".to_string(),
            "network timeout/transport failure detected; consider raising timeout/retries or reducing part size".to_string(),
        );
    }
    if text.contains("rolling uploader dropped") {
        return (
            "upload_pipeline".to_string(),
            "rolling uploader stopped unexpectedly; inspect WebDAV upload errors in run events details"
                .to_string(),
        );
    }
    (
        "unknown".to_string(),
        "inspect error chain details and upstream WebDAV/proxy logs".to_string(),
    )
}

fn build_failed_event_fields(
    error: &anyhow::Error,
    soft: Option<&RunFailedWithSummary>,
) -> serde_json::Value {
    let mut fields = serde_json::Map::new();
    fields.insert(
        "error_code".to_string(),
        serde_json::Value::String(soft.map(|v| v.code).unwrap_or("run_failed").to_string()),
    );

    let chain: Vec<String> = error.chain().take(12).map(ToString::to_string).collect();
    fields.insert("error_chain".to_string(), serde_json::json!(chain));

    let mut found_webdav = false;
    for cause in error.chain() {
        if let Some(put) = cause.downcast_ref::<WebdavPutError>() {
            found_webdav = true;
            fields.insert(
                "source".to_string(),
                serde_json::Value::String("webdav_put".to_string()),
            );
            fields.insert(
                "error_kind".to_string(),
                serde_json::Value::String(put.diagnostic.kind.as_str().to_string()),
            );
            fields.insert(
                "transport_code".to_string(),
                serde_json::Value::String(put.diagnostic.code.to_string()),
            );
            fields.insert(
                "hint".to_string(),
                serde_json::Value::String(put.diagnostic.hint.to_string()),
            );
            if let Some(status) = put.diagnostic.http_status {
                fields.insert("http_status".to_string(), serde_json::json!(status));
            }
            if let Some(retry_after) = put.diagnostic.retry_after {
                fields.insert(
                    "retry_after_secs".to_string(),
                    serde_json::json!(retry_after.as_secs()),
                );
            }
            fields.insert("attempt".to_string(), serde_json::json!(put.attempt));
            fields.insert(
                "max_attempts".to_string(),
                serde_json::json!(put.max_attempts),
            );
            fields.insert("part_size_bytes".to_string(), serde_json::json!(put.size));
            fields.insert(
                "target_url".to_string(),
                serde_json::Value::String(put.url.clone()),
            );
            if let Some(part_name) = part_name_from_url(&put.url) {
                fields.insert(
                    "part_name".to_string(),
                    serde_json::Value::String(part_name),
                );
            }
            break;
        }
    }

    if !found_webdav {
        let (error_kind, hint) = fallback_error_hint(error);
        fields.insert(
            "error_kind".to_string(),
            serde_json::Value::String(error_kind),
        );
        fields.insert("hint".to_string(), serde_json::Value::String(hint));
    }

    serde_json::Value::Object(fields)
}

pub(super) async fn execute_and_complete(
    ctx: &WorkerLoopCtx<'_>,
    job: &jobs_repo::Job,
    run: &runs_repo::Run,
    started_at: OffsetDateTime,
    spec: job_spec::JobSpecV1,
) {
    let cancel_token = global_cancel_registry().register_run(&run.id);
    struct CancelRegistration {
        run_id: String,
    }
    impl Drop for CancelRegistration {
        fn drop(&mut self) {
            global_cancel_registry().unregister_run(&self.run_id);
        }
    }
    let _registration = CancelRegistration {
        run_id: run.id.clone(),
    };

    match execute_run(ExecuteRunArgs {
        db: ctx.db,
        secrets: ctx.secrets,
        run_events_bus: ctx.run_events_bus,
        data_dir: ctx.data_dir,
        job,
        run_id: &run.id,
        started_at,
        cancel_token,
        spec: spec.clone(),
    })
    .await
    {
        Ok(summary) => {
            info!(run_id = %run.id, "run ok");
            let ended_at = OffsetDateTime::now_utc().unix_timestamp();
            let completed = match runs_repo::complete_run(
                ctx.db,
                &run.id,
                RunStatus::Success,
                Some(summary),
                None,
            )
            .await
            {
                Ok(v) => v,
                Err(error) => {
                    warn!(run_id = %run.id, error = %error, "failed to complete run");
                    return;
                }
            };
            if !completed {
                warn!(run_id = %run.id, "run completion skipped (already finalized)");
            }

            let final_status = runs_repo::get_run(ctx.db, &run.id)
                .await
                .ok()
                .flatten()
                .map(|r| r.status)
                .unwrap_or(RunStatus::Success);

            if final_status == RunStatus::Canceled {
                let _ = run_events::append_and_broadcast(
                    ctx.db,
                    ctx.run_events_bus,
                    &run.id,
                    "info",
                    "canceled",
                    "canceled",
                    None,
                )
                .await;
                info!(run_id = %run.id, "run canceled");
                return;
            }

            if final_status != RunStatus::Success {
                warn!(
                    run_id = %run.id,
                    status = %final_status.as_str(),
                    "run completed but final status is not success"
                );
                return;
            }

            if let Err(error) =
                run_artifacts_repo::upsert_run_artifact_from_successful_run(ctx.db, &run.id).await
            {
                warn!(run_id = %run.id, error = %error, "failed to index run artifact");
            } else {
                let _ = run_events::append_and_broadcast(
                    ctx.db,
                    ctx.run_events_bus,
                    &run.id,
                    "info",
                    "complete",
                    "complete",
                    None,
                )
                .await;
                notifications::enqueue_for_run_spec(ctx, &spec, &run.id).await;
                debug!(run_id = %run.id, ended_at, "run completed");
            }
        }
        Err(error) => {
            let canceled = error.downcast_ref::<RunCanceled>().is_some();
            if canceled {
                info!(run_id = %run.id, "run canceled");
            } else {
                warn!(run_id = %run.id, error = %error, "run failed");
            }

            let soft = error.downcast_ref::<RunFailedWithSummary>();
            let requested_status = if canceled {
                RunStatus::Canceled
            } else {
                RunStatus::Failed
            };
            let summary = if canceled {
                None
            } else {
                soft.map(|e| e.summary.clone())
            };
            let error_code = if canceled {
                Some("canceled")
            } else {
                Some(soft.map(|e| e.code).unwrap_or("run_failed"))
            };

            let completed = match runs_repo::complete_run(
                ctx.db,
                &run.id,
                requested_status,
                summary,
                error_code,
            )
            .await
            {
                Ok(v) => v,
                Err(complete_error) => {
                    warn!(
                        run_id = %run.id,
                        error = %complete_error,
                        "failed to complete run after error"
                    );
                    return;
                }
            };
            if !completed {
                warn!(run_id = %run.id, "run completion skipped (already finalized)");
            }

            let final_status = runs_repo::get_run(ctx.db, &run.id)
                .await
                .ok()
                .flatten()
                .map(|r| r.status)
                .unwrap_or(requested_status);

            if final_status == RunStatus::Canceled {
                let _ = run_events::append_and_broadcast(
                    ctx.db,
                    ctx.run_events_bus,
                    &run.id,
                    "info",
                    "canceled",
                    "canceled",
                    None,
                )
                .await;
                return;
            }

            let message = format!("failed: {error}");
            let fields = build_failed_event_fields(&error, soft);
            let _ = run_events::append_and_broadcast(
                ctx.db,
                ctx.run_events_bus,
                &run.id,
                "error",
                "failed",
                &message,
                Some(fields),
            )
            .await;
            notifications::enqueue_for_run_spec(ctx, &spec, &run.id).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use bastion_core::run_failure::RunFailedWithSummary;

    use super::build_failed_event_fields;

    #[test]
    fn failed_event_fields_include_error_code_and_chain() {
        let soft = RunFailedWithSummary::new("source_consistency", "boom", serde_json::json!({}));
        let error = anyhow::anyhow!("boom");

        let fields = build_failed_event_fields(&error, Some(&soft));
        let obj = fields.as_object().expect("object");
        assert_eq!(
            obj.get("error_code").and_then(|v| v.as_str()),
            Some("source_consistency")
        );
        assert!(obj.get("error_chain").is_some());
    }

    #[test]
    fn failed_event_fields_map_payload_too_large_hint() {
        let error = anyhow::anyhow!("webdav request failed: HTTP 413: entity too large");
        let fields = build_failed_event_fields(&error, None);
        let obj = fields.as_object().expect("object");
        assert_eq!(
            obj.get("error_kind").and_then(|v| v.as_str()),
            Some("payload_too_large")
        );
        assert!(
            obj.get("hint")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s.contains("part_size_bytes"))
        );
    }
}
