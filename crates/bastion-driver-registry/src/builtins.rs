use std::path::Path;
use std::sync::{Arc, OnceLock};

use serde::Deserialize;
use url::Url;

use bastion_driver_api::{
    DriverError, DriverFuture, DriverId, StoreRunProgress, StoreRunRequest, TargetDriver,
    TargetDriverCapabilities, TargetRequestLimits,
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
            supports_cleanup_run: false,
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
struct WebdavTargetConfig {
    base_url: String,
    username: String,
    password: String,
    #[serde(default)]
    secret_name: Option<String>,
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
            supports_raw_tree_direct_upload: false,
            supports_cleanup_run: false,
            supports_restore_reader: true,
        }
    }

    fn store_run(
        &self,
        request: StoreRunRequest,
    ) -> DriverFuture<Result<serde_json::Value, DriverError>> {
        Box::pin(async move {
            let cfg: WebdavTargetConfig = serde_json::from_value(request.target_config.clone())
                .map_err(|error| {
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
            .map_err(|error| DriverError::network(error.to_string()))?;

            Ok(serde_json::json!({
                "type": TARGET_KIND_WEBDAV,
                "run_url": run_url.as_str(),
            }))
        })
    }

    fn snapshot_redacted(
        &self,
        target_config: &serde_json::Value,
    ) -> Result<serde_json::Value, DriverError> {
        let cfg: WebdavTargetConfig =
            serde_json::from_value(target_config.clone()).map_err(|error| {
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

fn to_webdav_limits(limits: TargetRequestLimits) -> bastion_targets::WebdavRequestLimits {
    bastion_targets::WebdavRequestLimits {
        concurrency: limits.concurrency.unwrap_or(4),
        put_qps: limits.put_qps,
        head_qps: limits.head_qps,
        mkcol_qps: limits.mkcol_qps,
        burst: limits.burst,
    }
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
}
