use std::net::SocketAddr;
use std::path::PathBuf;

use ipnet::IpNet;

pub mod data_dir;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub data_dir: PathBuf,
    pub insecure_http: bool,
    pub debug_errors: bool,
    pub hub_timezone: String,
    pub run_retention_days: i64,
    pub incomplete_cleanup_days: i64,
    pub trusted_proxies: Vec<IpNet>,
}
