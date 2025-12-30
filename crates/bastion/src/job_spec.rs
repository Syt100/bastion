use globset::Glob;
use serde::Deserialize;
use url::Url;

pub const JOB_SPEC_VERSION: u32 = 1;

fn default_part_size_bytes() -> u64 {
    256 * 1024 * 1024
}

#[derive(Debug, Deserialize)]
pub struct FilesystemSource {
    pub root: String,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SqliteSource {
    pub path: String,
    #[serde(default)]
    pub integrity_check: bool,
}

#[derive(Debug, Deserialize)]
pub struct VaultwardenSource {
    pub data_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct WebdavTarget {
    #[serde(rename = "type")]
    pub target_type: String,
    pub base_url: String,
    pub secret_name: String,
    #[serde(default = "default_part_size_bytes")]
    pub part_size_bytes: u64,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JobSpecV1 {
    Filesystem {
        v: u32,
        source: FilesystemSource,
        target: WebdavTarget,
    },
    Sqlite {
        v: u32,
        source: SqliteSource,
        target: WebdavTarget,
    },
    Vaultwarden {
        v: u32,
        source: VaultwardenSource,
        target: WebdavTarget,
    },
}

pub fn parse_value(spec: &serde_json::Value) -> Result<JobSpecV1, anyhow::Error> {
    Ok(serde_json::from_value(spec.clone())?)
}

pub fn validate_value(spec: &serde_json::Value) -> Result<(), anyhow::Error> {
    let spec = parse_value(spec)?;
    validate(&spec)
}

pub fn validate(spec: &JobSpecV1) -> Result<(), anyhow::Error> {
    match spec {
        JobSpecV1::Filesystem { v, source, target } => {
            validate_version(*v)?;
            if source.root.trim().is_empty() {
                anyhow::bail!("filesystem.source.root is required");
            }
            validate_globs(&source.include)?;
            validate_globs(&source.exclude)?;
            validate_webdav_target(target)?;
        }
        JobSpecV1::Sqlite { v, source, target } => {
            validate_version(*v)?;
            if source.path.trim().is_empty() {
                anyhow::bail!("sqlite.source.path is required");
            }
            validate_webdav_target(target)?;
        }
        JobSpecV1::Vaultwarden { v, source, target } => {
            validate_version(*v)?;
            if source.data_dir.trim().is_empty() {
                anyhow::bail!("vaultwarden.source.data_dir is required");
            }
            validate_webdav_target(target)?;
        }
    }

    Ok(())
}

fn validate_version(v: u32) -> Result<(), anyhow::Error> {
    if v != JOB_SPEC_VERSION {
        anyhow::bail!("unsupported job spec version");
    }
    Ok(())
}

fn validate_globs(patterns: &[String]) -> Result<(), anyhow::Error> {
    for p in patterns {
        let _ = Glob::new(p)?;
    }
    Ok(())
}

fn validate_webdav_target(target: &WebdavTarget) -> Result<(), anyhow::Error> {
    if target.target_type != "webdav" {
        anyhow::bail!("target.type must be webdav");
    }
    if target.base_url.trim().is_empty() {
        anyhow::bail!("target.base_url is required");
    }
    if target.secret_name.trim().is_empty() {
        anyhow::bail!("target.secret_name is required");
    }
    let url = Url::parse(target.base_url.trim())?;
    if !matches!(url.scheme(), "http" | "https") {
        anyhow::bail!("target.base_url must be http(s)");
    }
    if target.part_size_bytes < 1024 * 1024 {
        anyhow::bail!("target.part_size_bytes must be >= 1048576");
    }
    Ok(())
}
