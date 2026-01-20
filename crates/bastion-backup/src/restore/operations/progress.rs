use std::time::{Duration, Instant};

use sqlx::SqlitePool;
use time::OffsetDateTime;

use bastion_core::progress::{ProgressKindV1, ProgressSnapshotV1, ProgressUnitsV1};
use bastion_storage::operations_repo;

const OP_PROGRESS_MIN_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy)]
pub(super) struct OperationProgressUpdate {
    pub(super) stage: &'static str,
    pub(super) done: ProgressUnitsV1,
    pub(super) total: Option<ProgressUnitsV1>,
}

pub(super) fn spawn_operation_progress_writer(
    db: SqlitePool,
    op_id: String,
    kind: ProgressKindV1,
) -> tokio::sync::watch::Sender<Option<OperationProgressUpdate>> {
    let (tx, mut rx) = tokio::sync::watch::channel::<Option<OperationProgressUpdate>>(None);

    tokio::spawn(async move {
        let mut last_emit = Instant::now()
            .checked_sub(OP_PROGRESS_MIN_INTERVAL)
            .unwrap_or_else(Instant::now);
        let mut last_stage: Option<&'static str> = None;
        let mut last_ts: Option<i64> = None;
        let mut last_done_bytes: u64 = 0;

        while rx.changed().await.is_ok() {
            let Some(update) = rx.borrow().clone() else {
                continue;
            };

            let stage_changed = last_stage != Some(update.stage);
            let now_ts = OffsetDateTime::now_utc().unix_timestamp();

            if !stage_changed && last_emit.elapsed() < OP_PROGRESS_MIN_INTERVAL {
                continue;
            }

            let (rate_bps, eta_seconds) = if stage_changed {
                (None, None)
            } else {
                let dt = last_ts.map(|ts| now_ts.saturating_sub(ts)).unwrap_or(0);
                let delta = update.done.bytes.saturating_sub(last_done_bytes);
                let rate = if dt > 0 && delta > 0 {
                    Some(delta.saturating_div(dt as u64).max(1))
                } else {
                    None
                };

                let eta = match (rate, update.total.as_ref()) {
                    (Some(rate), Some(total)) if rate > 0 && total.bytes > update.done.bytes => {
                        Some(
                            total
                                .bytes
                                .saturating_sub(update.done.bytes)
                                .saturating_div(rate),
                        )
                    }
                    _ => None,
                };
                (rate, eta)
            };

            last_emit = Instant::now();
            last_stage = Some(update.stage);
            last_ts = Some(now_ts);
            last_done_bytes = update.done.bytes;

            let snapshot = ProgressSnapshotV1 {
                v: 1,
                kind,
                stage: update.stage.to_string(),
                ts: now_ts,
                done: update.done,
                total: update.total,
                rate_bps,
                eta_seconds,
                detail: None,
            };

            let Ok(payload) = serde_json::to_value(snapshot) else {
                continue;
            };
            let _ = operations_repo::set_operation_progress(&db, &op_id, Some(payload)).await;
        }
    });

    tx
}
