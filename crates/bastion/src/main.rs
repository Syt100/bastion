mod agent;
mod agent_client;
mod agent_manager;
mod agent_protocol;
mod agent_tasks_repo;
mod agents_repo;
mod auth;
mod backup;
mod backup_encryption;
mod config;
mod data_dir;
mod db;
mod http;
mod job_spec;
mod jobs_repo;
mod logging;
mod maintenance;
mod notifications;
mod notifications_repo;
mod operations_repo;
mod restore;
mod run_events;
mod run_events_bus;
mod run_failure;
mod runs_repo;
mod scheduler;
mod secrets;
mod secrets_repo;
mod smtp;
mod targets;
mod webdav;
mod wecom;

use std::sync::Arc;

use clap::Parser;
use tracing::info;

use crate::config::{Cli, Command, KeypackCommand};
use crate::http::AppState;
use crate::run_events_bus::RunEventsBus;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let Cli {
        command,
        hub,
        logging: logging_args,
    } = Cli::parse();

    let _logging_guard = logging::init(&logging_args)?;

    info!(
        log_file = ?logging_args.log_file,
        log_rotation = ?logging_args.log_rotation,
        log_keep_files = logging_args.log_keep_files,
        "logging initialized"
    );

    if let Some(command) = command {
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
                        secrets::export_keypack(&config.data_dir, &args.out, &password)?;
                        println!("exported keypack to {}", args.out.display());
                    }
                    KeypackCommand::Import(args) => {
                        let password = read_keypack_password(args.password, args.password_stdin)?;
                        secrets::import_keypack(
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
                        let result = secrets::rotate_master_key(&config.data_dir)?;
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

    let config = Arc::new(hub.into_config()?);
    let pool = db::init(&config.data_dir).await?;
    let secrets = Arc::new(secrets::SecretsCrypto::load_or_create(&config.data_dir)?);
    let master_kid = secrets.active_kid();
    let agent_manager = agent_manager::AgentManager::default();
    let run_events_bus = Arc::new(RunEventsBus::new());

    scheduler::spawn(
        pool.clone(),
        config.data_dir.clone(),
        secrets.clone(),
        agent_manager.clone(),
        config.run_retention_days,
        config.incomplete_cleanup_days,
        run_events_bus.clone(),
    );
    notifications::spawn(pool.clone(), secrets.clone(), run_events_bus.clone());
    maintenance::spawn(pool.clone());

    let app = http::router(AppState {
        config: config.clone(),
        db: pool,
        secrets,
        agent_manager,
        run_events_bus,
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
    .await?;
    Ok(())
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
