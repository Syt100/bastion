use std::collections::HashMap;
use std::sync::Arc;

use bastion_driver_api::{
    CleanupRunRequest, CleanupRunStatus, DriverError, DriverErrorKind, DriverId, OpenReaderRequest,
    SourceDriver, SourceDriverCapabilities, StoreRunRequest, TargetDriver,
    TargetDriverCapabilities, TargetRunReader,
};

pub mod builtins;
pub mod target_runtime;

#[derive(Default)]
pub struct DriverRegistry {
    source_drivers: HashMap<String, Arc<dyn SourceDriver>>,
    target_drivers: HashMap<String, Arc<dyn TargetDriver>>,
}

#[derive(Clone)]
pub struct TargetRunWriter {
    driver: Arc<dyn TargetDriver>,
    request: StoreRunRequest,
    summary: Option<serde_json::Value>,
    uploaded: bool,
    aborted: bool,
}

impl std::fmt::Debug for TargetRunWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TargetRunWriter")
            .field("request", &self.request)
            .field("uploaded", &self.uploaded)
            .field("aborted", &self.aborted)
            .field("has_summary", &self.summary.is_some())
            .finish()
    }
}

impl TargetRunWriter {
    pub async fn upload(&mut self) -> Result<(), DriverError> {
        if self.aborted {
            return Err(DriverError::config("writer is aborted"));
        }
        if self.uploaded {
            return Err(DriverError::config("writer upload already completed"));
        }

        let summary = self.driver.store_run(self.request.clone()).await?;
        self.summary = Some(summary);
        self.uploaded = true;
        Ok(())
    }

    pub async fn finalize(mut self) -> Result<serde_json::Value, DriverError> {
        if self.aborted {
            return Err(DriverError::config("writer is aborted"));
        }
        self.summary
            .take()
            .ok_or_else(|| DriverError::config("writer finalize requires upload first"))
    }

    pub async fn abort(&mut self) -> Result<(), DriverError> {
        if self.aborted {
            return Ok(());
        }

        let snapshot = self.driver.snapshot_redacted(&self.request.target_config)?;
        let cleanup = self
            .driver
            .cleanup_run(CleanupRunRequest {
                job_id: self.request.job_id.clone(),
                run_id: self.request.run_id.clone(),
                target_snapshot: snapshot,
            })
            .await;

        if let Err(error) = cleanup
            && error.kind != DriverErrorKind::Unsupported
        {
            return Err(error);
        }

        self.summary = None;
        self.aborted = true;
        Ok(())
    }
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

    pub fn open_writer(
        &self,
        id: &DriverId,
        request: StoreRunRequest,
    ) -> Result<TargetRunWriter, DriverError> {
        let driver = self.resolve_target_driver(id)?;
        Ok(TargetRunWriter {
            driver,
            request,
            summary: None,
            uploaded: false,
            aborted: false,
        })
    }

    pub fn open_reader(
        &self,
        id: &DriverId,
        request: OpenReaderRequest,
    ) -> Result<Arc<dyn TargetRunReader>, DriverError> {
        let driver = self.resolve_target_driver(id)?;
        driver.open_reader(request)
    }

    pub async fn store_run(
        &self,
        id: &DriverId,
        request: StoreRunRequest,
    ) -> Result<serde_json::Value, DriverError> {
        let driver = self.resolve_target_driver(id)?;
        driver.store_run(request).await
    }

    pub async fn cleanup_run(
        &self,
        id: &DriverId,
        request: CleanupRunRequest,
    ) -> Result<CleanupRunStatus, DriverError> {
        let driver = self.resolve_target_driver(id)?;
        driver.cleanup_run(request).await
    }

    pub fn snapshot_redacted(
        &self,
        id: &DriverId,
        target_config: &serde_json::Value,
    ) -> Result<serde_json::Value, DriverError> {
        let driver = self.resolve_target_driver(id)?;
        driver.snapshot_redacted(target_config)
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
        store_calls: Arc<AtomicUsize>,
        cleanup_calls: Arc<AtomicUsize>,
    }

    impl TargetDriver for TestTargetDriver {
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
            _request: StoreRunRequest,
        ) -> DriverFuture<Result<serde_json::Value, DriverError>> {
            let calls = self.store_calls.clone();
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(serde_json::json!({"ok": true}))
            })
        }

        fn cleanup_run(
            &self,
            _request: bastion_driver_api::CleanupRunRequest,
        ) -> DriverFuture<Result<CleanupRunStatus, DriverError>> {
            let calls = self.cleanup_calls.clone();
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(CleanupRunStatus::SkipNotFound)
            })
        }

        fn snapshot_redacted(
            &self,
            _target_config: &serde_json::Value,
        ) -> Result<serde_json::Value, DriverError> {
            Ok(serde_json::json!({"type": "test"}))
        }
    }

    fn test_artifacts() -> bastion_core::backup_format::LocalRunArtifacts {
        bastion_core::backup_format::LocalRunArtifacts {
            run_dir: std::path::PathBuf::from("/tmp/run"),
            parts: vec![],
            entries_index_path: std::path::PathBuf::from("/tmp/entries"),
            entries_count: 0,
            manifest_path: std::path::PathBuf::from("/tmp/manifest"),
            complete_path: std::path::PathBuf::from("/tmp/complete"),
        }
    }

    fn setup_registry() -> (DriverRegistry, Arc<AtomicUsize>, Arc<AtomicUsize>, DriverId) {
        let mut registry = DriverRegistry::new();
        let store_calls = Arc::new(AtomicUsize::new(0));
        let cleanup_calls = Arc::new(AtomicUsize::new(0));
        let id = DriverId::new("test_target", 1).expect("id");
        let driver = Arc::new(TestTargetDriver {
            id: id.clone(),
            store_calls: store_calls.clone(),
            cleanup_calls: cleanup_calls.clone(),
        });
        registry
            .register_target_driver(driver)
            .expect("register driver");
        (registry, store_calls, cleanup_calls, id)
    }

    #[tokio::test]
    async fn resolves_registered_target_driver_and_delegates_store() {
        let (registry, store_calls, _cleanup_calls, id) = setup_registry();

        let summary = registry
            .store_run(
                &id,
                StoreRunRequest {
                    job_id: "j".to_string(),
                    run_id: "r".to_string(),
                    target_config: serde_json::json!({}),
                    artifacts: test_artifacts(),
                    limits: Some(TargetRequestLimits::default()),
                    on_progress: Some(Arc::new(|_p: StoreRunProgress| {})),
                },
            )
            .await
            .expect("store run");

        assert_eq!(summary["ok"], serde_json::json!(true));
        assert_eq!(store_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn open_writer_upload_finalize_roundtrip() {
        let (registry, store_calls, _cleanup_calls, id) = setup_registry();

        let mut writer = registry
            .open_writer(
                &id,
                StoreRunRequest {
                    job_id: "j".to_string(),
                    run_id: "r".to_string(),
                    target_config: serde_json::json!({}),
                    artifacts: test_artifacts(),
                    limits: Some(TargetRequestLimits::default()),
                    on_progress: None,
                },
            )
            .expect("open writer");

        writer.upload().await.expect("upload");
        let summary = writer.finalize().await.expect("finalize");
        assert_eq!(summary["ok"], serde_json::json!(true));
        assert_eq!(store_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn driver_contract_open_writer_abort_is_idempotent() {
        let (registry, _store_calls, cleanup_calls, id) = setup_registry();

        let mut writer = registry
            .open_writer(
                &id,
                StoreRunRequest {
                    job_id: "j".to_string(),
                    run_id: "r".to_string(),
                    target_config: serde_json::json!({}),
                    artifacts: test_artifacts(),
                    limits: None,
                    on_progress: None,
                },
            )
            .expect("open writer");

        writer.abort().await.expect("abort once");
        writer.abort().await.expect("abort twice");
        assert_eq!(cleanup_calls.load(Ordering::SeqCst), 1);

        let err = writer
            .upload()
            .await
            .expect_err("aborted writer cannot upload");
        assert_eq!(err.kind, bastion_driver_api::DriverErrorKind::Config);
    }

    #[test]
    fn open_reader_local_dir_builds_run_path() {
        let registry = builtins::target_registry();
        let id = builtins::local_dir_driver_id();

        let reader = registry
            .open_reader(
                &id,
                OpenReaderRequest {
                    job_id: "job1".to_string(),
                    run_id: "run1".to_string(),
                    target_config: serde_json::json!({ "base_dir": "/tmp/base" }),
                },
            )
            .expect("open reader");

        assert_eq!(reader.target_kind(), builtins::TARGET_KIND_LOCAL_DIR);
        assert_eq!(
            reader.local_run_dir(),
            Some(std::path::PathBuf::from("/tmp/base/job1/run1"))
        );
    }

    #[test]
    fn duplicate_target_registration_is_rejected() {
        let mut registry = DriverRegistry::new();
        let driver = Arc::new(TestTargetDriver {
            id: DriverId::new("dup", 1).expect("id"),
            store_calls: Arc::new(AtomicUsize::new(0)),
            cleanup_calls: Arc::new(AtomicUsize::new(0)),
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
