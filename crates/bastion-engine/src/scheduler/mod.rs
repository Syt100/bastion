use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use bastion_storage::secrets::SecretsCrypto;

use crate::agent_manager::AgentManager;
use crate::run_events_bus::RunEventsBus;

mod artifact_delete;
mod cron;
mod incomplete_cleanup;
mod queue;
mod retention;
mod snapshot_retention;
mod target_snapshot;
mod worker;

pub struct SchedulerArgs {
    pub db: SqlitePool,
    pub data_dir: std::path::PathBuf,
    pub secrets: Arc<SecretsCrypto>,
    pub agent_manager: AgentManager,
    pub run_retention_days: i64,
    pub incomplete_cleanup_days: i64,
    pub run_events_bus: Arc<RunEventsBus>,
    pub run_queue_notify: Arc<Notify>,
    pub incomplete_cleanup_notify: Arc<Notify>,
    pub artifact_delete_notify: Arc<Notify>,
    pub jobs_notify: Arc<Notify>,
    pub notifications_notify: Arc<Notify>,
    pub shutdown: CancellationToken,
}

pub fn spawn(args: SchedulerArgs) {
    let SchedulerArgs {
        db,
        data_dir,
        secrets,
        agent_manager,
        run_retention_days,
        incomplete_cleanup_days,
        run_events_bus,
        run_queue_notify,
        incomplete_cleanup_notify,
        artifact_delete_notify,
        jobs_notify,
        notifications_notify,
        shutdown,
    } = args;

    let agent_manager_cron = agent_manager.clone();
    let agent_manager_worker = agent_manager.clone();
    tokio::spawn(cron::run_cron_loop(
        db.clone(),
        run_events_bus.clone(),
        run_queue_notify.clone(),
        jobs_notify.clone(),
        agent_manager_cron,
        shutdown.clone(),
    ));

    tokio::spawn(worker::run_worker_loop(worker::WorkerLoopArgs {
        db: db.clone(),
        data_dir,
        secrets: secrets.clone(),
        agent_manager: agent_manager_worker,
        run_events_bus: run_events_bus.clone(),
        run_queue_notify: run_queue_notify.clone(),
        notifications_notify: notifications_notify.clone(),
        shutdown: shutdown.clone(),
    }));

    tokio::spawn(retention::run_retention_loop(
        db.clone(),
        run_retention_days,
        shutdown.clone(),
    ));

    tokio::spawn(snapshot_retention::run_snapshot_retention_loop(
        db.clone(),
        artifact_delete_notify.clone(),
        shutdown.clone(),
    ));

    tokio::spawn(artifact_delete::run_artifact_delete_loop(
        db.clone(),
        secrets.clone(),
        agent_manager,
        artifact_delete_notify,
        shutdown.clone(),
    ));

    if incomplete_cleanup_days > 0 {
        tokio::spawn(incomplete_cleanup::run_incomplete_cleanup_loop(
            db.clone(),
            secrets,
            incomplete_cleanup_days,
            incomplete_cleanup_notify,
            shutdown,
        ));
    }
}

pub fn validate_cron(expr: &str) -> Result<(), anyhow::Error> {
    cron::validate_cron(expr)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use tokio::sync::Notify;

    use crate::run_events_bus::RunEventsBus;
    use bastion_storage::db;
    use bastion_storage::jobs_repo::{self, OverlapPolicy};
    use bastion_storage::runs_repo::{self, RunStatus};

    use super::queue::enqueue_run;

    #[tokio::test]
    async fn overlap_policy_reject_inserts_rejected_run() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        let job = jobs_repo::create_job(
            &pool,
            "job1",
            None,
            None,
            Some("UTC"),
            OverlapPolicy::Reject,
            serde_json::json!({
                "v": 1,
                "type": "filesystem",
                "source": { "root": "/" },
                "target": { "type": "local_dir", "base_dir": "/tmp" }
            }),
        )
        .await
        .expect("create job");

        // Existing running run triggers rejection.
        let _existing =
            runs_repo::create_run(&pool, &job.id, RunStatus::Running, 1, None, None, None)
                .await
                .expect("existing run");

        let bus = RunEventsBus::new_with_options(8, 60, 1);
        let notify = Notify::new();
        enqueue_run(&pool, &bus, &notify, &job, "cron")
            .await
            .expect("enqueue");

        let runs = runs_repo::list_runs_for_job(&pool, &job.id, 10)
            .await
            .expect("list runs");
        let newest = &runs[0];
        assert_eq!(newest.status, RunStatus::Rejected);
        assert!(newest.ended_at.is_some());
        assert_eq!(newest.error.as_deref(), Some("overlap_rejected"));
    }

    #[tokio::test]
    async fn overlap_policy_queue_inserts_queued_run() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        let job = jobs_repo::create_job(
            &pool,
            "job1",
            None,
            None,
            Some("UTC"),
            OverlapPolicy::Queue,
            serde_json::json!({
                "v": 1,
                "type": "filesystem",
                "source": { "root": "/" },
                "target": { "type": "local_dir", "base_dir": "/tmp" }
            }),
        )
        .await
        .expect("create job");

        let _existing =
            runs_repo::create_run(&pool, &job.id, RunStatus::Running, 1, None, None, None)
                .await
                .expect("existing run");

        let bus = RunEventsBus::new_with_options(8, 60, 1);
        let notify = Notify::new();
        enqueue_run(&pool, &bus, &notify, &job, "cron")
            .await
            .expect("enqueue");

        let runs = runs_repo::list_runs_for_job(&pool, &job.id, 10)
            .await
            .expect("list runs");
        let newest = &runs[0];
        assert_eq!(newest.status, RunStatus::Queued);
        assert!(newest.ended_at.is_none());
        assert!(newest.error.is_none());
    }
}
