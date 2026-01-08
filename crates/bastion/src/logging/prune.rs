use std::path::{Path, PathBuf};

use tracing::warn;

use crate::config::LogRotation;

use super::file_config::LogFileConfig;
use super::suffix::{is_daily_suffix, is_hourly_suffix};

pub(super) fn spawn_log_prune_loop(log_file: PathBuf, rotation: LogRotation, keep_files: usize) {
    if keep_files == 0 {
        return;
    }
    if rotation == LogRotation::Never {
        return;
    }

    tokio::spawn(async move {
        loop {
            let log_file = log_file.clone();
            let result = tokio::task::spawn_blocking(move || {
                prune_rotated_log_files(&log_file, rotation, keep_files)
            })
            .await;

            match result {
                Ok(Ok(pruned)) => {
                    if pruned > 0 {
                        tracing::info!(pruned, "pruned rotated log files");
                    }
                }
                Ok(Err(error)) => {
                    warn!(error = %error, "failed to prune rotated log files");
                }
                Err(error) => {
                    warn!(error = %error, "failed to join rotated log prune task");
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await;
        }
    });
}

pub(super) fn prune_rotated_log_files(
    log_file: &Path,
    rotation: LogRotation,
    keep_files: usize,
) -> Result<usize, anyhow::Error> {
    if keep_files == 0 {
        return Ok(0);
    }
    let config = LogFileConfig::new(log_file)?;
    let Ok(entries) = std::fs::read_dir(&config.directory) else {
        return Ok(0);
    };

    let mut rotated: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(v) => v,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if is_rotated_name(name, &config.prefix, rotation) {
            rotated.push(path);
        }
    }

    rotated.sort_by(|a, b| {
        let a = a.file_name().and_then(|v| v.to_str()).unwrap_or_default();
        let b = b.file_name().and_then(|v| v.to_str()).unwrap_or_default();
        b.cmp(a)
    });

    if rotated.len() <= keep_files {
        return Ok(0);
    }

    let mut pruned = 0;
    for path in rotated.into_iter().skip(keep_files) {
        if std::fs::remove_file(&path).is_ok() {
            pruned += 1;
        }
    }

    Ok(pruned)
}

fn is_rotated_name(file_name: &str, prefix: &str, rotation: LogRotation) -> bool {
    let Some(suffix) = file_name.strip_prefix(prefix) else {
        return false;
    };
    let Some(suffix) = suffix.strip_prefix('.') else {
        return false;
    };

    match rotation {
        LogRotation::Daily => is_daily_suffix(suffix),
        LogRotation::Hourly => is_hourly_suffix(suffix),
        LogRotation::Never => false,
    }
}
