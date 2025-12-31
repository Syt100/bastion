mod agent;
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

use crate::{config::Cli, http::AppState};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Arc::new(cli.into_config()?);
    let pool = db::init(&config.data_dir).await?;
    let secrets = Arc::new(secrets::SecretsCrypto::load_or_create(&config.data_dir)?);

    scheduler::spawn(
        pool.clone(),
        config.data_dir.clone(),
        secrets.clone(),
        config.run_retention_days,
    );
    notifications::spawn(pool.clone(), secrets.clone());

    let app = http::router(AppState {
        config: config.clone(),
        db: pool,
        secrets,
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
