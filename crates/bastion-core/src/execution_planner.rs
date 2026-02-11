use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::job_spec::{ConsistencyPolicyV1, WebdavRawTreeDirectModeV1};
use crate::manifest::ArtifactFormatV1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannerDriverRefV1 {
    pub kind: String,
    pub version: u32,
}

impl PlannerDriverRefV1 {
    pub fn new(kind: impl Into<String>, version: u32) -> Result<Self, ExecutionPlannerErrorV1> {
        let kind = kind.into();
        if kind.trim().is_empty() {
            return Err(ExecutionPlannerErrorV1::invalid_input(
                "driver kind is required",
            ));
        }
        if version == 0 {
            return Err(ExecutionPlannerErrorV1::invalid_input(
                "driver version must be >= 1",
            ));
        }
        Ok(Self { kind, version })
    }

    pub fn label(&self) -> String {
        format!("{}@{}", self.kind, self.version)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PlannerSourceCapabilitiesV1 {
    #[serde(default)]
    pub supports_snapshots: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PlannerTargetCapabilitiesV1 {
    #[serde(default)]
    pub supports_archive_rolling_upload: bool,
    #[serde(default)]
    pub supports_raw_tree_direct_upload: bool,
    #[serde(default)]
    pub supports_cleanup_run: bool,
    #[serde(default)]
    pub supports_restore_reader: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DirectUploadPreferenceV1 {
    #[default]
    Off,
    Auto,
    On,
}

impl From<WebdavRawTreeDirectModeV1> for DirectUploadPreferenceV1 {
    fn from(value: WebdavRawTreeDirectModeV1) -> Self {
        match value {
            WebdavRawTreeDirectModeV1::Off => Self::Off,
            WebdavRawTreeDirectModeV1::Auto => Self::Auto,
            WebdavRawTreeDirectModeV1::On => Self::On,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPlanModeV1 {
    StagedUpload,
    RollingUpload,
    RawTreeWebdavDirect,
}

impl ExecutionPlanModeV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StagedUpload => "staged_upload",
            Self::RollingUpload => "rolling_upload",
            Self::RawTreeWebdavDirect => "raw_tree_webdav_direct",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPlanFallbackReasonV1 {
    ConsistencyFailPolicyBlocksRollingUpload,
    TargetMissingRollingUploadCapability,
    DirectUploadRequiresRawTreeFormat,
    DirectUploadRequiresWebdavTarget,
    TargetMissingRawTreeDirectCapability,
}

impl ExecutionPlanFallbackReasonV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ConsistencyFailPolicyBlocksRollingUpload => {
                "consistency_fail_policy_blocks_rolling_upload"
            }
            Self::TargetMissingRollingUploadCapability => {
                "target_missing_rolling_upload_capability"
            }
            Self::DirectUploadRequiresRawTreeFormat => "direct_upload_requires_raw_tree_format",
            Self::DirectUploadRequiresWebdavTarget => "direct_upload_requires_webdav_target",
            Self::TargetMissingRawTreeDirectCapability => {
                "target_missing_raw_tree_direct_capability"
            }
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::ConsistencyFailPolicyBlocksRollingUpload => {
                "consistency fail policy blocks remote upload before validation"
            }
            Self::TargetMissingRollingUploadCapability => {
                "target driver does not support rolling upload"
            }
            Self::DirectUploadRequiresRawTreeFormat => {
                "direct upload requires raw_tree_v1 artifact format"
            }
            Self::DirectUploadRequiresWebdavTarget => {
                "direct upload requires a webdav target driver"
            }
            Self::TargetMissingRawTreeDirectCapability => {
                "target driver does not support raw-tree direct upload"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlannerInputV1 {
    pub source_driver: PlannerDriverRefV1,
    pub source_capabilities: PlannerSourceCapabilitiesV1,
    pub target_driver: PlannerDriverRefV1,
    pub target_capabilities: PlannerTargetCapabilitiesV1,
    pub artifact_format: ArtifactFormatV1,
    pub direct_upload_preference: DirectUploadPreferenceV1,
    pub consistency_policy: Option<ConsistencyPolicyV1>,
    pub upload_on_consistency_failure: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPlanV1 {
    pub mode: ExecutionPlanModeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<ExecutionPlanFallbackReasonV1>,
    #[serde(default)]
    pub allow_rolling_upload: bool,
    #[serde(default)]
    pub enable_raw_tree_webdav_direct_upload: bool,
    #[serde(default)]
    pub link_stage_data_to_local_target: bool,
    #[serde(default)]
    pub required_target_capabilities: PlannerTargetCapabilitiesV1,
}

impl ExecutionPlanV1 {
    pub fn observability_fields(
        &self,
        source_driver: &PlannerDriverRefV1,
        target_driver: &PlannerDriverRefV1,
    ) -> serde_json::Value {
        serde_json::json!({
            "source_driver": source_driver.label(),
            "target_driver": target_driver.label(),
            "plan_mode": self.mode.as_str(),
            "plan_fallback_reason": self.fallback_reason.map(ExecutionPlanFallbackReasonV1::as_str),
        })
    }

    pub fn summary_payload(
        &self,
        source_driver: &PlannerDriverRefV1,
        target_driver: &PlannerDriverRefV1,
    ) -> serde_json::Value {
        serde_json::json!({
            "source_driver": source_driver.label(),
            "target_driver": target_driver.label(),
            "mode": self.mode,
            "fallback_reason": self.fallback_reason,
            "allow_rolling_upload": self.allow_rolling_upload,
            "enable_raw_tree_webdav_direct_upload": self.enable_raw_tree_webdav_direct_upload,
            "link_stage_data_to_local_target": self.link_stage_data_to_local_target,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlannerErrorV1 {
    code: &'static str,
    message: String,
    pub fallback_reason: Option<ExecutionPlanFallbackReasonV1>,
}

impl ExecutionPlannerErrorV1 {
    fn invalid_input(message: impl Into<String>) -> Self {
        Self {
            code: "planner_invalid_input",
            message: message.into(),
            fallback_reason: None,
        }
    }

    fn unsupported_plan(
        reason: ExecutionPlanFallbackReasonV1,
        source: &PlannerDriverRefV1,
        target: &PlannerDriverRefV1,
    ) -> Self {
        Self {
            code: "planner_unsupported_required_mode",
            message: format!(
                "required direct-upload mode is unsupported for source {} and target {}: {}",
                source.label(),
                target.label(),
                reason.message()
            ),
            fallback_reason: Some(reason),
        }
    }

    pub fn code(&self) -> &'static str {
        self.code
    }
}

impl Display for ExecutionPlannerErrorV1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ExecutionPlannerErrorV1 {}

pub fn plan_execution(
    input: &ExecutionPlannerInputV1,
) -> Result<ExecutionPlanV1, ExecutionPlannerErrorV1> {
    if input.source_driver.kind.trim().is_empty() {
        return Err(ExecutionPlannerErrorV1::invalid_input(
            "source driver kind is required",
        ));
    }
    if input.source_driver.version == 0 {
        return Err(ExecutionPlannerErrorV1::invalid_input(
            "source driver version must be >= 1",
        ));
    }
    if input.target_driver.kind.trim().is_empty() {
        return Err(ExecutionPlannerErrorV1::invalid_input(
            "target driver kind is required",
        ));
    }
    if input.target_driver.version == 0 {
        return Err(ExecutionPlannerErrorV1::invalid_input(
            "target driver version must be >= 1",
        ));
    }

    let policy_allows_remote_upload = !matches!(
        (
            input.consistency_policy,
            input.upload_on_consistency_failure.unwrap_or(false),
        ),
        (Some(ConsistencyPolicyV1::Fail), false)
    );

    let mut fallback_reason = None;

    let direct_requirement_reason =
        if input.direct_upload_preference == DirectUploadPreferenceV1::Off {
            None
        } else {
            direct_requirement_reason(input, policy_allows_remote_upload)
        };

    if let Some(reason) = direct_requirement_reason {
        match input.direct_upload_preference {
            DirectUploadPreferenceV1::On => {
                return Err(ExecutionPlannerErrorV1::unsupported_plan(
                    reason,
                    &input.source_driver,
                    &input.target_driver,
                ));
            }
            DirectUploadPreferenceV1::Auto => {
                fallback_reason = Some(reason);
            }
            DirectUploadPreferenceV1::Off => {}
        }
    }

    let direct_enabled = input.direct_upload_preference != DirectUploadPreferenceV1::Off
        && direct_requirement_reason.is_none();

    if direct_enabled {
        return Ok(ExecutionPlanV1 {
            mode: ExecutionPlanModeV1::RawTreeWebdavDirect,
            fallback_reason,
            allow_rolling_upload: false,
            enable_raw_tree_webdav_direct_upload: true,
            link_stage_data_to_local_target: false,
            required_target_capabilities: PlannerTargetCapabilitiesV1 {
                supports_raw_tree_direct_upload: true,
                ..Default::default()
            },
        });
    }

    let rolling_enabled = policy_allows_remote_upload
        && input.target_capabilities.supports_archive_rolling_upload
        && input.artifact_format == ArtifactFormatV1::ArchiveV1;

    if !rolling_enabled && fallback_reason.is_none() {
        fallback_reason = if !policy_allows_remote_upload {
            Some(ExecutionPlanFallbackReasonV1::ConsistencyFailPolicyBlocksRollingUpload)
        } else if !input.target_capabilities.supports_archive_rolling_upload {
            Some(ExecutionPlanFallbackReasonV1::TargetMissingRollingUploadCapability)
        } else {
            None
        };
    }

    Ok(ExecutionPlanV1 {
        mode: if rolling_enabled {
            ExecutionPlanModeV1::RollingUpload
        } else {
            ExecutionPlanModeV1::StagedUpload
        },
        fallback_reason,
        allow_rolling_upload: rolling_enabled,
        enable_raw_tree_webdav_direct_upload: false,
        link_stage_data_to_local_target: input.artifact_format == ArtifactFormatV1::RawTreeV1
            && input.target_driver.kind == "local_dir",
        required_target_capabilities: PlannerTargetCapabilitiesV1 {
            supports_archive_rolling_upload: rolling_enabled,
            ..Default::default()
        },
    })
}

fn direct_requirement_reason(
    input: &ExecutionPlannerInputV1,
    policy_allows_remote_upload: bool,
) -> Option<ExecutionPlanFallbackReasonV1> {
    if !policy_allows_remote_upload {
        return Some(ExecutionPlanFallbackReasonV1::ConsistencyFailPolicyBlocksRollingUpload);
    }
    if input.artifact_format != ArtifactFormatV1::RawTreeV1 {
        return Some(ExecutionPlanFallbackReasonV1::DirectUploadRequiresRawTreeFormat);
    }
    if input.target_driver.kind != "webdav" {
        return Some(ExecutionPlanFallbackReasonV1::DirectUploadRequiresWebdavTarget);
    }
    if !input.target_capabilities.supports_raw_tree_direct_upload {
        return Some(ExecutionPlanFallbackReasonV1::TargetMissingRawTreeDirectCapability);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filesystem_webdav_input() -> ExecutionPlannerInputV1 {
        ExecutionPlannerInputV1 {
            source_driver: PlannerDriverRefV1::new("filesystem", 1).expect("source"),
            source_capabilities: PlannerSourceCapabilitiesV1 {
                supports_snapshots: true,
            },
            target_driver: PlannerDriverRefV1::new("webdav", 1).expect("target"),
            target_capabilities: PlannerTargetCapabilitiesV1 {
                supports_archive_rolling_upload: true,
                supports_raw_tree_direct_upload: true,
                supports_cleanup_run: true,
                supports_restore_reader: true,
            },
            artifact_format: ArtifactFormatV1::RawTreeV1,
            direct_upload_preference: DirectUploadPreferenceV1::Auto,
            consistency_policy: Some(ConsistencyPolicyV1::Warn),
            upload_on_consistency_failure: Some(true),
        }
    }

    #[test]
    fn planner_selects_raw_tree_direct_when_supported() {
        let input = filesystem_webdav_input();
        let plan = plan_execution(&input).expect("plan");

        assert_eq!(plan.mode, ExecutionPlanModeV1::RawTreeWebdavDirect);
        assert!(plan.enable_raw_tree_webdav_direct_upload);
        assert!(!plan.allow_rolling_upload);
        assert_eq!(plan.fallback_reason, None);
    }

    #[test]
    fn planner_falls_back_from_auto_direct_mode_with_reason() {
        let mut input = filesystem_webdav_input();
        input.target_capabilities.supports_raw_tree_direct_upload = false;

        let plan = plan_execution(&input).expect("plan");
        assert_eq!(plan.mode, ExecutionPlanModeV1::StagedUpload);
        assert_eq!(
            plan.fallback_reason,
            Some(ExecutionPlanFallbackReasonV1::TargetMissingRawTreeDirectCapability)
        );
    }

    #[test]
    fn planner_rejects_required_direct_mode_when_unsupported() {
        let mut input = filesystem_webdav_input();
        input.direct_upload_preference = DirectUploadPreferenceV1::On;
        input.target_capabilities.supports_raw_tree_direct_upload = false;

        let err = plan_execution(&input).expect_err("must fail");
        assert_eq!(err.code(), "planner_unsupported_required_mode");
        assert_eq!(
            err.fallback_reason,
            Some(ExecutionPlanFallbackReasonV1::TargetMissingRawTreeDirectCapability)
        );
    }

    #[test]
    fn planner_selects_rolling_upload_for_archive_when_allowed() {
        let mut input = filesystem_webdav_input();
        input.artifact_format = ArtifactFormatV1::ArchiveV1;
        input.direct_upload_preference = DirectUploadPreferenceV1::Off;

        let plan = plan_execution(&input).expect("plan");
        assert_eq!(plan.mode, ExecutionPlanModeV1::RollingUpload);
        assert!(plan.allow_rolling_upload);
        assert!(!plan.enable_raw_tree_webdav_direct_upload);
    }

    #[test]
    fn planner_falls_back_to_staged_when_policy_blocks_remote_upload() {
        let mut input = filesystem_webdav_input();
        input.artifact_format = ArtifactFormatV1::ArchiveV1;
        input.direct_upload_preference = DirectUploadPreferenceV1::Off;
        input.consistency_policy = Some(ConsistencyPolicyV1::Fail);
        input.upload_on_consistency_failure = Some(false);

        let plan = plan_execution(&input).expect("plan");
        assert_eq!(plan.mode, ExecutionPlanModeV1::StagedUpload);
        assert_eq!(
            plan.fallback_reason,
            Some(ExecutionPlanFallbackReasonV1::ConsistencyFailPolicyBlocksRollingUpload)
        );
    }

    #[test]
    fn planner_marks_local_dir_raw_tree_link_mode() {
        let mut input = filesystem_webdav_input();
        input.target_driver = PlannerDriverRefV1::new("local_dir", 1).expect("target");
        input.direct_upload_preference = DirectUploadPreferenceV1::Off;

        let plan = plan_execution(&input).expect("plan");
        assert_eq!(plan.mode, ExecutionPlanModeV1::StagedUpload);
        assert!(plan.link_stage_data_to_local_target);
    }

    #[test]
    fn planner_is_deterministic_for_same_inputs() {
        let input = filesystem_webdav_input();
        let p1 = plan_execution(&input).expect("plan");
        let p2 = plan_execution(&input).expect("plan");
        assert_eq!(p1, p2);
    }

    #[test]
    fn execution_planner_matrix_covers_supported_source_target_format_combinations() {
        struct Case {
            name: &'static str,
            input: ExecutionPlannerInputV1,
            expected_mode: ExecutionPlanModeV1,
        }

        let mut archive_webdav = filesystem_webdav_input();
        archive_webdav.artifact_format = ArtifactFormatV1::ArchiveV1;
        archive_webdav.direct_upload_preference = DirectUploadPreferenceV1::Off;

        let mut archive_local = filesystem_webdav_input();
        archive_local.target_driver = PlannerDriverRefV1::new("local_dir", 1).expect("target");
        archive_local.target_capabilities = PlannerTargetCapabilitiesV1 {
            supports_archive_rolling_upload: true,
            supports_raw_tree_direct_upload: false,
            supports_cleanup_run: true,
            supports_restore_reader: true,
        };
        archive_local.artifact_format = ArtifactFormatV1::ArchiveV1;
        archive_local.direct_upload_preference = DirectUploadPreferenceV1::Off;

        let mut raw_tree_local = filesystem_webdav_input();
        raw_tree_local.target_driver = PlannerDriverRefV1::new("local_dir", 1).expect("target");
        raw_tree_local.target_capabilities = PlannerTargetCapabilitiesV1 {
            supports_archive_rolling_upload: true,
            supports_raw_tree_direct_upload: false,
            supports_cleanup_run: true,
            supports_restore_reader: true,
        };
        raw_tree_local.direct_upload_preference = DirectUploadPreferenceV1::Off;

        let mut sqlite_archive_webdav = filesystem_webdav_input();
        sqlite_archive_webdav.source_driver = PlannerDriverRefV1::new("sqlite", 1).expect("source");
        sqlite_archive_webdav.source_capabilities = PlannerSourceCapabilitiesV1::default();
        sqlite_archive_webdav.artifact_format = ArtifactFormatV1::ArchiveV1;
        sqlite_archive_webdav.direct_upload_preference = DirectUploadPreferenceV1::Off;
        sqlite_archive_webdav.consistency_policy = None;
        sqlite_archive_webdav.upload_on_consistency_failure = None;

        let cases = vec![
            Case {
                name: "filesystem->webdav raw_tree direct",
                input: filesystem_webdav_input(),
                expected_mode: ExecutionPlanModeV1::RawTreeWebdavDirect,
            },
            Case {
                name: "filesystem->webdav archive rolling",
                input: archive_webdav,
                expected_mode: ExecutionPlanModeV1::RollingUpload,
            },
            Case {
                name: "filesystem->local_dir archive rolling",
                input: archive_local,
                expected_mode: ExecutionPlanModeV1::RollingUpload,
            },
            Case {
                name: "filesystem->local_dir raw_tree staged",
                input: raw_tree_local,
                expected_mode: ExecutionPlanModeV1::StagedUpload,
            },
            Case {
                name: "sqlite->webdav archive rolling",
                input: sqlite_archive_webdav,
                expected_mode: ExecutionPlanModeV1::RollingUpload,
            },
        ];

        for case in cases {
            let plan = plan_execution(&case.input).expect(case.name);
            assert_eq!(plan.mode, case.expected_mode, "{}", case.name);
        }
    }
}
