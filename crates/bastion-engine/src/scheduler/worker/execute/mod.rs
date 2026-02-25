use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;

use bastion_core::job_spec;
use bastion_storage::jobs_repo;
use bastion_storage::secrets::SecretsCrypto;

use crate::run_events_bus::RunEventsBus;

mod filesystem;
mod planner;
mod progress;
mod rolling_archive;
mod sqlite;
mod vaultwarden;

#[derive(Debug)]
pub(super) struct RunCanceled {
    pub(super) run_id: String,
}

impl std::fmt::Display for RunCanceled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "run canceled: {}", self.run_id)
    }
}

impl std::error::Error for RunCanceled {}

pub(super) fn check_run_canceled(
    run_id: &str,
    cancel_token: &CancellationToken,
) -> Result<(), anyhow::Error> {
    if cancel_token.is_cancelled() {
        return Err(anyhow::Error::new(RunCanceled {
            run_id: run_id.to_string(),
        }));
    }
    Ok(())
}

pub(super) struct ExecuteRunArgs<'a> {
    pub(super) db: &'a SqlitePool,
    pub(super) secrets: &'a SecretsCrypto,
    pub(super) run_events_bus: &'a RunEventsBus,
    pub(super) data_dir: &'a std::path::Path,
    pub(super) job: &'a jobs_repo::Job,
    pub(super) run_id: &'a str,
    pub(super) started_at: OffsetDateTime,
    pub(super) cancel_token: CancellationToken,
    pub(super) spec: job_spec::JobSpecV1,
}

pub(super) async fn execute_run(
    args: ExecuteRunArgs<'_>,
) -> Result<serde_json::Value, anyhow::Error> {
    let ExecuteRunArgs {
        db,
        secrets,
        run_events_bus,
        data_dir,
        job,
        run_id,
        started_at,
        cancel_token,
        spec,
    } = args;

    match spec {
        job_spec::JobSpecV1::Filesystem {
            pipeline,
            source,
            target,
            ..
        } => {
            filesystem::execute_filesystem_run(
                db,
                secrets,
                run_events_bus,
                data_dir,
                job,
                run_id,
                started_at,
                &cancel_token,
                pipeline,
                source,
                target,
            )
            .await
        }
        job_spec::JobSpecV1::Sqlite {
            pipeline,
            source,
            target,
            ..
        } => {
            sqlite::execute_sqlite_run(
                db,
                secrets,
                run_events_bus,
                data_dir,
                job,
                run_id,
                started_at,
                &cancel_token,
                pipeline,
                source,
                target,
            )
            .await
        }
        job_spec::JobSpecV1::Vaultwarden {
            pipeline,
            source,
            target,
            ..
        } => {
            vaultwarden::execute_vaultwarden_run(
                db,
                secrets,
                run_events_bus,
                data_dir,
                job,
                run_id,
                started_at,
                &cancel_token,
                pipeline,
                source,
                target,
            )
            .await
        }
    }
}
