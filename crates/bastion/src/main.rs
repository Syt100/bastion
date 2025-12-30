mod agent;
mod auth;
mod config;
mod data_dir;
mod db;
mod http;
mod secrets;
mod secrets_repo;

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
