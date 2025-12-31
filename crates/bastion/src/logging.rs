use std::path::{Path, PathBuf};

use tracing::warn;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

use crate::config::{LogRotation, LoggingArgs};

pub struct LoggingGuard {
    _file_guard: Option<WorkerGuard>,
}

pub fn init(args: &LoggingArgs) -> Result<LoggingGuard, anyhow::Error> {
    let filter = build_filter(args)?;

    use std::io::IsTerminal as _;
    let console_ansi = std::io::stdout().is_terminal();

    let console_layer = tracing_subscriber::fmt::layer()
        .with_ansi(console_ansi)
        .with_writer(std::io::stdout);

    let mut file_guard = None;
    let mut file_layer = None;

    if let Some(log_file) = args.log_file.as_deref() {
        let config = LogFileConfig::new(log_file)?;
        std::fs::create_dir_all(&config.directory)?;

        let rotation = config.rotation(args.log_rotation);
        let appender = tracing_appender::rolling::RollingFileAppender::new(
            rotation,
            &config.directory,
            &config.prefix,
        );
        let (non_blocking, guard) = tracing_appender::non_blocking(appender);
        file_guard = Some(guard);
        file_layer = Some(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking),
        );
    }

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    if let Some(log_file) = args.log_file.clone() {
        spawn_log_prune_loop(log_file, args.log_rotation, args.log_keep_files);
    }

    Ok(LoggingGuard {
        _file_guard: file_guard,
    })
}

fn build_filter(args: &LoggingArgs) -> Result<tracing_subscriber::EnvFilter, anyhow::Error> {
    let filter_str = if let Some(filter) = args.log.as_deref() {
        filter.to_string()
    } else if let Ok(filter) = std::env::var("RUST_LOG") {
        filter
    } else {
        // Conservative defaults: INFO for our code, but avoid noisy HTTP access logs by default.
        "info,tower_http=warn".to_string()
    };

    Ok(tracing_subscriber::EnvFilter::try_new(filter_str)?)
}

#[derive(Debug, Clone)]
struct LogFileConfig {
    directory: PathBuf,
    prefix: String,
}

impl LogFileConfig {
    fn new(path: &Path) -> Result<Self, anyhow::Error> {
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

    fn rotation(&self, rotation: LogRotation) -> tracing_appender::rolling::Rotation {
        match rotation {
            LogRotation::Never => tracing_appender::rolling::Rotation::NEVER,
            LogRotation::Hourly => tracing_appender::rolling::Rotation::HOURLY,
            LogRotation::Daily => tracing_appender::rolling::Rotation::DAILY,
        }
    }
}

fn spawn_log_prune_loop(log_file: PathBuf, rotation: LogRotation, keep_files: usize) {
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

fn prune_rotated_log_files(
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

fn is_daily_suffix(s: &str) -> bool {
    if s.len() != 10 {
        return false;
    }
    let bytes = s.as_bytes();
    for (idx, ch) in bytes.iter().enumerate() {
        match idx {
            4 | 7 => {
                if *ch != b'-' {
                    return false;
                }
            }
            _ => {
                if !ch.is_ascii_digit() {
                    return false;
                }
            }
        }
    }
    true
}

fn is_hourly_suffix(s: &str) -> bool {
    if s.len() != 13 {
        return false;
    }
    let bytes = s.as_bytes();
    for (idx, ch) in bytes.iter().enumerate() {
        match idx {
            4 | 7 | 10 => {
                if *ch != b'-' {
                    return false;
                }
            }
            _ => {
                if !ch.is_ascii_digit() {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_suffix_parsing() {
        assert!(is_daily_suffix("2025-12-31"));
        assert!(!is_daily_suffix("20251231"));
        assert!(!is_daily_suffix("2025-1-01"));
        assert!(!is_daily_suffix("2025-12-3a"));
    }

    #[test]
    fn hourly_suffix_parsing() {
        assert!(is_hourly_suffix("2025-12-31-23"));
        assert!(!is_hourly_suffix("2025-12-31"));
        assert!(!is_hourly_suffix("2025-12-31-2"));
        assert!(!is_hourly_suffix("2025-12-31-aa"));
    }

    #[test]
    fn pruning_keeps_newest_by_name() {
        let dir = tempfile::tempdir().unwrap();
        let log_file = dir.path().join("bastion.log");

        std::fs::write(dir.path().join("bastion.log.2025-01-01"), "x").unwrap();
        std::fs::write(dir.path().join("bastion.log.2025-01-02"), "x").unwrap();
        std::fs::write(dir.path().join("bastion.log.2025-01-03"), "x").unwrap();

        let pruned = prune_rotated_log_files(&log_file, LogRotation::Daily, 2).unwrap();
        assert_eq!(pruned, 1);

        assert!(dir.path().join("bastion.log.2025-01-03").exists());
        assert!(dir.path().join("bastion.log.2025-01-02").exists());
        assert!(!dir.path().join("bastion.log.2025-01-01").exists());
    }

    #[test]
    fn pruning_only_touches_expected_patterns() {
        let dir = tempfile::tempdir().unwrap();
        let log_file = dir.path().join("bastion.log");

        std::fs::write(dir.path().join("bastion.log.2025-01-01"), "x").unwrap();
        std::fs::write(dir.path().join("bastion.log.2025-01-02"), "x").unwrap();
        std::fs::write(dir.path().join("bastion.log.old"), "x").unwrap();

        let pruned = prune_rotated_log_files(&log_file, LogRotation::Daily, 1).unwrap();
        assert_eq!(pruned, 1);

        assert!(!dir.path().join("bastion.log.2025-01-01").exists());
        assert!(dir.path().join("bastion.log.2025-01-02").exists());
        assert!(dir.path().join("bastion.log.old").exists());
    }
}
