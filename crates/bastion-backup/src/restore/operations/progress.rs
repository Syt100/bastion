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
        let mut last_total_bytes: Option<u64> = None;
        let mut last_update: Option<OperationProgressUpdate> = None;

        while rx.changed().await.is_ok() {
            let Some(update) = *rx.borrow() else {
                continue;
            };
            last_update = Some(update);

            let stage_changed = last_stage != Some(update.stage);
            let finished = update
                .total
                .as_ref()
                .is_some_and(|t| update.done.bytes >= t.bytes);
            let now_ts = OffsetDateTime::now_utc().unix_timestamp();

            if !stage_changed && !finished && last_emit.elapsed() < OP_PROGRESS_MIN_INTERVAL {
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
            last_total_bytes = update.total.as_ref().map(|t| t.bytes);

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

        // Ensure the latest progress update is persisted even if it arrived within the throttling
        // window right before the sender was dropped.
        let Some(update) = last_update.or_else(|| *rx.borrow()) else {
            return;
        };

        let total_bytes = update.total.as_ref().map(|t| t.bytes);
        if last_stage == Some(update.stage)
            && last_done_bytes == update.done.bytes
            && last_total_bytes == total_bytes
        {
            return;
        }

        let stage_changed = last_stage != Some(update.stage);
        let now_ts = OffsetDateTime::now_utc().unix_timestamp();

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
                (Some(rate), Some(total)) if rate > 0 && total.bytes > update.done.bytes => Some(
                    total
                        .bytes
                        .saturating_sub(update.done.bytes)
                        .saturating_div(rate),
                ),
                _ => None,
            };
            (rate, eta)
        };

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

        if let Ok(payload) = serde_json::to_value(snapshot) {
            let _ = operations_repo::set_operation_progress(&db, &op_id, Some(payload)).await;
        }
    });

    tx
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use bastion_storage::db;
    use bastion_storage::operations_repo::{OperationKind, create_operation, get_operation};
    use tempfile::TempDir;

    #[tokio::test]
    async fn flushes_last_progress_update_on_drop() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        let op = create_operation(&pool, OperationKind::Restore, None)
            .await
            .expect("create op");

        let tx =
            spawn_operation_progress_writer(pool.clone(), op.id.clone(), ProgressKindV1::Restore);

        let _ = tx.send(Some(OperationProgressUpdate {
            stage: "restore",
            done: ProgressUnitsV1 {
                files: 0,
                dirs: 0,
                bytes: 94,
            },
            total: Some(ProgressUnitsV1 {
                files: 0,
                dirs: 0,
                bytes: 100,
            }),
        }));

        // Immediately hit 100% and drop the sender. Historically this could be lost due to
        // throttling, leaving the stored progress stuck below 100%.
        let _ = tx.send(Some(OperationProgressUpdate {
            stage: "restore",
            done: ProgressUnitsV1 {
                files: 0,
                dirs: 0,
                bytes: 100,
            },
            total: Some(ProgressUnitsV1 {
                files: 0,
                dirs: 0,
                bytes: 100,
            }),
        }));
        drop(tx);

        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                let op = get_operation(&pool, &op.id)
                    .await
                    .expect("get op")
                    .expect("present");
                if let Some(progress) = op.progress {
                    let snap: ProgressSnapshotV1 =
                        serde_json::from_value(progress).expect("deserialize");
                    if snap.stage == "restore" && snap.total.is_some_and(|t| t.bytes == 100) {
                        assert_eq!(snap.done.bytes, 100);
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("timeout waiting for final progress");
    }
}
