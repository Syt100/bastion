use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use clap::{Args, Parser, Subcommand};
use ipnet::IpNet;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub data_dir: PathBuf,
    pub insecure_http: bool,
    pub run_retention_days: i64,
    pub incomplete_cleanup_days: i64,
    pub trusted_proxies: Vec<IpNet>,
}

#[derive(Debug, Parser)]
#[command(name = "bastion", version, about = "Bastion backup server (MVP)")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub hub: HubArgs,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Agent(AgentArgs),
    Keypack {
        #[command(subcommand)]
        command: KeypackCommand,
    },
}

#[derive(Debug, Args, Clone)]
pub struct HubArgs {
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

    /// Cleanup incomplete runs (missing complete.json) older than N days (default: 7, 0 disables).
    #[arg(long, default_value_t = 7, env = "BASTION_INCOMPLETE_CLEANUP_DAYS")]
    pub incomplete_cleanup_days: i64,

    /// Trusted proxy IPs/CIDRs that are allowed to set X-Forwarded-* headers.
    ///
    /// Can be specified multiple times: `--trusted-proxy 127.0.0.1/32 --trusted-proxy ::1/128`.
    #[arg(long = "trusted-proxy", env = "BASTION_TRUSTED_PROXIES", value_delimiter = ',', num_args = 0..)]
    pub trusted_proxies: Vec<IpNet>,
}

#[derive(Debug, Args, Clone)]
pub struct AgentArgs {
    /// Hub base URL, e.g. `http://hub:9876` or `https://hub.example.com`.
    #[arg(long, env = "BASTION_HUB_URL")]
    pub hub_url: String,

    /// Enrollment token (only required when the agent is not enrolled yet).
    #[arg(long, env = "BASTION_AGENT_ENROLL_TOKEN")]
    pub enroll_token: Option<String>,

    /// Friendly agent name (stored on the Hub, optional).
    #[arg(long, env = "BASTION_AGENT_NAME")]
    pub name: Option<String>,

    /// Override the data directory (also supports BASTION_DATA_DIR).
    #[arg(long, env = "BASTION_DATA_DIR")]
    pub data_dir: Option<PathBuf>,

    /// Heartbeat interval in seconds (default: 15).
    #[arg(long, default_value_t = 15, env = "BASTION_AGENT_HEARTBEAT_SECONDS")]
    pub heartbeat_seconds: u64,
}

#[derive(Debug, Subcommand)]
pub enum KeypackCommand {
    Export(KeypackExportArgs),
    Import(KeypackImportArgs),
    Rotate(KeypackRotateArgs),
}

#[derive(Debug, Args, Clone)]
pub struct KeypackExportArgs {
    /// Output path for the password-encrypted keypack.
    #[arg(long)]
    pub out: PathBuf,

    /// Keypack password (not recommended to pass via CLI args; prefer --password-stdin).
    #[arg(long, env = "BASTION_KEYPACK_PASSWORD", hide_env_values = true)]
    pub password: Option<String>,

    /// Read the keypack password from stdin (trailing newline is trimmed).
    #[arg(long)]
    pub password_stdin: bool,
}

#[derive(Debug, Args, Clone)]
pub struct KeypackImportArgs {
    /// Input path of the password-encrypted keypack.
    #[arg(long)]
    pub r#in: PathBuf,

    /// Overwrite existing data_dir/master.key.
    #[arg(long)]
    pub force: bool,

    /// Keypack password (not recommended to pass via CLI args; prefer --password-stdin).
    #[arg(long, env = "BASTION_KEYPACK_PASSWORD", hide_env_values = true)]
    pub password: Option<String>,

    /// Read the keypack password from stdin (trailing newline is trimmed).
    #[arg(long)]
    pub password_stdin: bool,
}

#[derive(Debug, Args, Clone)]
pub struct KeypackRotateArgs {}

impl HubArgs {
    pub fn into_config(self) -> Result<Config, anyhow::Error> {
        let data_dir = crate::data_dir::resolve_data_dir(self.data_dir)?;

        if self.run_retention_days <= 0 {
            anyhow::bail!("run_retention_days must be > 0");
        }
        if self.incomplete_cleanup_days < 0 {
            anyhow::bail!("incomplete_cleanup_days must be >= 0");
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
            incomplete_cleanup_days: self.incomplete_cleanup_days,
            trusted_proxies,
        })
    }
}
