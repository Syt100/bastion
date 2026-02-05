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

fn default_max_delete_per_tick() -> u32 {
    50
}

fn default_max_delete_per_day() -> u32 {
    200
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RetentionPolicyV1 {
    /// When disabled, retention selection MUST be a no-op.
    #[serde(default)]
    pub enabled: bool,
    /// Keep the last N snapshots (ordered by ended_at DESC). `None`/`0` means disabled.
    #[serde(default)]
    pub keep_last: Option<u32>,
    /// Keep snapshots within the last D days. `None`/`0` means disabled.
    #[serde(default)]
    pub keep_days: Option<u32>,
    /// Safety valve: limit how many snapshots retention can enqueue per loop tick.
    #[serde(default = "default_max_delete_per_tick")]
    pub max_delete_per_tick: u32,
    /// Safety valve: limit how many snapshots retention can enqueue per day (UTC).
    #[serde(default = "default_max_delete_per_day")]
    pub max_delete_per_day: u32,
}

impl Default for RetentionPolicyV1 {
    fn default() -> Self {
        Self {
            enabled: false,
            keep_last: None,
            keep_days: None,
            max_delete_per_tick: default_max_delete_per_tick(),
            max_delete_per_day: default_max_delete_per_day(),
        }
    }
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotModeV1 {
    #[default]
    Off,
    Auto,
    Required,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConsistencyPolicyV1 {
    #[default]
    Warn,
    Fail,
    Ignore,
}

impl ConsistencyPolicyV1 {
    pub fn should_emit_warnings(self) -> bool {
        self != Self::Ignore
    }

    pub fn should_fail(self, total: u64, threshold: u64) -> bool {
        self == Self::Fail && total > threshold
    }
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
    #[serde(default)]
    pub snapshot_mode: SnapshotModeV1,
    #[serde(default)]
    pub snapshot_provider: Option<String>,
    #[serde(default)]
    pub consistency_policy: ConsistencyPolicyV1,
    #[serde(default)]
    pub consistency_fail_threshold: Option<u64>,
    #[serde(default)]
    pub upload_on_consistency_failure: Option<bool>,
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
    #[serde(default)]
    pub consistency_policy: ConsistencyPolicyV1,
    #[serde(default)]
    pub consistency_fail_threshold: Option<u64>,
    #[serde(default)]
    pub upload_on_consistency_failure: Option<bool>,
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
        #[serde(default)]
        retention: RetentionPolicyV1,
        source: FilesystemSource,
        target: TargetV1,
    },
    Sqlite {
        v: u32,
        #[serde(default)]
        pipeline: PipelineV1,
        #[serde(default)]
        notifications: NotificationsV1,
        #[serde(default)]
        retention: RetentionPolicyV1,
        source: SqliteSource,
        target: TargetV1,
    },
    Vaultwarden {
        v: u32,
        #[serde(default)]
        pipeline: PipelineV1,
        #[serde(default)]
        notifications: NotificationsV1,
        #[serde(default)]
        retention: RetentionPolicyV1,
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

    pub fn retention(&self) -> &RetentionPolicyV1 {
        match self {
            JobSpecV1::Filesystem { retention, .. } => retention,
            JobSpecV1::Sqlite { retention, .. } => retention,
            JobSpecV1::Vaultwarden { retention, .. } => retention,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::manifest::ArtifactFormatV1;

    #[test]
    fn filesystem_source_defaults_are_stable() -> Result<(), anyhow::Error> {
        let src: FilesystemSource = serde_json::from_value(serde_json::json!({}))?;
        assert!(src.pre_scan);
        assert!(src.paths.is_empty());
        assert_eq!(src.root, "");
        assert!(src.include.is_empty());
        assert!(src.exclude.is_empty());
        assert_eq!(src.symlink_policy, FsSymlinkPolicy::Keep);
        assert_eq!(src.hardlink_policy, FsHardlinkPolicy::Copy);
        assert_eq!(src.error_policy, FsErrorPolicy::FailFast);
        Ok(())
    }

    #[test]
    fn target_defaults_part_size_bytes_when_missing() -> Result<(), anyhow::Error> {
        let target: TargetV1 = serde_json::from_value(serde_json::json!({
            "type": "local_dir",
            "base_dir": "/tmp"
        }))?;
        assert_eq!(target.part_size_bytes(), 256 * 1024 * 1024);

        let target: TargetV1 = serde_json::from_value(serde_json::json!({
            "type": "webdav",
            "base_url": "https://example.invalid/",
            "secret_name": "s",
            "part_size_bytes": 123
        }))?;
        assert_eq!(target.part_size_bytes(), 123);
        Ok(())
    }

    #[test]
    fn pipeline_and_retention_defaults_are_stable() -> Result<(), anyhow::Error> {
        let p: PipelineV1 = serde_json::from_value(serde_json::json!({}))?;
        assert_eq!(p.format, ArtifactFormatV1::ArchiveV1);
        assert!(matches!(p.encryption, EncryptionV1::None));

        let r: RetentionPolicyV1 = serde_json::from_value(serde_json::json!({}))?;
        assert!(!r.enabled);
        assert_eq!(r.keep_last, None);
        assert_eq!(r.keep_days, None);
        assert_eq!(r.max_delete_per_tick, 50);
        assert_eq!(r.max_delete_per_day, 200);
        Ok(())
    }

    #[test]
    fn notifications_defaults_are_stable() -> Result<(), anyhow::Error> {
        let n: NotificationsV1 = serde_json::from_value(serde_json::json!({}))?;
        assert_eq!(n.mode, NotificationsModeV1::Inherit);
        assert!(n.wecom_bot.is_empty());
        assert!(n.email.is_empty());
        Ok(())
    }

    #[test]
    fn job_spec_parses_with_defaults() -> Result<(), anyhow::Error> {
        let spec: JobSpecV1 = serde_json::from_value(serde_json::json!({
            "type": "filesystem",
            "v": 1,
            "source": {
                "paths": ["/tmp"]
            },
            "target": {
                "type": "local_dir",
                "base_dir": "/tmp"
            }
        }))?;

        let JobSpecV1::Filesystem {
            v,
            pipeline,
            notifications,
            retention,
            source,
            target,
        } = spec
        else {
            anyhow::bail!("expected filesystem spec");
        };

        assert_eq!(v, 1);
        assert_eq!(pipeline.format, ArtifactFormatV1::ArchiveV1);
        assert_eq!(notifications.mode, NotificationsModeV1::Inherit);
        assert_eq!(retention.max_delete_per_day, 200);
        assert!(source.pre_scan);
        assert_eq!(source.paths, vec!["/tmp"]);
        assert_eq!(target.part_size_bytes(), 256 * 1024 * 1024);
        Ok(())
    }
}
