use sqlx::Row;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio::sync::Notify;

use bastion_storage::jobs_repo::{self, OverlapPolicy};
use bastion_storage::runs_repo::{self, RunStatus};

use crate::run_events;
use crate::run_events_bus::RunEventsBus;

pub(super) async fn enqueue_run(
    db: &SqlitePool,
    run_events_bus: &RunEventsBus,
    run_queue_notify: &Notify,
    job: &jobs_repo::Job,
    source: &str,
) -> anyhow::Result<()> {
    let running_count = sqlx::query(
        "SELECT COUNT(1) AS n FROM runs WHERE job_id = ? AND status IN ('running', 'queued')",
    )
    .bind(&job.id)
    .fetch_one(db)
    .await?
    .get::<i64, _>("n");

    let now = OffsetDateTime::now_utc().unix_timestamp();
    let (status, ended_at, error) =
        if job.overlap_policy == OverlapPolicy::Reject && running_count > 0 {
            (RunStatus::Rejected, Some(now), Some("overlap_rejected"))
        } else {
            (RunStatus::Queued, None, None)
        };

    let run = runs_repo::create_run(db, &job.id, status, now, ended_at, None, error).await?;
    run_events::append_and_broadcast(
        db,
        run_events_bus,
        &run.id,
        "info",
        status.as_str(),
        status.as_str(),
        Some(serde_json::json!({ "source": source })),
    )
    .await?;

    if status == RunStatus::Queued {
        run_queue_notify.notify_one();
    }

    Ok(())
}
