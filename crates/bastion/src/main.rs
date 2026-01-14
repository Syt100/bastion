mod agent_client;
mod config;
mod logging;

use std::sync::Arc;

use clap::{CommandFactory, FromArgMatches as _};
use clap::parser::ValueSource;
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::config::{Cli, Command, KeypackCommand, LogRotation};
use bastion_engine::run_events_bus::RunEventsBus;
use bastion_engine::{agent_manager, maintenance, notifications, scheduler};
use bastion_http::{
    AppState, ConfigValueSource, HubRuntimeConfigMeta, HubRuntimeConfigSources,
    HubRuntimeLoggingEffective,
};
use bastion_storage::hub_runtime_config_repo;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cmd = Cli::command();
    let matches = cmd.get_matches();
    let Cli {
        command,
        hub,
        logging: logging_args,
    } = Cli::from_arg_matches(&matches)?;

    if let Some(command) = command {
        let _logging_guard = logging::init(&logging_args)?;
        info!(
            log_file = ?logging_args.log_file,
            log_rotation = ?logging_args.log_rotation,
            log_keep_files = logging_args.log_keep_files,
            "logging initialized"
        );

        match command {
            Command::Agent(args) => {
                agent_client::run(args).await?;
                return Ok(());
            }
            Command::Keypack { command } => {
                let config = hub.into_config()?;
                match command {
                    KeypackCommand::Export(args) => {
                        let password = read_keypack_password(args.password, args.password_stdin)?;
                        bastion_storage::secrets::export_keypack(
                            &config.data_dir,
                            &args.out,
                            &password,
                        )?;
                        println!("exported keypack to {}", args.out.display());
                    }
                    KeypackCommand::Import(args) => {
                        let password = read_keypack_password(args.password, args.password_stdin)?;
                        bastion_storage::secrets::import_keypack(
                            &config.data_dir,
                            &args.r#in,
                            &password,
                            args.force,
                        )?;
                        println!(
                            "imported keypack into {} (master.key {})",
                            config.data_dir.display(),
                            if args.force { "overwritten" } else { "written" }
                        );
                        println!("restart the service to ensure the new keyring is loaded");
                    }
                    KeypackCommand::Rotate(_) => {
                        let result = bastion_storage::secrets::rotate_master_key(&config.data_dir)?;
                        println!(
                            "rotated master.key: {} -> {} (keys: {})",
                            result.previous_kid, result.active_kid, result.keys_count
                        );
                        println!("restart the service to use the new active key");
                    }
                }
                return Ok(());
            }
        }
    }

    let mut config = hub.into_config()?;
    let pool = bastion_storage::db::init(&config.data_dir).await?;

    let saved = hub_runtime_config_repo::get(&pool)
        .await?
        .unwrap_or_default();

    let mut sources = HubRuntimeConfigSources::default();
    sources.bind_host = map_value_source(matches.value_source("host"));
    sources.bind_port = map_value_source(matches.value_source("port"));
    sources.data_dir = map_value_source(matches.value_source("data_dir"));
    sources.insecure_http = map_value_source(matches.value_source("insecure_http"));
    sources.trusted_proxies = map_value_source(matches.value_source("trusted_proxies"));
    sources.debug_errors = map_value_source(matches.value_source("debug_errors"));

    sources.hub_timezone = map_value_source(matches.value_source("hub_timezone"));
    sources.run_retention_days = map_value_source(matches.value_source("run_retention_days"));
    sources.incomplete_cleanup_days =
        map_value_source(matches.value_source("incomplete_cleanup_days"));

    // Apply DB overrides for safe policy fields when not explicitly set via CLI/ENV.
    if sources.hub_timezone == ConfigValueSource::Default {
        if let Some(tz) = normalize_optional_string(saved.hub_timezone.as_deref())
            .and_then(|v| validate_timezone(&v).ok())
        {
            config.hub_timezone = tz;
            sources.hub_timezone = ConfigValueSource::Db;
        }
    }

    if sources.run_retention_days == ConfigValueSource::Default {
        if let Some(days) = saved.run_retention_days {
            if days > 0 {
                config.run_retention_days = days;
                sources.run_retention_days = ConfigValueSource::Db;
            }
        }
    }

    if sources.incomplete_cleanup_days == ConfigValueSource::Default {
        if let Some(days) = saved.incomplete_cleanup_days {
            if days >= 0 {
                config.incomplete_cleanup_days = days;
                sources.incomplete_cleanup_days = ConfigValueSource::Db;
            }
        }
    }

    let mut effective_logging_args = logging_args.clone();
    let (effective_log_filter, log_filter_source) =
        resolve_log_filter(&matches, saved.log_filter.as_deref(), effective_logging_args.log.as_deref());
    sources.log_filter = log_filter_source;
    if sources.log_filter == ConfigValueSource::Db {
        effective_logging_args.log = Some(effective_log_filter.clone());
    }

    // log file
    sources.log_file = map_value_source(matches.value_source("log_file"));
    if sources.log_file == ConfigValueSource::Default {
        if let Some(path) = normalize_optional_string(saved.log_file.as_deref()) {
            effective_logging_args.log_file = Some(path.into());
            sources.log_file = ConfigValueSource::Db;
        }
    }

    // log rotation
    sources.log_rotation = map_value_source(matches.value_source("log_rotation"));
    if sources.log_rotation == ConfigValueSource::Default {
        if let Some(rot) = normalize_optional_string(saved.log_rotation.as_deref())
            .and_then(|v| parse_log_rotation(&v))
        {
            effective_logging_args.log_rotation = rot;
            sources.log_rotation = ConfigValueSource::Db;
        }
    }

    // log keep files
    sources.log_keep_files = map_value_source(matches.value_source("log_keep_files"));
    if sources.log_keep_files == ConfigValueSource::Default {
        if let Some(keep) = saved.log_keep_files {
            effective_logging_args.log_keep_files = keep;
            sources.log_keep_files = ConfigValueSource::Db;
        }
    }

    let _logging_guard = logging::init(&effective_logging_args)?;
    info!(
        log_file = ?effective_logging_args.log_file,
        log_rotation = ?effective_logging_args.log_rotation,
        log_keep_files = effective_logging_args.log_keep_files,
        "logging initialized"
    );

    let runtime_logging = HubRuntimeLoggingEffective {
        filter: effective_log_filter,
        file: effective_logging_args
            .log_file
            .as_ref()
            .map(|p| p.display().to_string()),
        rotation: log_rotation_to_string(effective_logging_args.log_rotation).to_string(),
        keep_files: effective_logging_args.log_keep_files,
    };

    let hub_runtime_config = HubRuntimeConfigMeta {
        sources,
        logging: runtime_logging,
    };

    let config = Arc::new(config);
    let secrets = Arc::new(bastion_storage::secrets::SecretsCrypto::load_or_create(
        &config.data_dir,
    )?);
    let master_kid = secrets.active_kid();
    let agent_manager = agent_manager::AgentManager::default();
    let run_events_bus = Arc::new(RunEventsBus::new());
    let run_queue_notify = Arc::new(tokio::sync::Notify::new());
    let incomplete_cleanup_notify = Arc::new(tokio::sync::Notify::new());
    let jobs_notify = Arc::new(tokio::sync::Notify::new());
    let notifications_notify = Arc::new(tokio::sync::Notify::new());
    let shutdown = CancellationToken::new();

    scheduler::spawn(scheduler::SchedulerArgs {
        db: pool.clone(),
        data_dir: config.data_dir.clone(),
        secrets: secrets.clone(),
        agent_manager: agent_manager.clone(),
        run_retention_days: config.run_retention_days,
        incomplete_cleanup_days: config.incomplete_cleanup_days,
        run_events_bus: run_events_bus.clone(),
        run_queue_notify: run_queue_notify.clone(),
        incomplete_cleanup_notify: incomplete_cleanup_notify.clone(),
        jobs_notify: jobs_notify.clone(),
        notifications_notify: notifications_notify.clone(),
        shutdown: shutdown.clone(),
    });
    notifications::spawn(
        pool.clone(),
        secrets.clone(),
        run_events_bus.clone(),
        notifications_notify.clone(),
        shutdown.clone(),
    );
    maintenance::spawn(pool.clone(), shutdown.clone());

    let app = bastion_http::router(AppState {
        config: config.clone(),
        db: pool,
        secrets,
        agent_manager,
        run_queue_notify,
        incomplete_cleanup_notify,
        jobs_notify,
        notifications_notify,
        run_events_bus,
        hub_runtime_config,
    });

    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    let addr = listener.local_addr()?;

    info!(
        bind = %addr,
        data_dir = %config.data_dir.display(),
        master_kid,
        insecure_http = config.insecure_http,
        "bastion started"
    );

    let shutdown_signal = shutdown.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("shutdown signal received");
            shutdown_signal.cancel();
        }
    });

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async move { shutdown.cancelled().await })
    .await?;
    Ok(())
}

fn map_value_source(source: Option<ValueSource>) -> ConfigValueSource {
    match source {
        Some(ValueSource::CommandLine) => ConfigValueSource::Cli,
        Some(ValueSource::EnvVariable) => ConfigValueSource::Env,
        Some(ValueSource::DefaultValue) | None => ConfigValueSource::Default,
        Some(_) => ConfigValueSource::Default,
    }
}

fn normalize_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn validate_timezone(value: &str) -> Result<String, ()> {
    let v = value.trim();
    if v.is_empty() {
        return Err(());
    }
    if v.parse::<chrono_tz::Tz>().is_err() {
        return Err(());
    }
    Ok(v.to_string())
}

fn parse_log_rotation(value: &str) -> Option<LogRotation> {
    match value.trim().to_lowercase().as_str() {
        "never" => Some(LogRotation::Never),
        "hourly" => Some(LogRotation::Hourly),
        "daily" => Some(LogRotation::Daily),
        _ => None,
    }
}

fn log_rotation_to_string(value: LogRotation) -> &'static str {
    match value {
        LogRotation::Never => "never",
        LogRotation::Hourly => "hourly",
        LogRotation::Daily => "daily",
    }
}

fn resolve_log_filter(
    matches: &clap::ArgMatches,
    saved: Option<&str>,
    explicit_log: Option<&str>,
) -> (String, ConfigValueSource) {
    // BASTION_LOG / --log
    let explicit_source = map_value_source(matches.value_source("log"));
    if explicit_source != ConfigValueSource::Default {
        return (
            explicit_log.unwrap_or("info,tower_http=warn").to_string(),
            explicit_source,
        );
    }

    // RUST_LOG
    if let Ok(filter) = std::env::var("RUST_LOG") {
        return (filter, ConfigValueSource::EnvRustLog);
    }

    // DB
    if let Some(saved) = normalize_optional_string(saved) {
        return (saved, ConfigValueSource::Db);
    }

    // Default (keep in sync with logging::build_filter)
    ("info,tower_http=warn".to_string(), ConfigValueSource::Default)
}

fn read_keypack_password(
    password: Option<String>,
    password_stdin: bool,
) -> Result<String, anyhow::Error> {
    match (password, password_stdin) {
        (Some(pw), false) => Ok(pw),
        (None, true) => {
            use std::io::Read as _;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            let trimmed = buf.trim_end_matches(&['\r', '\n'][..]).to_string();
            if trimmed.is_empty() {
                anyhow::bail!("password from stdin is empty");
            }
            Ok(trimmed)
        }
        (Some(_), true) => anyhow::bail!("use either --password or --password-stdin, not both"),
        (None, false) => anyhow::bail!("missing password: provide --password or --password-stdin"),
    }
}
