mod agent_client;
mod config;
mod i18n;
mod logging;
#[cfg(windows)]
mod win_service;

use std::sync::Arc;

use clap::parser::ValueSource;
use clap::{CommandFactory, FromArgMatches as _};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::config::{Cli, Command, ConfigArgs, DoctorArgs, KeypackCommand, LogRotation};
use bastion_engine::run_events_bus::RunEventsBus;
use bastion_engine::{agent_manager, bulk_operations, maintenance, notifications, scheduler};
use bastion_http::{
    AppState, ConfigValueSource, HubRuntimeConfigMeta, HubRuntimeConfigSources,
    HubRuntimeLoggingEffective,
};
use bastion_storage::hub_runtime_config_repo;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let argv: Vec<_> = std::env::args_os().collect();
    let locale = i18n::cli::resolve_cli_locale();
    let cmd = i18n::cli::localize_command(Cli::command(), locale);
    let matches = cmd.get_matches_from(argv);
    let Cli {
        command,
        hub,
        logging: logging_args,
    } = Cli::from_arg_matches(&matches)?;

    if let Some(command) = command {
        match command {
            Command::Agent(args) => {
                let _logging_guard = logging::init(&logging_args)?;
                info!(
                    log_file = ?logging_args.log_file,
                    log_rotation = ?logging_args.log_rotation,
                    log_keep_files = logging_args.log_keep_files,
                    "logging initialized"
                );
                agent_client::run(args).await?;
            }
            Command::Config(args) => {
                run_config_command(args, hub, logging_args, &matches).await?;
            }
            Command::Doctor(args) => {
                run_doctor_command(args, hub, logging_args, &matches).await?;
            }
            #[cfg(windows)]
            Command::Service(args) => {
                win_service::run(args)?;
            }
            Command::Keypack { command } => {
                let _logging_guard = logging::init(&logging_args)?;
                info!(
                    log_file = ?logging_args.log_file,
                    log_rotation = ?logging_args.log_rotation,
                    log_keep_files = logging_args.log_keep_files,
                    "logging initialized"
                );
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
            }
        }
        return Ok(());
    }

    let shutdown = CancellationToken::new();
    spawn_shutdown_signal_handlers(shutdown.clone());
    run_hub(hub, logging_args, &matches, shutdown).await
}

fn spawn_shutdown_signal_handlers(shutdown: CancellationToken) {
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};

            let mut sigterm = match signal(SignalKind::terminate()) {
                Ok(v) => v,
                Err(err) => {
                    tracing::warn!(error = %err, "failed to register SIGTERM handler");
                    // Fall back to Ctrl-C only.
                    if tokio::signal::ctrl_c().await.is_ok() {
                        tracing::info!("shutdown signal received");
                        shutdown.cancel();
                    }
                    return;
                }
            };

            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("shutdown signal received");
                    shutdown.cancel();
                }
                _ = sigterm.recv() => {
                    tracing::info!("shutdown signal received (SIGTERM)");
                    shutdown.cancel();
                }
            }
        }

        #[cfg(not(unix))]
        {
            if tokio::signal::ctrl_c().await.is_ok() {
                tracing::info!("shutdown signal received");
                shutdown.cancel();
            }
        }
    });
}

pub(crate) async fn run_hub(
    hub: crate::config::HubArgs,
    logging_args: crate::config::LoggingArgs,
    matches: &clap::ArgMatches,
    shutdown: CancellationToken,
) -> Result<(), anyhow::Error> {
    let mut config = hub.into_config()?;
    let pool = bastion_storage::db::init(&config.data_dir).await?;

    let saved = hub_runtime_config_repo::get(&pool)
        .await?
        .unwrap_or_default();

    let (hub_runtime_config, effective_logging_args) =
        resolve_hub_runtime_config_meta(&mut config, matches, &saved, logging_args);

    let _logging_guard = logging::init(&effective_logging_args)?;
    info!(
        log_file = ?effective_logging_args.log_file,
        log_rotation = ?effective_logging_args.log_rotation,
        log_keep_files = effective_logging_args.log_keep_files,
        "logging initialized"
    );

    let config = Arc::new(config);
    let secrets = Arc::new(bastion_storage::secrets::SecretsCrypto::load_or_create(
        &config.data_dir,
    )?);
    let master_kid = secrets.active_kid();
    let agent_manager = agent_manager::AgentManager::default();
    let run_events_bus = Arc::new(RunEventsBus::new());
    let run_queue_notify = Arc::new(tokio::sync::Notify::new());
    let incomplete_cleanup_notify = Arc::new(tokio::sync::Notify::new());
    let artifact_delete_notify = Arc::new(tokio::sync::Notify::new());
    let jobs_notify = Arc::new(tokio::sync::Notify::new());
    let notifications_notify = Arc::new(tokio::sync::Notify::new());
    let bulk_ops_notify = Arc::new(tokio::sync::Notify::new());

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
        artifact_delete_notify: artifact_delete_notify.clone(),
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
    bulk_operations::spawn(bulk_operations::BulkOperationsArgs {
        db: pool.clone(),
        secrets: secrets.clone(),
        agent_manager: agent_manager.clone(),
        notify: bulk_ops_notify.clone(),
        shutdown: shutdown.clone(),
    });
    maintenance::spawn(pool.clone(), shutdown.clone());

    let app = bastion_http::router(AppState {
        config: config.clone(),
        db: pool,
        secrets,
        agent_manager,
        run_queue_notify,
        incomplete_cleanup_notify,
        artifact_delete_notify,
        jobs_notify,
        notifications_notify,
        bulk_ops_notify,
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

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async move { shutdown.cancelled().await })
    .await?;
    Ok(())
}

async fn run_config_command(
    args: ConfigArgs,
    hub: crate::config::HubArgs,
    logging_args: crate::config::LoggingArgs,
    matches: &clap::ArgMatches,
) -> Result<(), anyhow::Error> {
    let mut config = hub.into_config()?;
    let pool = bastion_storage::db::init(&config.data_dir).await?;
    let saved = hub_runtime_config_repo::get(&pool)
        .await?
        .unwrap_or_default();

    let (meta, _effective_logging_args) =
        resolve_hub_runtime_config_meta(&mut config, matches, &saved, logging_args);

    let ui = ui_assets_info();
    let docs = docs_assets_info();

    if args.json {
        let out = serde_json::json!({
            "effective": {
                "bind_host": config.bind.ip().to_string(),
                "bind_port": config.bind.port(),
                "bind": config.bind.to_string(),
                "data_dir": config.data_dir.display().to_string(),
                "insecure_http": config.insecure_http,
                "debug_errors": config.debug_errors,
                "trusted_proxies": config.trusted_proxies.iter().map(|v| v.to_string()).collect::<Vec<_>>(),
                "hub_timezone": config.hub_timezone,
                "run_retention_days": config.run_retention_days,
                "incomplete_cleanup_days": config.incomplete_cleanup_days,
                "logging": {
                    "filter": meta.logging.filter,
                    "file": meta.logging.file,
                    "rotation": meta.logging.rotation,
                    "keep_files": meta.logging.keep_files,
                },
                "ui": ui,
                "docs": docs,
            },
            "sources": {
                "bind_host": meta.sources.bind_host,
                "bind_port": meta.sources.bind_port,
                "data_dir": meta.sources.data_dir,
                "insecure_http": meta.sources.insecure_http,
                "trusted_proxies": meta.sources.trusted_proxies,
                "debug_errors": meta.sources.debug_errors,
                "hub_timezone": meta.sources.hub_timezone,
                "run_retention_days": meta.sources.run_retention_days,
                "incomplete_cleanup_days": meta.sources.incomplete_cleanup_days,
                "log_filter": meta.sources.log_filter,
                "log_file": meta.sources.log_file,
                "log_rotation": meta.sources.log_rotation,
                "log_keep_files": meta.sources.log_keep_files,
            }
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("Bastion config (effective)");
    println!(
        "- bind_host: {} ({})",
        config.bind.ip(),
        source_label(meta.sources.bind_host)
    );
    println!(
        "- bind_port: {} ({})",
        config.bind.port(),
        source_label(meta.sources.bind_port)
    );
    println!(
        "- data_dir: {} ({})",
        config.data_dir.display(),
        source_label(meta.sources.data_dir)
    );
    println!(
        "- insecure_http: {} ({})",
        config.insecure_http,
        source_label(meta.sources.insecure_http)
    );
    println!(
        "- debug_errors: {} ({})",
        config.debug_errors,
        source_label(meta.sources.debug_errors)
    );
    println!(
        "- trusted_proxies: {} ({})",
        config
            .trusted_proxies
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        source_label(meta.sources.trusted_proxies)
    );
    println!(
        "- hub_timezone: {} ({})",
        config.hub_timezone,
        source_label(meta.sources.hub_timezone)
    );
    println!(
        "- run_retention_days: {} ({})",
        config.run_retention_days,
        source_label(meta.sources.run_retention_days)
    );
    println!(
        "- incomplete_cleanup_days: {} ({})",
        config.incomplete_cleanup_days,
        source_label(meta.sources.incomplete_cleanup_days)
    );
    println!(
        "- logging.filter: {} ({})",
        meta.logging.filter,
        source_label(meta.sources.log_filter)
    );
    println!(
        "- logging.file: {} ({})",
        meta.logging.file.as_deref().unwrap_or("(none)"),
        source_label(meta.sources.log_file)
    );
    println!(
        "- logging.rotation: {} ({})",
        meta.logging.rotation,
        source_label(meta.sources.log_rotation)
    );
    println!(
        "- logging.keep_files: {} ({})",
        meta.logging.keep_files,
        source_label(meta.sources.log_keep_files)
    );
    print_assets_info("ui", &ui);
    print_assets_info("docs", &docs);

    Ok(())
}

async fn run_doctor_command(
    args: DoctorArgs,
    hub: crate::config::HubArgs,
    logging_args: crate::config::LoggingArgs,
    matches: &clap::ArgMatches,
) -> Result<(), anyhow::Error> {
    let mut checks: Vec<serde_json::Value> = Vec::new();

    let mut config = match hub.into_config() {
        Ok(c) => {
            checks.push(serde_json::json!({
                "id": "config",
                "status": "ok",
                "message": "config parsed"
            }));
            c
        }
        Err(e) => {
            checks.push(serde_json::json!({
                "id": "config",
                "status": "fail",
                "message": format!("config error: {e}")
            }));
            if args.json {
                let out = serde_json::json!({ "ok": false, "checks": checks });
                println!("{}", serde_json::to_string_pretty(&out)?);
                std::process::exit(1);
            }
            for c in &checks {
                println!("[FAIL] {}", c["message"].as_str().unwrap_or("config error"));
            }
            std::process::exit(1);
        }
    };

    // DB open + load saved runtime config (also gives us a consistent "effective" view).
    let pool = match bastion_storage::db::init(&config.data_dir).await {
        Ok(p) => {
            checks.push(serde_json::json!({
                "id": "db",
                "status": "ok",
                "message": "database opened",
            }));
            Some(p)
        }
        Err(e) => {
            checks.push(serde_json::json!({
                "id": "db",
                "status": "fail",
                "message": format!("database error: {e}"),
            }));
            None
        }
    };

    let saved = if let Some(pool) = &pool {
        hub_runtime_config_repo::get(pool)
            .await?
            .unwrap_or_default()
    } else {
        Default::default()
    };

    let (meta, _effective_logging_args) =
        resolve_hub_runtime_config_meta(&mut config, matches, &saved, logging_args);

    checks.push(serde_json::json!({
        "id": "runtime_config",
        "status": "ok",
        "message": "runtime config resolved",
        "sources": {
            "hub_timezone": meta.sources.hub_timezone,
            "run_retention_days": meta.sources.run_retention_days,
            "incomplete_cleanup_days": meta.sources.incomplete_cleanup_days,
            "log_filter": meta.sources.log_filter,
            "log_file": meta.sources.log_file,
            "log_rotation": meta.sources.log_rotation,
            "log_keep_files": meta.sources.log_keep_files,
        }
    }));

    // Secrets/keyring access.
    match bastion_storage::secrets::SecretsCrypto::load_or_create(&config.data_dir) {
        Ok(secrets) => {
            checks.push(serde_json::json!({
                "id": "secrets",
                "status": "ok",
                "message": "secrets keyring loaded",
                "active_kid": secrets.active_kid(),
            }));
        }
        Err(e) => {
            checks.push(serde_json::json!({
                "id": "secrets",
                "status": "fail",
                "message": format!("secrets error: {e}"),
            }));
        }
    }

    // UI/docs assets availability.
    let ui = ui_assets_info();
    checks.push(check_assets("ui", &ui));

    let docs = docs_assets_info();
    checks.push(check_assets("docs", &docs));

    let ok = checks
        .iter()
        .all(|c| c.get("status").and_then(|v| v.as_str()) != Some("fail"));

    if args.json {
        let out = serde_json::json!({
            "ok": ok,
            "checks": checks,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        if !ok {
            std::process::exit(1);
        }
        return Ok(());
    }

    println!("Bastion doctor");
    for c in &checks {
        let status = c
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let msg = c.get("message").and_then(|v| v.as_str()).unwrap_or("");
        match status {
            "ok" => println!("[OK] {msg}"),
            "warn" => println!("[WARN] {msg}"),
            "fail" => println!("[FAIL] {msg}"),
            _ => println!("[{status}] {msg}"),
        }
    }

    if !ok {
        std::process::exit(1);
    }

    Ok(())
}

fn resolve_hub_runtime_config_meta(
    config: &mut bastion_config::Config,
    matches: &clap::ArgMatches,
    saved: &hub_runtime_config_repo::HubRuntimeConfig,
    mut effective_logging_args: crate::config::LoggingArgs,
) -> (HubRuntimeConfigMeta, crate::config::LoggingArgs) {
    let mut sources = HubRuntimeConfigSources {
        bind_host: map_value_source(matches.value_source("host")),
        bind_port: map_value_source(matches.value_source("port")),
        data_dir: map_value_source(matches.value_source("data_dir")),
        insecure_http: map_value_source(matches.value_source("insecure_http")),
        trusted_proxies: map_value_source(matches.value_source("trusted_proxies")),
        debug_errors: map_value_source(matches.value_source("debug_errors")),
        hub_timezone: map_value_source(matches.value_source("hub_timezone")),
        run_retention_days: map_value_source(matches.value_source("run_retention_days")),
        incomplete_cleanup_days: map_value_source(matches.value_source("incomplete_cleanup_days")),
        ..HubRuntimeConfigSources::default()
    };

    // Apply DB overrides for safe policy fields when not explicitly set via CLI/ENV.
    if sources.hub_timezone == ConfigValueSource::Default
        && let Some(tz) = normalize_optional_string(saved.hub_timezone.as_deref())
            .and_then(|v| validate_timezone(&v).ok())
    {
        config.hub_timezone = tz;
        sources.hub_timezone = ConfigValueSource::Db;
    }

    if sources.run_retention_days == ConfigValueSource::Default
        && let Some(days) = saved.run_retention_days
        && days > 0
    {
        config.run_retention_days = days;
        sources.run_retention_days = ConfigValueSource::Db;
    }

    if sources.incomplete_cleanup_days == ConfigValueSource::Default
        && let Some(days) = saved.incomplete_cleanup_days
        && days >= 0
    {
        config.incomplete_cleanup_days = days;
        sources.incomplete_cleanup_days = ConfigValueSource::Db;
    }

    let (effective_log_filter, log_filter_source) = resolve_log_filter(
        matches,
        saved.log_filter.as_deref(),
        effective_logging_args.log.as_deref(),
    );
    sources.log_filter = log_filter_source;
    if sources.log_filter == ConfigValueSource::Db {
        effective_logging_args.log = Some(effective_log_filter.clone());
    }

    // log file
    sources.log_file = map_value_source(matches.value_source("log_file"));
    if sources.log_file == ConfigValueSource::Default
        && let Some(path) = normalize_optional_string(saved.log_file.as_deref())
    {
        effective_logging_args.log_file = Some(path.into());
        sources.log_file = ConfigValueSource::Db;
    }

    // log rotation
    sources.log_rotation = map_value_source(matches.value_source("log_rotation"));
    if sources.log_rotation == ConfigValueSource::Default
        && let Some(rot) = normalize_optional_string(saved.log_rotation.as_deref())
            .and_then(|v| parse_log_rotation(&v))
    {
        effective_logging_args.log_rotation = rot;
        sources.log_rotation = ConfigValueSource::Db;
    }

    // log keep files
    sources.log_keep_files = map_value_source(matches.value_source("log_keep_files"));
    if sources.log_keep_files == ConfigValueSource::Default
        && let Some(keep) = saved.log_keep_files
    {
        effective_logging_args.log_keep_files = keep;
        sources.log_keep_files = ConfigValueSource::Db;
    }

    let runtime_logging = HubRuntimeLoggingEffective {
        filter: effective_log_filter,
        file: effective_logging_args
            .log_file
            .as_ref()
            .map(|p| p.display().to_string()),
        rotation: log_rotation_to_string(effective_logging_args.log_rotation).to_string(),
        keep_files: effective_logging_args.log_keep_files,
    };

    (
        HubRuntimeConfigMeta {
            sources,
            logging: runtime_logging,
        },
        effective_logging_args,
    )
}

fn source_label(source: ConfigValueSource) -> &'static str {
    match source {
        ConfigValueSource::Cli => "cli",
        ConfigValueSource::Env => "env",
        ConfigValueSource::EnvRustLog => "env(RUST_LOG)",
        ConfigValueSource::Db => "db",
        ConfigValueSource::Default => "default",
    }
}

fn ui_assets_info() -> serde_json::Value {
    #[cfg(feature = "embed-ui")]
    {
        serde_json::json!({ "mode": "embedded" })
    }

    #[cfg(not(feature = "embed-ui"))]
    {
        let dir = std::env::var("BASTION_UI_DIR").unwrap_or_else(|_| "ui/dist".to_string());
        serde_json::json!({
            "mode": "filesystem",
            "env": "BASTION_UI_DIR",
            "dir": dir,
            "index": format!("{}/index.html", dir.trim_end_matches('/')),
        })
    }
}

fn docs_assets_info() -> serde_json::Value {
    #[cfg(feature = "embed-docs")]
    {
        serde_json::json!({ "mode": "embedded" })
    }

    #[cfg(not(feature = "embed-docs"))]
    {
        let dir = std::env::var("BASTION_DOCS_DIR")
            .unwrap_or_else(|_| "docs/.vitepress/dist".to_string());
        serde_json::json!({
            "mode": "filesystem",
            "env": "BASTION_DOCS_DIR",
            "dir": dir,
            "index": format!("{}/index.html", dir.trim_end_matches('/')),
        })
    }
}

fn print_assets_info(label: &str, info: &serde_json::Value) {
    let mode = info
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    match mode {
        "embedded" => println!("- {label}: embedded"),
        "filesystem" => {
            let dir = info.get("dir").and_then(|v| v.as_str()).unwrap_or("");
            let env = info.get("env").and_then(|v| v.as_str()).unwrap_or("");
            println!("- {label}: {dir} (env: {env})");
        }
        _ => println!("- {label}: {mode}"),
    }
}

fn check_assets(id: &str, info: &serde_json::Value) -> serde_json::Value {
    let mode = info
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    if mode == "embedded" {
        return serde_json::json!({
            "id": id,
            "status": "ok",
            "message": format!("{id} assets are embedded"),
        });
    }

    if mode != "filesystem" {
        return serde_json::json!({
            "id": id,
            "status": "warn",
            "message": format!("{id} assets mode is {mode}"),
        });
    }

    let index = info.get("index").and_then(|v| v.as_str()).unwrap_or("");
    let ok = !index.is_empty() && std::path::Path::new(index).is_file();
    if ok {
        return serde_json::json!({
            "id": id,
            "status": "ok",
            "message": format!("{id} assets found: {index}"),
        });
    }

    let env = info.get("env").and_then(|v| v.as_str()).unwrap_or("");
    let dir = info.get("dir").and_then(|v| v.as_str()).unwrap_or("");
    serde_json::json!({
        "id": id,
        "status": "fail",
        "message": format!("{id} assets missing: expected {index} (set {env} or build assets into {dir})"),
    })
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
    (
        "info,tower_http=warn".to_string(),
        ConfigValueSource::Default,
    )
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
