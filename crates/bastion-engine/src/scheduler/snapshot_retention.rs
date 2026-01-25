use std::sync::Arc;

use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use bastion_core::backup_retention::{RetentionSnapshot, select_retention};
use bastion_core::job_spec;
use bastion_storage::artifact_delete_repo;
use bastion_storage::jobs_repo;
use bastion_storage::run_artifacts_repo;

const LOOP_INTERVAL_SECS: u64 = 60 * 60; // hourly
const RETENTION_SCAN_LIMIT: u64 = 20_000;

fn day_start_utc(ts: i64) -> i64 {
    ts.saturating_div(24 * 60 * 60).saturating_mul(24 * 60 * 60)
}

#[derive(Debug, Default)]
struct TickStats {
    jobs_considered: u64,
    jobs_enabled: u64,
    enqueued: u64,
    skipped_due_to_limits: u64,
}

impl TickStats {
    fn any_activity(&self) -> bool {
        self.enqueued > 0 || self.skipped_due_to_limits > 0
    }
}

pub(super) async fn run_snapshot_retention_loop(
    db: SqlitePool,
    artifact_delete_notify: Arc<Notify>,
    shutdown: CancellationToken,
) {
    loop {
        if shutdown.is_cancelled() {
            break;
        }

        let now = OffsetDateTime::now_utc().unix_timestamp();

        match tick(&db, &artifact_delete_notify, now).await {
            Ok(stats) => {
                if stats.any_activity() {
                    info!(
                        jobs_considered = stats.jobs_considered,
                        jobs_enabled = stats.jobs_enabled,
                        enqueued = stats.enqueued,
                        skipped_due_to_limits = stats.skipped_due_to_limits,
                        "snapshot retention tick"
                    );
                }
            }
            Err(error) => {
                warn!(error = %error, "snapshot retention tick failed");
            }
        }

        tokio::select! {
            _ = shutdown.cancelled() => break,
            _ = tokio::time::sleep(std::time::Duration::from_secs(LOOP_INTERVAL_SECS)) => {}
        }
    }
}

async fn tick(db: &SqlitePool, notify: &Notify, now: i64) -> Result<TickStats, anyhow::Error> {
    let mut stats = TickStats::default();

    let jobs = jobs_repo::list_jobs(db).await?;
    stats.jobs_considered = jobs.len() as u64;

    for job in jobs {
        let parsed = match job_spec::parse_value(&job.spec) {
            Ok(v) => v,
            Err(error) => {
                warn!(job_id = %job.id, error = %error, "invalid job spec; skipping retention");
                continue;
            }
        };

        let retention = parsed.retention();
        if !retention.enabled {
            continue;
        }
        stats.jobs_enabled = stats.jobs_enabled.saturating_add(1);

        let rows =
            run_artifacts_repo::list_retention_items_for_job(db, &job.id, RETENTION_SCAN_LIMIT)
                .await?;

        if rows.is_empty() {
            continue;
        }

        let snapshots = rows
            .iter()
            .map(|r| RetentionSnapshot {
                run_id: r.run_id.clone(),
                ended_at: r.ended_at,
                pinned: r.pinned_at.is_some(),
            })
            .collect::<Vec<_>>();

        let selection = select_retention(retention, now, &snapshots);
        if selection.delete.is_empty() {
            continue;
        }

        let day_start = day_start_utc(now);
        let already = artifact_delete_repo::count_retention_enqueues_for_job_since(db, &job.id, day_start)
            .await?
            .min(u64::from(u32::MAX)) as u32;

        let remaining_day = retention.max_delete_per_day.saturating_sub(already);
        let allowed = std::cmp::min(retention.max_delete_per_tick, remaining_day) as usize;

        if allowed == 0 {
            stats.skipped_due_to_limits = stats
                .skipped_due_to_limits
                .saturating_add(selection.delete.len() as u64);
            continue;
        }

        let mut any_enqueued = false;
        for (idx, d) in selection.delete.iter().enumerate() {
            if idx >= allowed {
                stats.skipped_due_to_limits = stats
                    .skipped_due_to_limits
                    .saturating_add((selection.delete.len().saturating_sub(allowed)) as u64);
                break;
            }

            let Some(artifact) = run_artifacts_repo::get_run_artifact(db, &d.run_id).await? else {
                continue;
            };

            // Already gone -> idempotent no-op.
            if artifact.status == "deleted" || artifact.status == "missing" {
                continue;
            }

            let snapshot_json = serde_json::to_string(&artifact.target_snapshot)?;

            let inserted = artifact_delete_repo::upsert_task_if_missing(
                db,
                &artifact.run_id,
                &artifact.job_id,
                &artifact.node_id,
                &artifact.target_type,
                &snapshot_json,
                now,
            )
            .await?;

            if inserted {
                any_enqueued = true;
                stats.enqueued = stats.enqueued.saturating_add(1);
                let _ = artifact_delete_repo::append_event(
                    db,
                    &artifact.run_id,
                    "info",
                    "retention_queued",
                    "retention delete queued",
                    Some(serde_json::json!({
                        "job_id": job.id,
                        "keep_last": retention.keep_last,
                        "keep_days": retention.keep_days
                    })),
                    now,
                )
                .await;
            }

            let _ = run_artifacts_repo::mark_run_artifact_deleting(db, &artifact.run_id, now).await;
        }

        if any_enqueued {
            notify.notify_one();
        }
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use bastion_storage::db;
    use bastion_storage::jobs_repo::{self, OverlapPolicy};
    use bastion_storage::runs_repo::{self, RunStatus};
    use sqlx::Row;

    use super::tick;

    #[tokio::test]
    async fn tick_enqueues_retention_deletes_and_respects_limits() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();

        let job = jobs_repo::create_job(
            &pool,
            "job",
            None,
            None,
            Some("UTC"),
            OverlapPolicy::Queue,
            serde_json::json!({
              "v": 1,
              "type": "filesystem",
              "retention": { "enabled": true, "keep_last": 1, "keep_days": 0, "max_delete_per_tick": 1, "max_delete_per_day": 2 },
              "source": { "paths": ["/tmp"] },
              "target": { "type": "local_dir", "base_dir": "/tmp" }
            }),
        )
        .await
        .unwrap();

        let now = time::OffsetDateTime::now_utc().unix_timestamp();

        // 3 snapshots -> keep_last=1 keeps newest, 2 are delete candidates but per_tick=1 enqueues 1.
        let run_new = runs_repo::create_run(&pool, &job.id, RunStatus::Success, 1, None, None, None)
            .await
            .unwrap();
        let run_mid = runs_repo::create_run(&pool, &job.id, RunStatus::Success, 1, None, None, None)
            .await
            .unwrap();
        let run_old = runs_repo::create_run(&pool, &job.id, RunStatus::Success, 1, None, None, None)
            .await
            .unwrap();

        for (run_id, ended_at) in [
            (&run_new.id, now - 10),
            (&run_mid.id, now - 20),
            (&run_old.id, now - 30),
        ] {
            sqlx::query(
                r#"
                INSERT INTO run_artifacts (
                  run_id, job_id, node_id, target_type, target_snapshot_json,
                  artifact_format, status, started_at, ended_at,
                  created_at, updated_at
                ) VALUES (?, ?, 'hub', 'local_dir', ?, 'archive_v1', 'present', ?, ?, ?, ?)
                "#,
            )
            .bind(run_id)
            .bind(&job.id)
            .bind(serde_json::json!({ "node_id": "hub", "target": { "type": "local_dir", "base_dir": "/tmp" } }).to_string())
            .bind(now - 100)
            .bind(ended_at)
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await
            .unwrap();
        }

        let notify = tokio::sync::Notify::new();
        let stats = tick(&pool, &notify, now).await.unwrap();
        assert_eq!(stats.enqueued, 1);

        let rows = sqlx::query("SELECT COUNT(*) AS cnt FROM artifact_delete_tasks WHERE job_id = ?")
            .bind(&job.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        let cnt = rows.get::<i64, _>("cnt");
        assert_eq!(cnt, 1);
    }

    #[tokio::test]
    async fn tick_respects_max_delete_per_day_by_counting_retention_events() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();

        let job = jobs_repo::create_job(
            &pool,
            "job",
            None,
            None,
            Some("UTC"),
            OverlapPolicy::Queue,
            serde_json::json!({
              "v": 1,
              "type": "filesystem",
              "retention": { "enabled": true, "keep_last": 1, "keep_days": 0, "max_delete_per_tick": 10, "max_delete_per_day": 1 },
              "source": { "paths": ["/tmp"] },
              "target": { "type": "local_dir", "base_dir": "/tmp" }
            }),
        )
        .await
        .unwrap();

        let now = time::OffsetDateTime::now_utc().unix_timestamp();

        let run_new = runs_repo::create_run(&pool, &job.id, RunStatus::Success, 1, None, None, None)
            .await
            .unwrap();
        let run_old = runs_repo::create_run(&pool, &job.id, RunStatus::Success, 1, None, None, None)
            .await
            .unwrap();

        for (run_id, ended_at) in [(&run_new.id, now - 10), (&run_old.id, now - 20)] {
            sqlx::query(
                r#"
                INSERT INTO run_artifacts (
                  run_id, job_id, node_id, target_type, target_snapshot_json,
                  artifact_format, status, started_at, ended_at,
                  created_at, updated_at
                ) VALUES (?, ?, 'hub', 'local_dir', ?, 'archive_v1', 'present', ?, ?, ?, ?)
                "#,
            )
            .bind(run_id)
            .bind(&job.id)
            .bind(serde_json::json!({ "node_id": "hub", "target": { "type": "local_dir", "base_dir": "/tmp" } }).to_string())
            .bind(now - 100)
            .bind(ended_at)
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await
            .unwrap();
        }

        // Pretend we already enqueued one retention deletion today.
        sqlx::query(
            r#"
            INSERT INTO artifact_delete_tasks (
              run_id, job_id, node_id, target_type, target_snapshot_json,
              status, attempts, created_at, updated_at, next_attempt_at
            ) VALUES (?, ?, 'hub', 'local_dir', ?, 'queued', 0, ?, ?, ?)
            "#,
        )
        .bind(&run_old.id)
        .bind(&job.id)
        .bind(serde_json::json!({ "node_id": "hub", "target": { "type": "local_dir", "base_dir": "/tmp" } }).to_string())
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            INSERT INTO artifact_delete_events (run_id, seq, ts, level, kind, message, fields_json)
            VALUES (?, 1, ?, 'info', 'retention_queued', 'retention delete queued', NULL)
            "#,
        )
        .bind(&run_old.id)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        let notify = tokio::sync::Notify::new();
        let stats = tick(&pool, &notify, now).await.unwrap();

        // max_delete_per_day=1 and we already have 1 retention_queued event -> no new enqueues.
        assert_eq!(stats.enqueued, 0);
        assert_eq!(stats.skipped_due_to_limits, 1);

        let rows = sqlx::query("SELECT COUNT(*) AS cnt FROM artifact_delete_tasks WHERE job_id = ?")
            .bind(&job.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        let cnt = rows.get::<i64, _>("cnt");
        assert_eq!(cnt, 1);
    }
}
