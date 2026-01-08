use std::path::{Path, PathBuf};

use crate::config::LogRotation;

#[derive(Debug, Clone)]
pub(super) struct LogFileConfig {
    pub(super) directory: PathBuf,
    pub(super) prefix: String,
}

impl LogFileConfig {
    pub(super) fn new(path: &Path) -> Result<Self, anyhow::Error> {
        let prefix = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("log file path must include a file name"))?
            .to_string_lossy()
            .to_string();

        let directory = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        Ok(Self { directory, prefix })
    }

    pub(super) fn rotation(&self, rotation: LogRotation) -> tracing_appender::rolling::Rotation {
        match rotation {
            LogRotation::Never => tracing_appender::rolling::Rotation::NEVER,
            LogRotation::Hourly => tracing_appender::rolling::Rotation::HOURLY,
            LogRotation::Daily => tracing_appender::rolling::Rotation::DAILY,
        }
    }
}
