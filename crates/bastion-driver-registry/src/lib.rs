use std::collections::HashMap;
use std::sync::Arc;

use bastion_driver_api::{
    DriverError, DriverId, SourceDriver, SourceDriverCapabilities, StoreRunRequest, TargetDriver,
    TargetDriverCapabilities,
};

pub mod builtins;

#[derive(Default)]
pub struct DriverRegistry {
    source_drivers: HashMap<String, Arc<dyn SourceDriver>>,
    target_drivers: HashMap<String, Arc<dyn TargetDriver>>,
}

impl DriverRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_source_driver(
        &mut self,
        driver: Arc<dyn SourceDriver>,
    ) -> Result<(), DriverError> {
        let key = driver.id().key();
        if self.source_drivers.contains_key(&key) {
            return Err(DriverError::config(format!(
                "source driver already registered: {key}"
            )));
        }
        self.source_drivers.insert(key, driver);
        Ok(())
    }

    pub fn register_target_driver(
        &mut self,
        driver: Arc<dyn TargetDriver>,
    ) -> Result<(), DriverError> {
        let key = driver.id().key();
        if self.target_drivers.contains_key(&key) {
            return Err(DriverError::config(format!(
                "target driver already registered: {key}"
            )));
        }
        self.target_drivers.insert(key, driver);
        Ok(())
    }

    pub fn resolve_source_driver(
        &self,
        id: &DriverId,
    ) -> Result<Arc<dyn SourceDriver>, DriverError> {
        self.source_drivers
            .get(&id.key())
            .cloned()
            .ok_or_else(|| DriverError::unsupported(format!("unsupported source driver: {id}")))
    }

    pub fn resolve_target_driver(
        &self,
        id: &DriverId,
    ) -> Result<Arc<dyn TargetDriver>, DriverError> {
        self.target_drivers
            .get(&id.key())
            .cloned()
            .ok_or_else(|| DriverError::unsupported(format!("unsupported target driver: {id}")))
    }

    pub async fn store_run(
        &self,
        id: &DriverId,
        request: StoreRunRequest,
    ) -> Result<serde_json::Value, DriverError> {
        let driver = self.resolve_target_driver(id)?;
        driver.store_run(request).await
    }

    pub fn target_capabilities(
        &self,
        id: &DriverId,
    ) -> Result<TargetDriverCapabilities, DriverError> {
        let driver = self.resolve_target_driver(id)?;
        Ok(driver.capabilities())
    }

    pub fn source_capabilities(
        &self,
        id: &DriverId,
    ) -> Result<SourceDriverCapabilities, DriverError> {
        let driver = self.resolve_source_driver(id)?;
        Ok(driver.capabilities())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicUsize, Ordering};

    use bastion_driver_api::{
        CleanupRunStatus, DriverFuture, StoreRunProgress, TargetRequestLimits,
    };

    struct TestTargetDriver {
        id: DriverId,
        calls: Arc<AtomicUsize>,
    }

    impl TargetDriver for TestTargetDriver {
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
            _request: StoreRunRequest,
        ) -> DriverFuture<Result<serde_json::Value, DriverError>> {
            let calls = self.calls.clone();
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(serde_json::json!({"ok": true}))
            })
        }

        fn cleanup_run(
            &self,
            _request: bastion_driver_api::CleanupRunRequest,
        ) -> DriverFuture<Result<CleanupRunStatus, DriverError>> {
            Box::pin(async { Ok(CleanupRunStatus::SkipNotFound) })
        }

        fn snapshot_redacted(
            &self,
            _target_config: &serde_json::Value,
        ) -> Result<serde_json::Value, DriverError> {
            Ok(serde_json::json!({"type": "test"}))
        }
    }

    #[tokio::test]
    async fn resolves_registered_target_driver_and_delegates_store() {
        let mut registry = DriverRegistry::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let driver = Arc::new(TestTargetDriver {
            id: DriverId::new("test_target", 1).expect("id"),
            calls: calls.clone(),
        });
        registry
            .register_target_driver(driver)
            .expect("register driver");

        let artifacts = bastion_core::backup_format::LocalRunArtifacts {
            run_dir: std::path::PathBuf::from("/tmp/run"),
            parts: vec![],
            entries_index_path: std::path::PathBuf::from("/tmp/entries"),
            entries_count: 0,
            manifest_path: std::path::PathBuf::from("/tmp/manifest"),
            complete_path: std::path::PathBuf::from("/tmp/complete"),
        };
        let id = DriverId::new("test_target", 1).expect("id");
        let summary = registry
            .store_run(
                &id,
                StoreRunRequest {
                    job_id: "j".to_string(),
                    run_id: "r".to_string(),
                    target_config: serde_json::json!({}),
                    artifacts,
                    limits: Some(TargetRequestLimits::default()),
                    on_progress: Some(Arc::new(|_p: StoreRunProgress| {})),
                },
            )
            .await
            .expect("store run");

        assert_eq!(summary["ok"], serde_json::json!(true));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn duplicate_target_registration_is_rejected() {
        let mut registry = DriverRegistry::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let driver = Arc::new(TestTargetDriver {
            id: DriverId::new("dup", 1).expect("id"),
            calls: calls.clone(),
        });

        registry
            .register_target_driver(driver.clone())
            .expect("first register");
        let err = registry
            .register_target_driver(driver)
            .expect_err("duplicate must fail");
        assert_eq!(err.kind, bastion_driver_api::DriverErrorKind::Config);
    }

    #[test]
    fn missing_driver_returns_unsupported() {
        let registry = DriverRegistry::new();
        let err = registry
            .resolve_target_driver(&DriverId::new("missing", 1).expect("id"))
            .err()
            .expect("missing driver");
        assert_eq!(err.kind, bastion_driver_api::DriverErrorKind::Unsupported);
    }
}
