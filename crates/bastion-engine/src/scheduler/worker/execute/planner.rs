use bastion_core::execution_planner::{
    DirectUploadPreferenceV1, ExecutionPlanV1, ExecutionPlannerInputV1, PlannerDriverRefV1,
    PlannerSourceCapabilitiesV1, PlannerTargetCapabilitiesV1, plan_execution,
};
use bastion_core::job_spec;
use bastion_driver_registry::builtins;
use bastion_driver_registry::target_runtime;

#[derive(Debug, Clone)]
pub(super) struct PlannedExecution {
    pub(super) source_driver: PlannerDriverRefV1,
    pub(super) target_driver: PlannerDriverRefV1,
    pub(super) plan: ExecutionPlanV1,
}

pub(super) fn plan_filesystem_execution(
    pipeline: &job_spec::PipelineV1,
    source: &job_spec::FilesystemSource,
    target: &job_spec::TargetV1,
) -> Result<PlannedExecution, anyhow::Error> {
    let source_driver = PlannerDriverRefV1::new("filesystem", 1)?;
    let source_capabilities = PlannerSourceCapabilitiesV1 {
        supports_snapshots: true,
    };
    plan_for_target(
        source_driver,
        source_capabilities,
        pipeline,
        Some(source.consistency_policy),
        source.upload_on_consistency_failure,
        DirectUploadPreferenceV1::from(pipeline.webdav.raw_tree_direct.mode),
        target,
    )
}

pub(super) fn plan_sqlite_execution(
    pipeline: &job_spec::PipelineV1,
    target: &job_spec::TargetV1,
) -> Result<PlannedExecution, anyhow::Error> {
    let source_driver = PlannerDriverRefV1::new("sqlite", 1)?;
    plan_for_target(
        source_driver,
        PlannerSourceCapabilitiesV1::default(),
        pipeline,
        None,
        None,
        DirectUploadPreferenceV1::Off,
        target,
    )
}

pub(super) fn plan_vaultwarden_execution(
    pipeline: &job_spec::PipelineV1,
    source: &job_spec::VaultwardenSource,
    target: &job_spec::TargetV1,
) -> Result<PlannedExecution, anyhow::Error> {
    let source_driver = PlannerDriverRefV1::new("vaultwarden", 1)?;
    plan_for_target(
        source_driver,
        PlannerSourceCapabilitiesV1::default(),
        pipeline,
        Some(source.consistency_policy),
        source.upload_on_consistency_failure,
        DirectUploadPreferenceV1::Off,
        target,
    )
}

fn plan_for_target(
    source_driver: PlannerDriverRefV1,
    source_capabilities: PlannerSourceCapabilitiesV1,
    pipeline: &job_spec::PipelineV1,
    consistency_policy: Option<job_spec::ConsistencyPolicyV1>,
    upload_on_consistency_failure: Option<bool>,
    direct_upload_preference: DirectUploadPreferenceV1,
    target: &job_spec::TargetV1,
) -> Result<PlannedExecution, anyhow::Error> {
    let target_driver = target_driver_ref(target)?;
    let target_capabilities = resolve_target_capabilities(&target_driver)?;

    let plan = plan_execution(&ExecutionPlannerInputV1 {
        source_driver: source_driver.clone(),
        source_capabilities,
        target_driver: target_driver.clone(),
        target_capabilities,
        artifact_format: pipeline.format.clone(),
        direct_upload_preference,
        consistency_policy,
        upload_on_consistency_failure,
    })?;

    Ok(PlannedExecution {
        source_driver,
        target_driver,
        plan,
    })
}

fn target_driver_ref(target: &job_spec::TargetV1) -> Result<PlannerDriverRefV1, anyhow::Error> {
    let id = target_runtime::driver_id_for_job_target(target);
    Ok(PlannerDriverRefV1::new(id.kind, id.version)?)
}

fn resolve_target_capabilities(
    target_driver: &PlannerDriverRefV1,
) -> Result<PlannerTargetCapabilitiesV1, anyhow::Error> {
    let id = bastion_driver_api::DriverId::new(target_driver.kind.clone(), target_driver.version)?;
    let caps = builtins::target_registry().target_capabilities(&id)?;
    Ok(PlannerTargetCapabilitiesV1 {
        supports_archive_rolling_upload: caps.supports_archive_rolling_upload,
        supports_raw_tree_direct_upload: caps.supports_raw_tree_direct_upload,
        supports_cleanup_run: caps.supports_cleanup_run,
        supports_restore_reader: caps.supports_restore_reader,
    })
}

#[cfg(test)]
mod tests {
    use bastion_core::execution_planner::ExecutionPlanModeV1;
    use bastion_core::job_spec;
    use bastion_core::manifest::ArtifactFormatV1;

    use super::{plan_filesystem_execution, plan_sqlite_execution, plan_vaultwarden_execution};

    #[test]
    fn filesystem_planner_selects_direct_mode_when_requested() {
        let source = job_spec::FilesystemSource {
            pre_scan: true,
            paths: vec!["/tmp".to_string()],
            root: String::new(),
            include: vec![],
            exclude: vec![],
            symlink_policy: Default::default(),
            hardlink_policy: Default::default(),
            error_policy: Default::default(),
            snapshot_mode: Default::default(),
            snapshot_provider: None,
            consistency_policy: job_spec::ConsistencyPolicyV1::Warn,
            consistency_fail_threshold: Some(0),
            upload_on_consistency_failure: Some(true),
        };
        let target = job_spec::TargetV1::Webdav {
            base_url: "https://example.com/base/".to_string(),
            secret_name: "primary".to_string(),
            part_size_bytes: 123,
        };
        let pipeline = job_spec::PipelineV1 {
            format: ArtifactFormatV1::RawTreeV1,
            webdav: job_spec::PipelineWebdavV1 {
                raw_tree_direct: job_spec::WebdavRawTreeDirectSettingsV1 {
                    mode: job_spec::WebdavRawTreeDirectModeV1::Auto,
                    ..Default::default()
                },
            },
            ..Default::default()
        };

        let planned = plan_filesystem_execution(&pipeline, &source, &target).expect("plan");
        assert_eq!(planned.plan.mode, ExecutionPlanModeV1::RawTreeWebdavDirect);
    }

    #[test]
    fn sqlite_planner_prefers_rolling_for_archive_when_supported() {
        let target = job_spec::TargetV1::LocalDir {
            base_dir: "/tmp/out".to_string(),
            part_size_bytes: 123,
        };
        let pipeline = job_spec::PipelineV1 {
            format: ArtifactFormatV1::ArchiveV1,
            ..Default::default()
        };

        let planned = plan_sqlite_execution(&pipeline, &target).expect("plan");
        assert_eq!(planned.plan.mode, ExecutionPlanModeV1::RollingUpload);
    }

    #[test]
    fn vaultwarden_planner_blocks_rolling_when_consistency_policy_requires_it() {
        let source = job_spec::VaultwardenSource {
            data_dir: "/vw".to_string(),
            consistency_policy: job_spec::ConsistencyPolicyV1::Fail,
            consistency_fail_threshold: Some(0),
            upload_on_consistency_failure: Some(false),
        };
        let target = job_spec::TargetV1::Webdav {
            base_url: "https://example.com/base/".to_string(),
            secret_name: "primary".to_string(),
            part_size_bytes: 123,
        };
        let pipeline = job_spec::PipelineV1 {
            format: ArtifactFormatV1::ArchiveV1,
            ..Default::default()
        };

        let planned = plan_vaultwarden_execution(&pipeline, &source, &target).expect("plan");
        assert_eq!(planned.plan.mode, ExecutionPlanModeV1::StagedUpload);
    }
}
