use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use clap::Parser;
use ipnet::IpNet;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub data_dir: PathBuf,
    pub insecure_http: bool,
    pub run_retention_days: i64,
    pub trusted_proxies: Vec<IpNet>,
}

#[derive(Debug, Parser)]
#[command(name = "bastion", version, about = "Bastion backup server (MVP)")]
pub struct Cli {
    /// Bind host (default: 127.0.0.1).
    #[arg(long, default_value_t = IpAddr::V4(Ipv4Addr::LOCALHOST), env = "BASTION_HOST")]
    pub host: IpAddr,

    /// Bind port (default: 9876).
    #[arg(long, default_value_t = 9876, env = "BASTION_PORT")]
    pub port: u16,

    /// Override the data directory (also supports BASTION_DATA_DIR).
    #[arg(long, env = "BASTION_DATA_DIR")]
    pub data_dir: Option<PathBuf>,

    /// Explicitly allow insecure HTTP/WS mode (dev/LAN only).
    #[arg(long, env = "BASTION_INSECURE_HTTP")]
    pub insecure_http: bool,

    /// Run history retention in days (default: 180).
    #[arg(long, default_value_t = 180, env = "BASTION_RUN_RETENTION_DAYS")]
    pub run_retention_days: i64,

    /// Trusted proxy IPs/CIDRs that are allowed to set X-Forwarded-* headers.
    ///
    /// Can be specified multiple times: `--trusted-proxy 127.0.0.1/32 --trusted-proxy ::1/128`.
    #[arg(long = "trusted-proxy", env = "BASTION_TRUSTED_PROXIES", value_delimiter = ',', num_args = 0..)]
    pub trusted_proxies: Vec<IpNet>,
}

impl Cli {
    pub fn into_config(self) -> Result<Config, anyhow::Error> {
        let data_dir = crate::data_dir::resolve_data_dir(self.data_dir)?;

        if self.run_retention_days <= 0 {
            anyhow::bail!("run_retention_days must be > 0");
        }

        let mut trusted_proxies = self.trusted_proxies;
        if trusted_proxies.is_empty() {
            trusted_proxies.push("127.0.0.1/32".parse()?);
            trusted_proxies.push("::1/128".parse()?);
        }

        Ok(Config {
            bind: SocketAddr::new(self.host, self.port),
            data_dir,
            insecure_http: self.insecure_http,
            run_retention_days: self.run_retention_days,
            trusted_proxies,
        })
    }
}
