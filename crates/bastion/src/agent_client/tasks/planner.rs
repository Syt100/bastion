use bastion_core::agent_protocol::{PipelineResolvedV1, TargetResolvedV1};
use bastion_core::execution_planner::{
    DirectUploadPreferenceV1, ExecutionPlanV1, ExecutionPlannerInputV1, PlannerDriverRefV1,
    PlannerSourceCapabilitiesV1, PlannerTargetCapabilitiesV1, plan_execution,
};
use bastion_core::job_spec;

#[derive(Debug, Clone)]
pub(super) struct PlannedExecution {
    pub(super) source_driver: PlannerDriverRefV1,
    pub(super) target_driver: PlannerDriverRefV1,
    pub(super) plan: ExecutionPlanV1,
}

pub(super) fn plan_filesystem_execution(
    pipeline: &PipelineResolvedV1,
    source: &job_spec::FilesystemSource,
    target: &TargetResolvedV1,
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
    pipeline: &PipelineResolvedV1,
    target: &TargetResolvedV1,
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
    pipeline: &PipelineResolvedV1,
    source: &job_spec::VaultwardenSource,
    target: &TargetResolvedV1,
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
    pipeline: &PipelineResolvedV1,
    consistency_policy: Option<job_spec::ConsistencyPolicyV1>,
    upload_on_consistency_failure: Option<bool>,
    direct_upload_preference: DirectUploadPreferenceV1,
    target: &TargetResolvedV1,
) -> Result<PlannedExecution, anyhow::Error> {
    let (target_driver, target_capabilities) = match target {
        TargetResolvedV1::Webdav { .. } => (
            PlannerDriverRefV1::new("webdav", 1)?,
            PlannerTargetCapabilitiesV1 {
                supports_archive_rolling_upload: true,
                supports_raw_tree_direct_upload: true,
                supports_cleanup_run: true,
                supports_restore_reader: true,
            },
        ),
        TargetResolvedV1::LocalDir { .. } => (
            PlannerDriverRefV1::new("local_dir", 1)?,
            PlannerTargetCapabilitiesV1 {
                supports_archive_rolling_upload: true,
                supports_raw_tree_direct_upload: false,
                supports_cleanup_run: true,
                supports_restore_reader: true,
            },
        ),
    };

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

#[cfg(test)]
mod tests {
    use bastion_core::agent_protocol::{PipelineResolvedV1, TargetResolvedV1};
    use bastion_core::execution_planner::ExecutionPlanModeV1;
    use bastion_core::manifest::ArtifactFormatV1;

    use super::{plan_filesystem_execution, plan_sqlite_execution, plan_vaultwarden_execution};

    #[test]
    fn filesystem_planner_selects_direct_mode_for_webdav_raw_tree() {
        let source = bastion_core::job_spec::FilesystemSource {
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
            consistency_policy: bastion_core::job_spec::ConsistencyPolicyV1::Warn,
            consistency_fail_threshold: Some(0),
            upload_on_consistency_failure: Some(true),
        };
        let target = TargetResolvedV1::Webdav {
            base_url: "https://example.com/base/".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            part_size_bytes: 1024,
        };
        let pipeline = PipelineResolvedV1 {
            format: ArtifactFormatV1::RawTreeV1,
            webdav: bastion_core::job_spec::PipelineWebdavV1 {
                raw_tree_direct: bastion_core::job_spec::WebdavRawTreeDirectSettingsV1 {
                    mode: bastion_core::job_spec::WebdavRawTreeDirectModeV1::Auto,
                    ..Default::default()
                },
            },
            ..Default::default()
        };

        let planned = plan_filesystem_execution(&pipeline, &source, &target).expect("plan");
        assert_eq!(planned.plan.mode, ExecutionPlanModeV1::RawTreeWebdavDirect);
    }

    #[test]
    fn sqlite_planner_uses_rolling_upload_for_archive() {
        let target = TargetResolvedV1::LocalDir {
            base_dir: "/tmp/out".to_string(),
            part_size_bytes: 1024,
        };
        let pipeline = PipelineResolvedV1 {
            format: ArtifactFormatV1::ArchiveV1,
            ..Default::default()
        };

        let planned = plan_sqlite_execution(&pipeline, &target).expect("plan");
        assert_eq!(planned.plan.mode, ExecutionPlanModeV1::RollingUpload);
    }

    #[test]
    fn vaultwarden_planner_blocks_rolling_for_strict_consistency() {
        let source = bastion_core::job_spec::VaultwardenSource {
            data_dir: "/vw".to_string(),
            consistency_policy: bastion_core::job_spec::ConsistencyPolicyV1::Fail,
            consistency_fail_threshold: Some(0),
            upload_on_consistency_failure: Some(false),
        };
        let target = TargetResolvedV1::Webdav {
            base_url: "https://example.com/base/".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            part_size_bytes: 1024,
        };
        let pipeline = PipelineResolvedV1 {
            format: ArtifactFormatV1::ArchiveV1,
            ..Default::default()
        };

        let planned = plan_vaultwarden_execution(&pipeline, &source, &target).expect("plan");
        assert_eq!(planned.plan.mode, ExecutionPlanModeV1::StagedUpload);
    }
}
