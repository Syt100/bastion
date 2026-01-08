use std::path::{Path, PathBuf};

pub(in super::super) fn offline_runs_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("agent").join("offline_runs")
}

pub(in super::super) fn offline_run_dir(data_dir: &Path, run_id: &str) -> PathBuf {
    offline_runs_dir(data_dir).join(run_id)
}
