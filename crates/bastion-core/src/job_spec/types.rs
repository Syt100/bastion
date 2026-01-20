use serde::{Deserialize, Serialize};

use crate::manifest::ArtifactFormatV1;

fn default_part_size_bytes() -> u64 {
    256 * 1024 * 1024
}

fn default_true() -> bool {
    true
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
    pub format: ArtifactFormatV1,
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
    #[serde(default = "default_true")]
    pub pre_scan: bool,
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
