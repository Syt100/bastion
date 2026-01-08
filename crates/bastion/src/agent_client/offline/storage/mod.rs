mod io;
mod paths;
mod types;
mod writer;

use std::path::{Path, PathBuf};

pub(super) fn offline_runs_dir(data_dir: &Path) -> PathBuf {
    paths::offline_runs_dir(data_dir)
}

pub(super) fn offline_run_dir(data_dir: &Path, run_id: &str) -> PathBuf {
    paths::offline_run_dir(data_dir, run_id)
}

pub(super) use types::{OfflineRunEventV1, OfflineRunFileV1, OfflineRunStatusV1};
pub(super) use writer::OfflineRunWriterHandle;
