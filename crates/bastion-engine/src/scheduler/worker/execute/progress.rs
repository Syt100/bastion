use std::time::{Duration, Instant};

use sqlx::SqlitePool;
use time::OffsetDateTime;

use bastion_core::progress::{ProgressKindV1, ProgressSnapshotV1, ProgressUnitsV1};
use bastion_storage::runs_repo;

pub(super) const RUN_PROGRESS_MIN_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub(super) struct RunProgressUpdate {
    pub(super) stage: &'static str,
    pub(super) done: ProgressUnitsV1,
    pub(super) total: Option<ProgressUnitsV1>,
    pub(super) detail: Option<serde_json::Value>,
}

pub(super) fn spawn_run_progress_writer(
    db: SqlitePool,
    run_id: String,
    kind: ProgressKindV1,
) -> tokio::sync::watch::Sender<Option<RunProgressUpdate>> {
    let (tx, mut rx) = tokio::sync::watch::channel::<Option<RunProgressUpdate>>(None);

    tokio::spawn(async move {
        let mut last_emit = Instant::now()
            .checked_sub(RUN_PROGRESS_MIN_INTERVAL)
            .unwrap_or_else(Instant::now);
        let mut last_stage: Option<&'static str> = None;
        let mut last_ts: Option<i64> = None;
        let mut last_done_bytes: u64 = 0;
        let mut last_update: Option<RunProgressUpdate> = None;

        while rx.changed().await.is_ok() {
            let Some(update) = rx.borrow().clone() else {
                continue;
            };
            last_update = Some(update.clone());

            let stage_changed = last_stage != Some(update.stage);
            let finished = update
                .total
                .as_ref()
                .is_some_and(|t| update.done.bytes >= t.bytes);
            let now_ts = OffsetDateTime::now_utc().unix_timestamp();

            if !stage_changed && !finished && last_emit.elapsed() < RUN_PROGRESS_MIN_INTERVAL {
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
                detail: update.detail,
            };

            let Ok(payload) = serde_json::to_value(snapshot) else {
                continue;
            };

            let _ = runs_repo::set_run_progress(&db, &run_id, Some(payload)).await;
        }

        // Ensure the latest progress update is persisted even if it arrived within the throttling
        // window right before the sender was dropped (e.g. upload hits 100% and the run ends).
        let Some(update) = last_update.or_else(|| rx.borrow().clone()) else {
            return;
        };

        let now_ts = OffsetDateTime::now_utc().unix_timestamp();

        let snapshot = ProgressSnapshotV1 {
            v: 1,
            kind,
            stage: update.stage.to_string(),
            ts: now_ts,
            done: update.done,
            total: update.total,
            rate_bps: None,
            eta_seconds: None,
            detail: update.detail,
        };

        if let Ok(payload) = serde_json::to_value(snapshot) {
            // Best-effort final write. Prefer not to regress, but stage changes are already
            // persisted immediately above, and this final write is intended to close the gap.
            let _ = runs_repo::set_run_progress(&db, &run_id, Some(payload)).await;
        }
    });

    tx
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use bastion_storage::db;
    use bastion_storage::runs_repo::{RunStatus, create_run, get_run_progress};
    use tempfile::TempDir;

    #[tokio::test]
    async fn flushes_last_progress_update_on_drop() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        sqlx::query(
            "INSERT INTO jobs (id, name, schedule, overlap_policy, spec_json, created_at, updated_at) VALUES (?, ?, NULL, 'queue', ?, ?, ?)",
        )
        .bind("job1")
        .bind("job1")
        .bind(r#"{"v":1,"type":"filesystem","source":{"root":"/"},"target":{"type":"local_dir","base_dir":"/tmp"}}"#)
        .bind(1000)
        .bind(1000)
        .execute(&pool)
        .await
        .expect("insert job");

        let run = create_run(&pool, "job1", RunStatus::Queued, 1000, None, None, None)
            .await
            .expect("create run");

        let tx = spawn_run_progress_writer(pool.clone(), run.id.clone(), ProgressKindV1::Backup);

        let _ = tx.send(Some(RunProgressUpdate {
            stage: "upload",
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
            detail: None,
        }));

        // Immediately hit 100% and drop the sender. Historically this could be lost due to
        // throttling, leaving the stored progress stuck below 100%.
        let _ = tx.send(Some(RunProgressUpdate {
            stage: "upload",
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
            detail: None,
        }));
        drop(tx);

        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if let Some(v) = get_run_progress(&pool, &run.id)
                    .await
                    .expect("get progress")
                {
                    let snap: ProgressSnapshotV1 = serde_json::from_value(v).expect("deserialize");
                    if snap.stage == "upload" && snap.total.is_some_and(|t| t.bytes == 100) {
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
