use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use serde::Deserialize;
use url::Url;

use bastion_core::backup_format::{COMPLETE_NAME, ENTRIES_INDEX_NAME, MANIFEST_NAME};
use bastion_driver_api::{
    DriverError, DriverErrorKind, DriverFuture, DriverId, OpenReaderRequest, StoreRunProgress,
    StoreRunRequest, TargetDriver, TargetDriverCapabilities, TargetRequestLimits, TargetRunReader,
};

use crate::DriverRegistry;

pub const BUILTIN_DRIVER_VERSION: u32 = 1;
pub const TARGET_KIND_LOCAL_DIR: &str = "local_dir";
pub const TARGET_KIND_WEBDAV: &str = "webdav";

pub fn local_dir_driver_id() -> DriverId {
    DriverId {
        kind: TARGET_KIND_LOCAL_DIR.to_string(),
        version: BUILTIN_DRIVER_VERSION,
    }
}

pub fn webdav_driver_id() -> DriverId {
    DriverId {
        kind: TARGET_KIND_WEBDAV.to_string(),
        version: BUILTIN_DRIVER_VERSION,
    }
}

pub fn target_registry() -> &'static DriverRegistry {
    static REGISTRY: OnceLock<DriverRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry = DriverRegistry::new();
        registry
            .register_target_driver(Arc::new(LocalDirTargetDriver::new()))
            .expect("register local_dir target driver");
        registry
            .register_target_driver(Arc::new(WebdavTargetDriver::new()))
            .expect("register webdav target driver");
        registry
    })
}

#[derive(Debug, Deserialize)]
struct LocalDirTargetConfig {
    base_dir: String,
}

#[derive(Debug, Clone)]
struct LocalDirRunReader {
    run_dir: PathBuf,
}

impl LocalDirRunReader {
    fn path_for(&self, artifact: &str) -> PathBuf {
        self.run_dir.join(artifact)
    }
}

impl TargetRunReader for LocalDirRunReader {
    fn target_kind(&self) -> &str {
        TARGET_KIND_LOCAL_DIR
    }

    fn describe_location(&self) -> String {
        self.run_dir.display().to_string()
    }

    fn local_run_dir(&self) -> Option<PathBuf> {
        Some(self.run_dir.clone())
    }

    fn complete_exists(&self) -> DriverFuture<Result<bool, DriverError>> {
        let path = self.path_for(COMPLETE_NAME);
        Box::pin(async move { Ok(tokio::fs::try_exists(path).await.unwrap_or(false)) })
    }

    fn read_bytes(&self, artifact_path: String) -> DriverFuture<Result<Vec<u8>, DriverError>> {
        let path = self.path_for(&artifact_path);
        Box::pin(async move {
            tokio::fs::read(&path)
                .await
                .map_err(|error| DriverError::io(error.to_string()))
        })
    }

    fn head_size(&self, artifact_path: String) -> DriverFuture<Result<Option<u64>, DriverError>> {
        let path = self.path_for(&artifact_path);
        Box::pin(async move {
            match tokio::fs::metadata(path).await {
                Ok(meta) => Ok(Some(meta.len())),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(DriverError::io(error.to_string())),
            }
        })
    }

    fn get_to_file(
        &self,
        artifact_path: String,
        dest: PathBuf,
        expected_size: Option<u64>,
        _retries: usize,
    ) -> DriverFuture<Result<u64, DriverError>> {
        let src = self.path_for(&artifact_path);
        Box::pin(async move {
            if let Some(parent) = dest.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|error| DriverError::io(error.to_string()))?;
            }

            let src_meta = tokio::fs::metadata(&src)
                .await
                .map_err(|error| DriverError::io(error.to_string()))?;
            if let Some(expected) = expected_size
                && src_meta.len() != expected
            {
                return Err(DriverError::io(format!(
                    "artifact size mismatch for {}: expected {}, got {}",
                    artifact_path,
                    expected,
                    src_meta.len()
                )));
            }

            if src == dest {
                return Ok(src_meta.len());
            }

            let copied = tokio::fs::copy(&src, &dest)
                .await
                .map_err(|error| DriverError::io(error.to_string()))?;
            Ok(copied)
        })
    }
}

struct LocalDirTargetDriver {
    id: DriverId,
}

impl LocalDirTargetDriver {
    fn new() -> Self {
        Self {
            id: local_dir_driver_id(),
        }
    }
}

impl TargetDriver for LocalDirTargetDriver {
    fn id(&self) -> &DriverId {
        &self.id
    }

    fn capabilities(&self) -> TargetDriverCapabilities {
        TargetDriverCapabilities {
            supports_archive_rolling_upload: true,
            supports_raw_tree_direct_upload: false,
            supports_cleanup_run: true,
            supports_restore_reader: true,
        }
    }

    fn store_run(
        &self,
        request: StoreRunRequest,
    ) -> DriverFuture<Result<serde_json::Value, DriverError>> {
        Box::pin(async move {
            let cfg: LocalDirTargetConfig = serde_json::from_value(request.target_config.clone())
                .map_err(|error| {
                DriverError::config(format!("invalid local_dir target config: {error}"))
            })?;
            let base_dir = cfg.base_dir.trim();
            if base_dir.is_empty() {
                return Err(DriverError::config("local_dir.base_dir is required"));
            }

            let job_id = request.job_id;
            let run_id = request.run_id;
            let artifacts = request.artifacts;
            let progress = request.on_progress;
            let base_dir = base_dir.to_string();

            let run_dir = tokio::task::spawn_blocking(move || {
                if let Some(cb) = progress.as_ref() {
                    let adapter = |p: bastion_targets::StoreRunProgress| {
                        cb(StoreRunProgress {
                            bytes_done: p.bytes_done,
                            bytes_total: p.bytes_total,
                        });
                    };
                    bastion_targets::local_dir::store_run(
                        Path::new(&base_dir),
                        &job_id,
                        &run_id,
                        &artifacts,
                        Some(&adapter),
                    )
                } else {
                    bastion_targets::local_dir::store_run(
                        Path::new(&base_dir),
                        &job_id,
                        &run_id,
                        &artifacts,
                        None,
                    )
                }
            })
            .await
            .map_err(|error| DriverError::unknown(format!("local_dir store join error: {error}")))?
            .map_err(|error| DriverError::io(error.to_string()))?;

            Ok(serde_json::json!({
                "type": TARGET_KIND_LOCAL_DIR,
                "run_dir": run_dir.to_string_lossy().to_string()
            }))
        })
    }

    fn open_reader(
        &self,
        request: OpenReaderRequest,
    ) -> Result<Arc<dyn TargetRunReader>, DriverError> {
        let cfg: LocalDirTargetConfig =
            serde_json::from_value(request.target_config).map_err(|error| {
                DriverError::config(format!("invalid local_dir target config: {error}"))
            })?;

        let base_dir = cfg.base_dir.trim();
        if base_dir.is_empty() {
            return Err(DriverError::config("local_dir.base_dir is required"));
        }

        let run_dir = Path::new(base_dir)
            .join(request.job_id)
            .join(request.run_id);
        Ok(Arc::new(LocalDirRunReader { run_dir }))
    }

    fn cleanup_run(
        &self,
        request: bastion_driver_api::CleanupRunRequest,
    ) -> DriverFuture<Result<bastion_driver_api::CleanupRunStatus, DriverError>> {
        Box::pin(async move {
            let cfg: LocalDirTargetConfig = serde_json::from_value(request.target_snapshot)
                .map_err(|error| {
                    DriverError::config(format!(
                        "invalid local_dir cleanup snapshot config: {error}"
                    ))
                })?;

            let run_dir = Path::new(cfg.base_dir.trim())
                .join(request.job_id)
                .join(request.run_id);
            if !run_dir.exists() {
                return Ok(bastion_driver_api::CleanupRunStatus::SkipNotFound);
            }
            if run_dir.join(COMPLETE_NAME).exists() {
                return Ok(bastion_driver_api::CleanupRunStatus::SkipComplete);
            }

            let mut looks_like_bastion = false;
            if run_dir.join(MANIFEST_NAME).exists() || run_dir.join(ENTRIES_INDEX_NAME).exists() {
                looks_like_bastion = true;
            } else if let Ok(entries) = std::fs::read_dir(&run_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if name.starts_with("payload.part") || name.ends_with(".partial") {
                        looks_like_bastion = true;
                        break;
                    }
                }
            }
            if !looks_like_bastion {
                return Ok(bastion_driver_api::CleanupRunStatus::SkipNotFound);
            }

            std::fs::remove_dir_all(&run_dir)
                .map_err(|error| DriverError::io(error.to_string()))?;
            Ok(bastion_driver_api::CleanupRunStatus::Deleted)
        })
    }

    fn snapshot_redacted(
        &self,
        target_config: &serde_json::Value,
    ) -> Result<serde_json::Value, DriverError> {
        let cfg: LocalDirTargetConfig =
            serde_json::from_value(target_config.clone()).map_err(|error| {
                DriverError::config(format!("invalid local_dir target config: {error}"))
            })?;
        Ok(serde_json::json!({
            "type": TARGET_KIND_LOCAL_DIR,
            "base_dir": cfg.base_dir,
        }))
    }
}

#[derive(Debug, Deserialize)]
struct WebdavTargetStoreConfig {
    base_url: String,
    username: String,
    password: String,
    #[serde(default, rename = "secret_name")]
    _secret_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WebdavTargetSnapshotConfig {
    base_url: String,
    #[serde(default)]
    secret_name: Option<String>,
}

#[derive(Debug, Clone)]
struct WebdavRunReader {
    client: bastion_targets::WebdavClient,
    run_url: Url,
}

impl WebdavRunReader {
    fn artifact_url(&self, artifact_path: &str) -> Result<Url, DriverError> {
        self.run_url
            .join(artifact_path)
            .map_err(|error| DriverError::config(error.to_string()))
    }
}

impl TargetRunReader for WebdavRunReader {
    fn target_kind(&self) -> &str {
        TARGET_KIND_WEBDAV
    }

    fn describe_location(&self) -> String {
        redact_run_url(&self.run_url)
    }

    fn complete_exists(&self) -> DriverFuture<Result<bool, DriverError>> {
        let client = self.client.clone();
        let url = self.artifact_url(COMPLETE_NAME);
        Box::pin(async move {
            let url = url?;
            let size = client
                .head_size(&url)
                .await
                .map_err(map_webdav_anyhow_to_driver_error)?;
            Ok(size.is_some())
        })
    }

    fn read_bytes(&self, artifact_path: String) -> DriverFuture<Result<Vec<u8>, DriverError>> {
        let client = self.client.clone();
        let url = self.artifact_url(&artifact_path);
        Box::pin(async move {
            let url = url?;
            client
                .get_bytes(&url)
                .await
                .map_err(map_webdav_anyhow_to_driver_error)
        })
    }

    fn head_size(&self, artifact_path: String) -> DriverFuture<Result<Option<u64>, DriverError>> {
        let client = self.client.clone();
        let url = self.artifact_url(&artifact_path);
        Box::pin(async move {
            let url = url?;
            client
                .head_size(&url)
                .await
                .map_err(map_webdav_anyhow_to_driver_error)
        })
    }

    fn get_to_file(
        &self,
        artifact_path: String,
        dest: PathBuf,
        expected_size: Option<u64>,
        retries: usize,
    ) -> DriverFuture<Result<u64, DriverError>> {
        let client = self.client.clone();
        let url = self.artifact_url(&artifact_path);
        Box::pin(async move {
            let url = url?;
            if let Some(parent) = dest.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|error| DriverError::io(error.to_string()))?;
            }
            let retries = u32::try_from(retries).unwrap_or(u32::MAX);
            client
                .get_to_file(&url, &dest, expected_size, retries)
                .await
                .map_err(map_webdav_anyhow_to_driver_error)
        })
    }
}

struct WebdavTargetDriver {
    id: DriverId,
}

impl WebdavTargetDriver {
    fn new() -> Self {
        Self {
            id: webdav_driver_id(),
        }
    }
}

impl TargetDriver for WebdavTargetDriver {
    fn id(&self) -> &DriverId {
        &self.id
    }

    fn capabilities(&self) -> TargetDriverCapabilities {
        TargetDriverCapabilities {
            supports_archive_rolling_upload: true,
            supports_raw_tree_direct_upload: true,
            supports_cleanup_run: true,
            supports_restore_reader: true,
        }
    }

    fn store_run(
        &self,
        request: StoreRunRequest,
    ) -> DriverFuture<Result<serde_json::Value, DriverError>> {
        Box::pin(async move {
            let cfg: WebdavTargetStoreConfig =
                serde_json::from_value(request.target_config.clone()).map_err(|error| {
                    DriverError::config(format!("invalid webdav target config: {error}"))
                })?;
            if cfg.base_url.trim().is_empty() {
                return Err(DriverError::config("webdav.base_url is required"));
            }
            if cfg.username.trim().is_empty() {
                return Err(DriverError::auth("webdav.username is required"));
            }
            if cfg.password.trim().is_empty() {
                return Err(DriverError::auth("webdav.password is required"));
            }

            let limits = request.limits.map(to_webdav_limits);
            let progress = request.on_progress.map(|cb| {
                Arc::new(move |p: bastion_targets::StoreRunProgress| {
                    cb(StoreRunProgress {
                        bytes_done: p.bytes_done,
                        bytes_total: p.bytes_total,
                    })
                }) as Arc<dyn Fn(bastion_targets::StoreRunProgress) + Send + Sync>
            });
            let creds = bastion_targets::WebdavCredentials {
                username: cfg.username,
                password: cfg.password,
            };

            let run_url = bastion_targets::webdav::store_run(
                &cfg.base_url,
                creds,
                &request.job_id,
                &request.run_id,
                &request.artifacts,
                limits,
                progress,
            )
            .await
            .map_err(map_webdav_anyhow_to_driver_error)?;

            Ok(serde_json::json!({
                "type": TARGET_KIND_WEBDAV,
                "run_url": run_url.as_str(),
            }))
        })
    }

    fn open_reader(
        &self,
        request: OpenReaderRequest,
    ) -> Result<Arc<dyn TargetRunReader>, DriverError> {
        let cfg: WebdavTargetStoreConfig =
            serde_json::from_value(request.target_config).map_err(|error| {
                DriverError::config(format!("invalid webdav target config: {error}"))
            })?;

        let base_url = cfg.base_url.trim();
        if base_url.is_empty() {
            return Err(DriverError::config("webdav.base_url is required"));
        }
        let username = cfg.username.trim();
        if username.is_empty() {
            return Err(DriverError::auth("webdav.username is required"));
        }
        let password = cfg.password.trim();
        if password.is_empty() {
            return Err(DriverError::auth("webdav.password is required"));
        }

        let mut parsed_base = Url::parse(base_url)
            .map_err(|error| DriverError::config(format!("invalid webdav.base_url: {error}")))?;
        if !parsed_base.path().ends_with('/') {
            parsed_base.set_path(&format!("{}/", parsed_base.path()));
        }

        let client = bastion_targets::WebdavClient::new(
            parsed_base.clone(),
            bastion_targets::WebdavCredentials {
                username: username.to_string(),
                password: password.to_string(),
            },
        )
        .map_err(map_webdav_anyhow_to_driver_error)?;

        let job_url = parsed_base
            .join(&format!("{}/", request.job_id))
            .map_err(|error| DriverError::config(error.to_string()))?;
        let run_url = job_url
            .join(&format!("{}/", request.run_id))
            .map_err(|error| DriverError::config(error.to_string()))?;

        Ok(Arc::new(WebdavRunReader { client, run_url }))
    }

    fn cleanup_run(
        &self,
        request: bastion_driver_api::CleanupRunRequest,
    ) -> DriverFuture<Result<bastion_driver_api::CleanupRunStatus, DriverError>> {
        Box::pin(async move {
            let cfg: WebdavTargetStoreConfig = serde_json::from_value(request.target_snapshot)
                .map_err(|error| {
                    DriverError::config(format!("invalid webdav cleanup snapshot config: {error}"))
                })?;
            if cfg.base_url.trim().is_empty() {
                return Err(DriverError::config("webdav.base_url is required"));
            }
            if cfg.username.trim().is_empty() {
                return Err(DriverError::auth("webdav.username is required"));
            }
            if cfg.password.trim().is_empty() {
                return Err(DriverError::auth("webdav.password is required"));
            }

            let mut base_url = Url::parse(&cfg.base_url).map_err(|error| {
                DriverError::config(format!("invalid webdav.base_url: {error}"))
            })?;
            if !base_url.path().ends_with('/') {
                base_url.set_path(&format!("{}/", base_url.path()));
            }

            let client = bastion_targets::WebdavClient::new(
                base_url.clone(),
                bastion_targets::WebdavCredentials {
                    username: cfg.username,
                    password: cfg.password,
                },
            )
            .map_err(map_webdav_anyhow_to_driver_error)?;

            let job_url = base_url
                .join(&format!("{}/", request.job_id))
                .map_err(|error| DriverError::config(error.to_string()))?;
            let run_url = job_url
                .join(&format!("{}/", request.run_id))
                .map_err(|error| DriverError::config(error.to_string()))?;
            let complete_url = run_url
                .join(COMPLETE_NAME)
                .map_err(|error| DriverError::config(error.to_string()))?;

            if client
                .head_size(&complete_url)
                .await
                .map_err(map_webdav_anyhow_to_driver_error)?
                .is_some()
            {
                return Ok(bastion_driver_api::CleanupRunStatus::SkipComplete);
            }

            match client
                .delete(&run_url)
                .await
                .map_err(map_webdav_anyhow_to_driver_error)?
            {
                true => Ok(bastion_driver_api::CleanupRunStatus::Deleted),
                false => Ok(bastion_driver_api::CleanupRunStatus::SkipNotFound),
            }
        })
    }

    fn snapshot_redacted(
        &self,
        target_config: &serde_json::Value,
    ) -> Result<serde_json::Value, DriverError> {
        let cfg: WebdavTargetSnapshotConfig = serde_json::from_value(target_config.clone())
            .map_err(|error| {
                DriverError::config(format!("invalid webdav target config: {error}"))
            })?;
        let mut out = serde_json::json!({
            "type": TARGET_KIND_WEBDAV,
            "base_url": redact_base_url(&cfg.base_url),
        });
        if let Some(secret_name) = cfg.secret_name
            && let Some(obj) = out.as_object_mut()
        {
            obj.insert(
                "secret_name".to_string(),
                serde_json::Value::String(secret_name),
            );
        }
        Ok(out)
    }
}

fn map_webdav_anyhow_to_driver_error<E>(error: E) -> DriverError
where
    E: std::fmt::Display,
{
    fn classify_webdav_message_kind(message: &str) -> DriverErrorKind {
        let text = message.to_lowercase();

        if text.contains("kind=auth")
            || text.contains("kind=permission")
            || text.contains("http 401")
            || text.contains("http 403")
            || text.contains("[http_status=401]")
            || text.contains("[http_status=403]")
            || text.contains("unauthorized")
            || text.contains("forbidden")
        {
            return DriverErrorKind::Auth;
        }

        if text.contains("kind=config")
            || text.contains("kind=payload_too_large")
            || text.contains("kind=storage_full")
            || text.contains("http 400")
            || text.contains("http 404")
            || text.contains("http 409")
            || text.contains("http 412")
            || text.contains("http 422")
            || text.contains("[http_status=400]")
            || text.contains("[http_status=404]")
            || text.contains("[http_status=409]")
            || text.contains("[http_status=412]")
            || text.contains("[http_status=422]")
            || text.contains("invalid webdav")
            || text.contains("missing webdav secret")
            || text.contains("invalid target snapshot")
            || text.contains("webdav.base_url")
            || text.contains("payload too large")
            || text.contains("insufficient storage")
            || text.contains("no space left")
            || text.contains("quota exceeded")
            || text.contains("not found")
            || text.contains("no such file")
        {
            return DriverErrorKind::Config;
        }

        if text.contains("kind=rate_limited")
            || text.contains("kind=timeout")
            || text.contains("kind=upstream_unavailable")
            || text.contains("kind=network")
            || text.contains("http 408")
            || text.contains("http 429")
            || text.contains("http 500")
            || text.contains("http 502")
            || text.contains("http 503")
            || text.contains("http 504")
            || text.contains("[http_status=408]")
            || text.contains("[http_status=429]")
            || text.contains("[http_status=500]")
            || text.contains("[http_status=502]")
            || text.contains("[http_status=503]")
            || text.contains("[http_status=504]")
            || text.contains("status code 408")
            || text.contains("status code 429")
            || text.contains("status code 500")
            || text.contains("status code 502")
            || text.contains("status code 503")
            || text.contains("status code 504")
            || text.contains("error sending request")
            || text.contains("timed out")
            || text.contains("timeout")
            || text.contains("connection refused")
            || text.contains("connection reset")
            || text.contains("connection aborted")
            || text.contains("broken pipe")
            || text.contains("dns")
            || text.contains("failed to lookup")
            || text.contains("network")
            || text.contains("temporary failure in name resolution")
            || text.contains("name or service not known")
        {
            return DriverErrorKind::Network;
        }

        if text.contains("http ") || text.contains("[http_status=") {
            return DriverErrorKind::Config;
        }

        DriverErrorKind::Unknown
    }

    let message = error.to_string();
    DriverError::new(classify_webdav_message_kind(&message), message)
}

fn to_webdav_limits(limits: TargetRequestLimits) -> bastion_targets::WebdavRequestLimits {
    bastion_targets::WebdavRequestLimits {
        concurrency: limits.concurrency.unwrap_or(4),
        put_qps: limits.put_qps,
        head_qps: limits.head_qps,
        mkcol_qps: limits.mkcol_qps,
        burst: limits.burst,
        request_timeout_secs: limits.request_timeout_secs,
        connect_timeout_secs: limits.connect_timeout_secs,
        max_put_attempts: limits.max_put_attempts,
    }
}

fn redact_run_url(run_url: &Url) -> String {
    let mut out = run_url.clone();
    let _ = out.set_username("");
    let _ = out.set_password(None);
    out.set_query(None);
    out.set_fragment(None);
    out.to_string()
}

fn redact_base_url(base_url: &str) -> String {
    let Ok(mut url) = Url::parse(base_url) else {
        return base_url.to_string();
    };

    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_query(None);
    url.set_fragment(None);

    if !url.path().ends_with('/') {
        url.set_path(&format!("{}/", url.path()));
    }

    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_registry_contains_builtin_webdav_and_local_dir() {
        let registry = target_registry();
        let _ = registry
            .resolve_target_driver(&local_dir_driver_id())
            .expect("local_dir driver");
        let _ = registry
            .resolve_target_driver(&webdav_driver_id())
            .expect("webdav driver");
    }

    #[tokio::test]
    async fn driver_contract_local_dir_cleanup_run_is_idempotent() {
        let driver = LocalDirTargetDriver::new();
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let base_dir = std::env::temp_dir().join(format!("bastion-local-cleanup-{unique}"));
        let job_id = "job_contract";
        let run_id = "run_contract";
        let run_dir = base_dir.join(job_id).join(run_id);
        std::fs::create_dir_all(&run_dir).expect("create run dir");
        std::fs::write(run_dir.join(MANIFEST_NAME), b"{}").expect("write manifest");

        let request = bastion_driver_api::CleanupRunRequest {
            job_id: job_id.to_string(),
            run_id: run_id.to_string(),
            target_snapshot: serde_json::json!({
                "base_dir": base_dir.to_string_lossy().to_string(),
            }),
        };

        let first = driver
            .cleanup_run(request.clone())
            .await
            .expect("cleanup first");
        assert_eq!(first, bastion_driver_api::CleanupRunStatus::Deleted);

        let second = driver.cleanup_run(request).await.expect("cleanup second");
        assert_eq!(second, bastion_driver_api::CleanupRunStatus::SkipNotFound);

        let _ = std::fs::remove_dir_all(&base_dir);
    }

    #[tokio::test]
    async fn driver_contract_local_dir_reader_supports_complete_and_copy() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let base_dir = std::env::temp_dir().join(format!("bastion-local-reader-{unique}"));
        let run_dir = base_dir.join("job1").join("run1");
        std::fs::create_dir_all(&run_dir).expect("create run dir");
        std::fs::write(run_dir.join(COMPLETE_NAME), b"{}").expect("write complete");
        std::fs::write(run_dir.join(MANIFEST_NAME), br#"{"v":1}"#).expect("write manifest");

        let reader = target_registry()
            .open_reader(
                &local_dir_driver_id(),
                bastion_driver_api::OpenReaderRequest {
                    job_id: "job1".to_string(),
                    run_id: "run1".to_string(),
                    target_config: serde_json::json!({ "base_dir": base_dir.to_string_lossy().to_string() }),
                },
            )
            .expect("open reader");

        assert_eq!(reader.target_kind(), TARGET_KIND_LOCAL_DIR);
        assert!(reader.complete_exists().await.expect("complete exists"));

        let manifest = reader
            .read_bytes(MANIFEST_NAME.to_string())
            .await
            .expect("read manifest");
        assert_eq!(manifest, br#"{"v":1}"#.to_vec());

        let staging = base_dir.join("staging");
        let copied = reader
            .get_to_file(
                MANIFEST_NAME.to_string(),
                staging.join(MANIFEST_NAME),
                Some(manifest.len() as u64),
                3,
            )
            .await
            .expect("copy manifest");
        assert_eq!(copied, manifest.len() as u64);

        let _ = std::fs::remove_dir_all(&base_dir);
    }
    #[test]
    fn webdav_snapshot_redacts_credentials_query_and_fragment() {
        let driver = WebdavTargetDriver::new();
        let out = driver
            .snapshot_redacted(&serde_json::json!({
                "base_url": "https://user:pass@example.com/base?q=1#frag",
                "username": "u",
                "password": "p",
                "secret_name": "main"
            }))
            .expect("snapshot");

        assert_eq!(out["type"], TARGET_KIND_WEBDAV);
        assert_eq!(out["secret_name"], "main");

        let base_url = out["base_url"].as_str().expect("base_url string");
        let parsed = Url::parse(base_url).expect("url parse");
        assert_eq!(parsed.username(), "");
        assert!(parsed.password().is_none());
        assert!(parsed.query().is_none());
        assert!(parsed.fragment().is_none());
        assert!(parsed.path().ends_with('/'));
    }

    #[test]
    fn map_webdav_anyhow_to_driver_error_maps_http_status_texts() {
        #[derive(Debug)]
        struct MsgError(&'static str);

        impl std::fmt::Display for MsgError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::error::Error for MsgError {}

        let auth = map_webdav_anyhow_to_driver_error(MsgError(
            "webdav request failed: HTTP 401: unauthorized",
        ));
        assert_eq!(auth.kind, DriverErrorKind::Auth);

        let config =
            map_webdav_anyhow_to_driver_error(MsgError("webdav request failed: HTTP 404: missing"));
        assert_eq!(config.kind, DriverErrorKind::Config);

        let network =
            map_webdav_anyhow_to_driver_error(MsgError("webdav request failed: HTTP 503: busy"));
        assert_eq!(network.kind, DriverErrorKind::Network);
    }

    #[test]
    fn map_webdav_anyhow_to_driver_error_uses_network_message_fallback() {
        #[derive(Debug)]
        struct MsgError(&'static str);

        impl std::fmt::Display for MsgError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::error::Error for MsgError {}

        let error = MsgError("error sending request: connection refused");
        assert_eq!(
            map_webdav_anyhow_to_driver_error(error).kind,
            DriverErrorKind::Network
        );
    }

    #[test]
    fn map_webdav_anyhow_to_driver_error_maps_not_found_io_to_config() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "No such file");
        let mapped = map_webdav_anyhow_to_driver_error(io_error);
        assert_eq!(mapped.kind, DriverErrorKind::Config);
    }
}
