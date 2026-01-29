use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use bastion_config::Config;
use clap::{Args, Parser, Subcommand, ValueEnum};
use ipnet::IpNet;

#[derive(Debug, Parser)]
#[command(
    name = "bastion",
    version,
    about = "Bastion backup server",
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub logging: LoggingArgs,

    #[command(flatten)]
    pub hub: HubArgs,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run a Bastion Agent and connect it to a Hub.
    Agent(AgentArgs),
    /// Inspect effective Hub configuration (values + sources).
    Config(ConfigArgs),
    /// Run diagnostics for common setup issues.
    Doctor(DoctorArgs),
    /// Run Bastion as a Windows Service (used by the MSI installer).
    #[cfg(windows)]
    Service(ServiceArgs),
    /// Manage secrets keypacks in the Hub data directory.
    Keypack {
        #[command(subcommand)]
        command: KeypackCommand,
    },
}

#[cfg(windows)]
#[derive(Debug, Args, Clone)]
pub struct ServiceArgs {
    #[command(subcommand)]
    pub command: ServiceCommand,
}

#[cfg(windows)]
#[derive(Debug, Subcommand, Clone)]
pub enum ServiceCommand {
    /// Run as a Windows Service (internal).
    #[command(hide = true)]
    Run,
}

#[derive(Debug, Args, Clone)]
pub struct ConfigArgs {
    /// Output JSON instead of human-readable text.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct DoctorArgs {
    /// Output JSON instead of human-readable text.
    #[arg(long)]
    pub json: bool,
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

    /// Include debug-only error diagnostics in API responses (dev only).
    ///
    /// When enabled, HTTP 500 `internal_error` responses may include safe diagnostics in `details.debug`.
    #[arg(long, env = "BASTION_DEBUG_ERRORS")]
    pub debug_errors: bool,

    /// Run history retention in days (default: 180).
    #[arg(long, default_value_t = 180, env = "BASTION_RUN_RETENTION_DAYS")]
    pub run_retention_days: i64,

    /// Cleanup incomplete runs (missing complete.json) older than N days (default: 7, 0 disables).
    #[arg(long, default_value_t = 7, env = "BASTION_INCOMPLETE_CLEANUP_DAYS")]
    pub incomplete_cleanup_days: i64,

    /// Hub timezone (IANA), used as the default schedule timezone (default: local system timezone).
    ///
    /// Examples: `UTC`, `Asia/Shanghai`, `America/Los_Angeles`.
    #[arg(long, env = "BASTION_HUB_TIMEZONE")]
    pub hub_timezone: Option<String>,

    /// Trusted proxy IPs/CIDRs that are allowed to set X-Forwarded-* headers.
    ///
    /// Can be specified multiple times: `--trusted-proxy 127.0.0.1/32 --trusted-proxy ::1/128`.
    #[arg(long = "trusted-proxy", env = "BASTION_TRUSTED_PROXIES", value_delimiter = ',', num_args = 0..)]
    pub trusted_proxies: Vec<IpNet>,
}

#[derive(Debug, Args, Clone)]
pub struct LoggingArgs {
    /// Logging filter (same syntax as RUST_LOG), e.g. `info`, `bastion=debug,tower_http=warn`.
    ///
    /// When not set, Bastion defaults to a conservative `info,tower_http=warn` filter.
    #[arg(long, env = "BASTION_LOG")]
    pub log: Option<String>,

    /// Optional log file path. When set, logs are written to both console and file.
    ///
    /// For rotated logs, Bastion uses the file name as a prefix (e.g. `bastion.log.2025-12-31`).
    #[arg(long, env = "BASTION_LOG_FILE")]
    pub log_file: Option<PathBuf>,

    /// Log rotation for `--log-file` (default: daily).
    #[arg(long, env = "BASTION_LOG_ROTATION", value_enum, default_value_t = LogRotation::Daily)]
    pub log_rotation: LogRotation,

    /// How many rotated log files to keep (default: 30, 0 disables pruning).
    #[arg(long, env = "BASTION_LOG_KEEP_FILES", default_value_t = 30)]
    pub log_keep_files: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogRotation {
    Never,
    Hourly,
    Daily,
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
    /// Export a password-encrypted keypack.
    Export(KeypackExportArgs),
    /// Import a password-encrypted keypack.
    Import(KeypackImportArgs),
    /// Rotate the Hub master key (master.key).
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
        let data_dir = bastion_config::data_dir::resolve_data_dir(self.data_dir)?;

        if self.run_retention_days <= 0 {
            anyhow::bail!("run_retention_days must be > 0");
        }
        if self.incomplete_cleanup_days < 0 {
            anyhow::bail!("incomplete_cleanup_days must be >= 0");
        }

        let hub_timezone = match self.hub_timezone {
            Some(v) => {
                let trimmed = v.trim();
                if trimmed.is_empty() {
                    anyhow::bail!("hub_timezone must be non-empty");
                }
                let _ = trimmed
                    .parse::<chrono_tz::Tz>()
                    .map_err(|_| anyhow::anyhow!("invalid hub_timezone: {}", trimmed))?;
                trimmed.to_string()
            }
            None => {
                let detected = iana_time_zone::get_timezone().unwrap_or_else(|_| "UTC".to_string());
                let trimmed = detected.trim();
                if trimmed.is_empty() {
                    "UTC".to_string()
                } else if trimmed.parse::<chrono_tz::Tz>().is_ok() {
                    trimmed.to_string()
                } else {
                    "UTC".to_string()
                }
            }
        };

        let mut trusted_proxies = self.trusted_proxies;
        if trusted_proxies.is_empty() {
            trusted_proxies.push("127.0.0.1/32".parse()?);
            trusted_proxies.push("::1/128".parse()?);
        }

        Ok(Config {
            bind: SocketAddr::new(self.host, self.port),
            data_dir,
            insecure_http: self.insecure_http,
            debug_errors: self.debug_errors,
            hub_timezone,
            run_retention_days: self.run_retention_days,
            incomplete_cleanup_days: self.incomplete_cleanup_days,
            trusted_proxies,
        })
    }
}
