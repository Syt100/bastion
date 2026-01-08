use std::path::{Path, PathBuf};

use super::super::{MANAGED_CONFIG_FILE_NAME, MANAGED_SECRETS_FILE_NAME};

fn is_safe_task_id(task_id: &str) -> bool {
    !task_id.is_empty()
        && task_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn task_results_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("agent").join("task_results")
}

pub(super) fn managed_secrets_path(data_dir: &Path) -> PathBuf {
    data_dir
        .join("agent")
        .join("managed")
        .join(MANAGED_SECRETS_FILE_NAME)
}

pub(super) fn managed_config_path(data_dir: &Path) -> PathBuf {
    data_dir
        .join("agent")
        .join("managed")
        .join(MANAGED_CONFIG_FILE_NAME)
}

pub(super) fn task_result_path(data_dir: &Path, task_id: &str) -> Option<PathBuf> {
    if !is_safe_task_id(task_id) {
        return None;
    }
    Some(task_results_dir(data_dir).join(format!("{task_id}.json")))
}
