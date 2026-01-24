use std::time::{Duration, Instant};

use futures_util::Sink;
use tokio_tungstenite::tungstenite::Message;

use bastion_backup as backup;
use bastion_core::agent_protocol::PipelineResolvedV1;
use bastion_core::agent_protocol::TargetResolvedV1;
use bastion_core::job_spec::SqliteSource;
use bastion_core::progress::{ProgressKindV1, ProgressSnapshotV1, ProgressUnitsV1};

use super::super::targets::{store_artifacts_to_resolved_target, target_part_size_bytes};

struct UploadProgressBuilder {
    last_ts: Option<i64>,
    last_done_bytes: u64,
}

impl UploadProgressBuilder {
    fn new() -> Self {
        Self {
            last_ts: None,
            last_done_bytes: 0,
        }
    }

    fn snapshot(&mut self, done_bytes: u64, total_bytes: Option<u64>) -> ProgressSnapshotV1 {
        let now_ts = time::OffsetDateTime::now_utc().unix_timestamp();

        let dt = self
            .last_ts
            .map(|ts| now_ts.saturating_sub(ts))
            .unwrap_or(0);
        let delta = done_bytes.saturating_sub(self.last_done_bytes);
        let rate = if dt > 0 && delta > 0 {
            Some(delta.saturating_div(dt as u64).max(1))
        } else {
            None
        };

        let eta = match (rate, total_bytes) {
            (Some(rate), Some(total)) if rate > 0 && total > done_bytes => {
                Some(total.saturating_sub(done_bytes).saturating_div(rate))
            }
            _ => None,
        };

        self.last_ts = Some(now_ts);
        self.last_done_bytes = done_bytes;

        ProgressSnapshotV1 {
            v: 1,
            kind: ProgressKindV1::Backup,
            stage: "upload".to_string(),
            ts: now_ts,
            done: ProgressUnitsV1 {
                files: 0,
                dirs: 0,
                bytes: done_bytes,
            },
            total: total_bytes.map(|bytes| ProgressUnitsV1 {
                files: 0,
                dirs: 0,
                bytes,
            }),
            rate_bps: rate,
            eta_seconds: eta,
            detail: None,
        }
    }
}

pub(super) async fn run_sqlite_backup(
    tx: &mut (impl Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin),
    ctx: &super::TaskContext<'_>,
    pipeline: PipelineResolvedV1,
    source: SqliteSource,
    target: TargetResolvedV1,
) -> Result<serde_json::Value, anyhow::Error> {
    super::send_run_event(tx, ctx.run_id, "info", "snapshot", "snapshot", None).await?;
    let sqlite_path = source.path.clone();
    let part_size = target_part_size_bytes(&target);
    let encryption = super::payload_encryption(pipeline.encryption);
    let artifact_format = pipeline.format;
    let started_at = ctx.started_at;

    let data_dir_buf = ctx.data_dir.to_path_buf();
    let job_id_clone = ctx.job_id.to_string();
    let run_id_clone = ctx.run_id.to_string();
    let build = tokio::task::spawn_blocking(move || {
        backup::sqlite::build_sqlite_run(
            &data_dir_buf,
            &job_id_clone,
            &run_id_clone,
            started_at,
            &source,
            backup::BuildPipelineOptions {
                artifact_format,
                encryption: &encryption,
                part_size_bytes: part_size,
            },
            None,
        )
    })
    .await??;

    if let Some(check) = build.integrity_check.as_ref() {
        let data = serde_json::json!({
            "ok": check.ok,
            "truncated": check.truncated,
            "lines": check.lines,
        });
        super::send_run_event(
            tx,
            ctx.run_id,
            if check.ok { "info" } else { "error" },
            "integrity_check",
            "integrity_check",
            Some(data),
        )
        .await?;
        if !check.ok {
            let first = check.lines.first().cloned().unwrap_or_default();
            anyhow::bail!("sqlite integrity_check failed: {}", first);
        }
    }

    super::send_run_event(tx, ctx.run_id, "info", "upload", "upload", None).await?;

    struct UploadThrottle {
        last_emit: Instant,
        last_done: u64,
        last_total: Option<u64>,
    }
    const UPLOAD_PROGRESS_MIN_INTERVAL: Duration = Duration::from_secs(1);

    let upload_throttle = std::sync::Arc::new(std::sync::Mutex::new(UploadThrottle {
        last_emit: Instant::now()
            .checked_sub(UPLOAD_PROGRESS_MIN_INTERVAL)
            .unwrap_or_else(Instant::now),
        last_done: 0,
        last_total: None,
    }));

    let (progress_tx, mut progress_rx) =
        tokio::sync::mpsc::channel::<bastion_targets::StoreRunProgress>(8);
    let upload_cb: std::sync::Arc<dyn Fn(bastion_targets::StoreRunProgress) + Send + Sync> = {
        let upload_throttle = upload_throttle.clone();
        std::sync::Arc::new(move |p: bastion_targets::StoreRunProgress| {
            let now = Instant::now();
            let mut guard = match upload_throttle.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };

            let total_bytes = p.bytes_total;
            let done_bytes = p.bytes_done;
            let finished = total_bytes.is_some_and(|t| done_bytes >= t);
            let should_emit =
                finished || now.duration_since(guard.last_emit) >= UPLOAD_PROGRESS_MIN_INTERVAL;
            if !should_emit {
                return;
            }
            if done_bytes == guard.last_done && total_bytes == guard.last_total {
                return;
            }

            guard.last_emit = now;
            guard.last_done = done_bytes;
            guard.last_total = total_bytes;

            let _ = progress_tx.try_send(p);
        })
    };

    let mut upload_fut = std::pin::pin!(store_artifacts_to_resolved_target(
        ctx.job_id,
        ctx.run_id,
        &target,
        &build.artifacts,
        Some(upload_cb),
    ));

    let mut progress = UploadProgressBuilder::new();
    let target_summary = loop {
        tokio::select! {
            res = &mut upload_fut => break res?,
            maybe_update = progress_rx.recv() => {
                if let Some(p) = maybe_update {
                    super::send_run_progress_snapshot(tx, ctx.run_id, progress.snapshot(p.bytes_done, p.bytes_total)).await?;
                }
            }
        }
    };
    let _ = tokio::fs::remove_dir_all(&build.artifacts.run_dir).await;

    Ok(serde_json::json!({
        "target": target_summary,
        "entries_count": build.artifacts.entries_count,
        "parts": build.artifacts.parts.len(),
        "sqlite": {
            "path": sqlite_path,
            "snapshot_name": build.snapshot_name,
            "snapshot_size": build.snapshot_size,
            "integrity_check": build.integrity_check.map(|check| serde_json::json!({
                "ok": check.ok,
                "truncated": check.truncated,
                "lines": check.lines,
            })),
        },
    }))
}
