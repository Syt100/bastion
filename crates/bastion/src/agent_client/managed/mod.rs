mod config_snapshot;
mod io;
mod paths;
mod secrets_snapshot;
mod task_results;

use bastion_core::agent_protocol::JobConfigV1;
use serde::{Deserialize, Serialize};

pub(super) use config_snapshot::{load_managed_config_snapshot, save_managed_config_snapshot};
pub(super) use secrets_snapshot::{
    load_managed_backup_age_identity, load_managed_webdav_credentials, save_managed_secrets_snapshot,
};
pub(super) use task_results::{load_cached_task_result, save_task_result};
pub(super) use task_results::load_cached_operation_result;

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct ManagedConfigPlainV1 {
    pub(super) v: u32,
    pub(super) snapshot_id: String,
    pub(super) issued_at: i64,
    pub(super) jobs: Vec<JobConfigV1>,
}

#[cfg(test)]
mod tests;
