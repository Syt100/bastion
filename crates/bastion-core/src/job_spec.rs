use globset::Glob;
use serde::{Deserialize, Serialize};
use url::Url;

pub const JOB_SPEC_VERSION: u32 = 1;

fn default_part_size_bytes() -> u64 {
    256 * 1024 * 1024
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EncryptionV1 {
    #[default]
    None,
    AgeX25519 {
        key_name: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PipelineV1 {
    #[serde(default)]
    pub encryption: EncryptionV1,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum NotificationsModeV1 {
    #[default]
    Inherit,
    Custom,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NotificationsV1 {
    #[serde(default)]
    pub mode: NotificationsModeV1,
    #[serde(default)]
    pub wecom_bot: Vec<String>,
    #[serde(default)]
    pub email: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FsSymlinkPolicy {
    #[default]
    Keep,
    Follow,
    Skip,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FsHardlinkPolicy {
    #[default]
    Copy,
    Keep,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FsErrorPolicy {
    #[default]
    FailFast,
    SkipFail,
    SkipOk,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesystemSource {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub root: String,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub symlink_policy: FsSymlinkPolicy,
    #[serde(default)]
    pub hardlink_policy: FsHardlinkPolicy,
    #[serde(default)]
    pub error_policy: FsErrorPolicy,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqliteSource {
    pub path: String,
    #[serde(default)]
    pub integrity_check: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultwardenSource {
    pub data_dir: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TargetV1 {
    Webdav {
        base_url: String,
        secret_name: String,
        #[serde(default = "default_part_size_bytes")]
        part_size_bytes: u64,
    },
    LocalDir {
        base_dir: String,
        #[serde(default = "default_part_size_bytes")]
        part_size_bytes: u64,
    },
}

impl TargetV1 {
    pub fn part_size_bytes(&self) -> u64 {
        match self {
            TargetV1::Webdav {
                part_size_bytes, ..
            } => *part_size_bytes,
            TargetV1::LocalDir {
                part_size_bytes, ..
            } => *part_size_bytes,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JobSpecV1 {
    Filesystem {
        v: u32,
        #[serde(default)]
        pipeline: PipelineV1,
        #[serde(default)]
        notifications: NotificationsV1,
        source: FilesystemSource,
        target: TargetV1,
    },
    Sqlite {
        v: u32,
        #[serde(default)]
        pipeline: PipelineV1,
        #[serde(default)]
        notifications: NotificationsV1,
        source: SqliteSource,
        target: TargetV1,
    },
    Vaultwarden {
        v: u32,
        #[serde(default)]
        pipeline: PipelineV1,
        #[serde(default)]
        notifications: NotificationsV1,
        source: VaultwardenSource,
        target: TargetV1,
    },
}

impl JobSpecV1 {
    pub fn notifications(&self) -> &NotificationsV1 {
        match self {
            JobSpecV1::Filesystem { notifications, .. } => notifications,
            JobSpecV1::Sqlite { notifications, .. } => notifications,
            JobSpecV1::Vaultwarden { notifications, .. } => notifications,
        }
    }
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
        JobSpecV1::Filesystem {
            v,
            pipeline,
            notifications,
            source,
            target,
        } => {
            validate_version(*v)?;
            validate_pipeline(pipeline)?;
            validate_notifications(notifications)?;
            let has_paths = source.paths.iter().any(|p| !p.trim().is_empty());
            let has_root = !source.root.trim().is_empty();
            if !has_paths && !has_root {
                anyhow::bail!("filesystem.source.paths (or legacy filesystem.source.root) is required");
            }
            validate_globs(&source.include)?;
            validate_globs(&source.exclude)?;
            validate_target(target)?;
        }
        JobSpecV1::Sqlite {
            v,
            pipeline,
            notifications,
            source,
            target,
        } => {
            validate_version(*v)?;
            validate_pipeline(pipeline)?;
            validate_notifications(notifications)?;
            if source.path.trim().is_empty() {
                anyhow::bail!("sqlite.source.path is required");
            }
            validate_target(target)?;
        }
        JobSpecV1::Vaultwarden {
            v,
            pipeline,
            notifications,
            source,
            target,
        } => {
            validate_version(*v)?;
            validate_pipeline(pipeline)?;
            validate_notifications(notifications)?;
            if source.data_dir.trim().is_empty() {
                anyhow::bail!("vaultwarden.source.data_dir is required");
            }
            validate_target(target)?;
        }
    }

    Ok(())
}

fn validate_pipeline(pipeline: &PipelineV1) -> Result<(), anyhow::Error> {
    match &pipeline.encryption {
        EncryptionV1::None => {}
        EncryptionV1::AgeX25519 { key_name } => {
            if key_name.trim().is_empty() {
                anyhow::bail!("pipeline.encryption.key_name is required");
            }
        }
    }
    Ok(())
}

fn validate_notifications(notifications: &NotificationsV1) -> Result<(), anyhow::Error> {
    if notifications.mode == NotificationsModeV1::Custom {
        for name in notifications
            .wecom_bot
            .iter()
            .chain(notifications.email.iter())
        {
            if name.trim().is_empty() {
                anyhow::bail!("notifications destination name is required");
            }
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

fn validate_target(target: &TargetV1) -> Result<(), anyhow::Error> {
    match target {
        TargetV1::Webdav {
            base_url,
            secret_name,
            part_size_bytes,
        } => {
            if base_url.trim().is_empty() {
                anyhow::bail!("target.base_url is required");
            }
            if secret_name.trim().is_empty() {
                anyhow::bail!("target.secret_name is required");
            }
            let url = Url::parse(base_url.trim())?;
            if !matches!(url.scheme(), "http" | "https") {
                anyhow::bail!("target.base_url must be http(s)");
            }
            if *part_size_bytes < 1024 * 1024 {
                anyhow::bail!("target.part_size_bytes must be >= 1048576");
            }
        }
        TargetV1::LocalDir {
            base_dir,
            part_size_bytes,
        } => {
            if base_dir.trim().is_empty() {
                anyhow::bail!("target.base_dir is required");
            }
            if *part_size_bytes < 1024 * 1024 {
                anyhow::bail!("target.part_size_bytes must be >= 1048576");
            }
        }
    }
    Ok(())
}
