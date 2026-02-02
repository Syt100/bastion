use std::path::{Path, PathBuf};

pub(in super::super) fn offline_runs_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("agent").join("offline_runs")
}

pub(in super::super) fn offline_run_dir(data_dir: &Path, run_id: &str) -> PathBuf {
    offline_runs_dir(data_dir).join(run_id)
}

#[cfg(test)]
mod tests {
    use super::{offline_run_dir, offline_runs_dir};

    #[test]
    fn offline_paths_join_expected_locations() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path();

        assert_eq!(
            offline_runs_dir(base),
            base.join("agent").join("offline_runs")
        );
        assert_eq!(
            offline_run_dir(base, "run1"),
            base.join("agent").join("offline_runs").join("run1")
        );
    }
}
