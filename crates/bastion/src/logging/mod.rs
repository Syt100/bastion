mod file_config;
mod prune;
mod suffix;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

use crate::config::LoggingArgs;

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
        let config = file_config::LogFileConfig::new(log_file)?;
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
        prune::spawn_log_prune_loop(log_file, args.log_rotation, args.log_keep_files);
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

#[cfg(test)]
mod tests;
