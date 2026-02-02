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

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::super::super::{MANAGED_CONFIG_FILE_NAME, MANAGED_SECRETS_FILE_NAME};
    use super::{managed_config_path, managed_secrets_path, task_result_path};

    #[test]
    fn managed_paths_join_expected_locations() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        assert_eq!(
            managed_secrets_path(base),
            base.join("agent")
                .join("managed")
                .join(MANAGED_SECRETS_FILE_NAME)
        );
        assert_eq!(
            managed_config_path(base),
            base.join("agent")
                .join("managed")
                .join(MANAGED_CONFIG_FILE_NAME)
        );
    }

    #[test]
    fn task_result_path_rejects_unsafe_task_ids() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        for bad in ["", " ", "../x", "a/b", "a\\b", "a.b", "a:b", "a?b"] {
            assert!(task_result_path(base, bad).is_none(), "bad: {bad}");
        }
    }

    #[test]
    fn task_result_path_accepts_safe_task_ids() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path();

        let path = task_result_path(base, "task-1_ok").expect("path");
        assert_eq!(
            path,
            base.join("agent")
                .join("task_results")
                .join("task-1_ok.json")
        );
    }
}
