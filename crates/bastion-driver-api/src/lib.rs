use std::fmt::{Display, Formatter};
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use bastion_core::backup_format::LocalRunArtifacts;
use serde::{Deserialize, Serialize};

pub type DriverFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DriverId {
    pub kind: String,
    pub version: u32,
}

impl DriverId {
    pub fn new(kind: impl Into<String>, version: u32) -> Result<Self, DriverError> {
        let kind = kind.into();
        if kind.trim().is_empty() {
            return Err(DriverError::config("driver kind is required"));
        }
        if version == 0 {
            return Err(DriverError::config("driver version must be >= 1"));
        }
        Ok(Self { kind, version })
    }

    pub fn key(&self) -> String {
        format!("{}@{}", self.kind, self.version)
    }
}

impl Display for DriverId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.kind, self.version)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriverErrorKind {
    Unsupported,
    Config,
    Auth,
    Network,
    Io,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriverError {
    pub kind: DriverErrorKind,
    pub message: String,
}

impl DriverError {
    pub fn new(kind: DriverErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(DriverErrorKind::Unsupported, message)
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::new(DriverErrorKind::Config, message)
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(DriverErrorKind::Auth, message)
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::new(DriverErrorKind::Network, message)
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(DriverErrorKind::Io, message)
    }

    pub fn unknown(message: impl Into<String>) -> Self {
        Self::new(DriverErrorKind::Unknown, message)
    }
}

impl Display for DriverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DriverError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SourceDriverCapabilities {
    #[serde(default)]
    pub supports_snapshots: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TargetDriverCapabilities {
    #[serde(default)]
    pub supports_archive_rolling_upload: bool,
    #[serde(default)]
    pub supports_raw_tree_direct_upload: bool,
    #[serde(default)]
    pub supports_cleanup_run: bool,
    #[serde(default)]
    pub supports_restore_reader: bool,
}

pub trait SourceDriver: Send + Sync {
    fn id(&self) -> &DriverId;
    fn capabilities(&self) -> SourceDriverCapabilities;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TargetRequestLimits {
    #[serde(default)]
    pub concurrency: Option<u32>,
    #[serde(default)]
    pub put_qps: Option<u32>,
    #[serde(default)]
    pub head_qps: Option<u32>,
    #[serde(default)]
    pub mkcol_qps: Option<u32>,
    #[serde(default)]
    pub burst: Option<u32>,
    #[serde(default)]
    pub request_timeout_secs: Option<u64>,
    #[serde(default)]
    pub connect_timeout_secs: Option<u64>,
    #[serde(default)]
    pub max_put_attempts: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct OpenReaderRequest {
    pub job_id: String,
    pub run_id: String,
    pub target_config: serde_json::Value,
}

pub trait TargetRunReader: Send + Sync {
    fn target_kind(&self) -> &str;

    fn describe_location(&self) -> String;

    fn local_run_dir(&self) -> Option<PathBuf> {
        None
    }

    fn complete_exists(&self) -> DriverFuture<Result<bool, DriverError>>;

    fn read_bytes(&self, artifact_path: String) -> DriverFuture<Result<Vec<u8>, DriverError>>;

    fn head_size(&self, artifact_path: String) -> DriverFuture<Result<Option<u64>, DriverError>>;

    fn get_to_file(
        &self,
        artifact_path: String,
        dest: PathBuf,
        expected_size: Option<u64>,
        retries: usize,
    ) -> DriverFuture<Result<u64, DriverError>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreRunProgress {
    pub bytes_done: u64,
    pub bytes_total: Option<u64>,
}

pub type ProgressCallback = Arc<dyn Fn(StoreRunProgress) + Send + Sync>;

#[derive(Clone)]
pub struct StoreRunRequest {
    pub job_id: String,
    pub run_id: String,
    pub target_config: serde_json::Value,
    pub artifacts: LocalRunArtifacts,
    pub limits: Option<TargetRequestLimits>,
    pub on_progress: Option<ProgressCallback>,
}

impl std::fmt::Debug for StoreRunRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreRunRequest")
            .field("job_id", &self.job_id)
            .field("run_id", &self.run_id)
            .field("target_config", &self.target_config)
            .field("artifacts", &self.artifacts)
            .field("limits", &self.limits)
            .field(
                "on_progress",
                &self.on_progress.as_ref().map(|_| "<callback>"),
            )
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupRunStatus {
    Deleted,
    SkipComplete,
    SkipNotFound,
}

#[derive(Debug, Clone)]
pub struct CleanupRunRequest {
    pub job_id: String,
    pub run_id: String,
    pub target_snapshot: serde_json::Value,
}

pub trait TargetDriver: Send + Sync {
    fn id(&self) -> &DriverId;
    fn capabilities(&self) -> TargetDriverCapabilities;

    fn store_run(
        &self,
        request: StoreRunRequest,
    ) -> DriverFuture<Result<serde_json::Value, DriverError>>;

    fn open_reader(
        &self,
        _request: OpenReaderRequest,
    ) -> Result<Arc<dyn TargetRunReader>, DriverError> {
        Err(DriverError::unsupported("open_reader is not implemented"))
    }

    fn cleanup_run(
        &self,
        _request: CleanupRunRequest,
    ) -> DriverFuture<Result<CleanupRunStatus, DriverError>> {
        Box::pin(async { Err(DriverError::unsupported("cleanup_run is not implemented")) })
    }

    fn snapshot_redacted(
        &self,
        target_config: &serde_json::Value,
    ) -> Result<serde_json::Value, DriverError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_id_rejects_empty_kind() {
        let err = DriverId::new("", 1).expect_err("must fail");
        assert_eq!(err.kind, DriverErrorKind::Config);
    }

    #[test]
    fn driver_id_rejects_zero_version() {
        let err = DriverId::new("webdav", 0).expect_err("must fail");
        assert_eq!(err.kind, DriverErrorKind::Config);
    }

    #[test]
    fn driver_id_key_includes_kind_and_version() {
        let id = DriverId::new("local_dir", 2).expect("valid");
        assert_eq!(id.key(), "local_dir@2");
        assert_eq!(id.to_string(), "local_dir@2");
    }

    #[test]
    fn target_capabilities_default_to_safe_false() {
        let caps = TargetDriverCapabilities::default();
        assert!(!caps.supports_archive_rolling_upload);
        assert!(!caps.supports_raw_tree_direct_upload);
        assert!(!caps.supports_cleanup_run);
        assert!(!caps.supports_restore_reader);
    }
}
