mod agent;
mod agent_client;
mod agent_manager;
mod agent_protocol;
mod agent_tasks_repo;
mod auth;
mod backup;
mod config;
mod data_dir;
mod db;
mod http;
mod job_spec;
mod jobs_repo;
mod notifications;
mod notifications_repo;
mod operations_repo;
mod restore;
mod runs_repo;
mod scheduler;
mod secrets;
mod secrets_repo;
mod targets;
mod webdav;
mod wecom;

use std::sync::Arc;

use clap::Parser;
use tracing::info;

use crate::config::{Cli, Command};
use crate::http::AppState;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let Cli { command, hub } = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if let Some(Command::Agent(args)) = command {
        agent_client::run(args).await?;
        return Ok(());
    }

    let config = Arc::new(hub.into_config()?);
    let pool = db::init(&config.data_dir).await?;
    let secrets = Arc::new(secrets::SecretsCrypto::load_or_create(&config.data_dir)?);
    let agent_manager = agent_manager::AgentManager::default();

    scheduler::spawn(
        pool.clone(),
        config.data_dir.clone(),
        secrets.clone(),
        agent_manager.clone(),
        config.run_retention_days,
    );
    notifications::spawn(pool.clone(), secrets.clone());

    let app = http::router(AppState {
        config: config.clone(),
        db: pool,
        secrets,
        agent_manager,
    });

    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    let addr = listener.local_addr()?;

    info!(
        bind = %addr,
        data_dir = %config.data_dir.display(),
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
