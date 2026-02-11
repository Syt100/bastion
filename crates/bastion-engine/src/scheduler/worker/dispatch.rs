use serde::Deserialize;
use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;

use bastion_core::agent_protocol::{
    BackupRunTaskV1, DriverRefV1, HubToAgentMessageV1, PROTOCOL_VERSION, TargetDriverCapabilitiesV1,
};
use bastion_core::job_spec;
use bastion_driver_api::DriverId;
use bastion_driver_registry::builtins;
use bastion_storage::agent_tasks_repo;
use bastion_storage::jobs_repo;
use bastion_storage::secrets::SecretsCrypto;

use crate::agent_manager::AgentManager;
use crate::run_events;
use crate::run_events_bus::RunEventsBus;

pub(super) struct DispatchRunToAgentArgs<'a> {
    pub(super) db: &'a SqlitePool,
    pub(super) secrets: &'a SecretsCrypto,
    pub(super) agent_manager: &'a AgentManager,
    pub(super) run_events_bus: &'a RunEventsBus,
    pub(super) job: &'a jobs_repo::Job,
    pub(super) run_id: &'a str,
    pub(super) started_at: OffsetDateTime,
    pub(super) spec: job_spec::JobSpecV1,
    pub(super) agent_id: &'a str,
}

#[derive(Debug, Deserialize, Default)]
struct AgentHelloPayload {
    #[serde(default)]
    capabilities: AgentCapabilitiesPayload,
}

#[derive(Debug, Deserialize, Default)]
struct AgentCapabilitiesPayload {
    #[serde(default)]
    drivers: Option<AgentDriverCatalogPayload>,
}

#[derive(Debug, Deserialize, Default)]
struct AgentDriverCatalogPayload {
    #[serde(default)]
    source: Vec<DriverRefV1>,
    #[serde(default)]
    target: Vec<DriverRefV1>,
}

fn target_capabilities_for_driver(
    target: &DriverRefV1,
) -> Result<TargetDriverCapabilitiesV1, anyhow::Error> {
    let id = DriverId::new(target.kind.clone(), target.version)?;
    let caps = builtins::target_registry()
        .target_capabilities(&id)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    Ok(TargetDriverCapabilitiesV1 {
        supports_archive_rolling_upload: caps.supports_archive_rolling_upload,
        supports_raw_tree_direct_upload: caps.supports_raw_tree_direct_upload,
        supports_cleanup_run: caps.supports_cleanup_run,
        supports_restore_reader: caps.supports_restore_reader,
    })
}

fn build_task_driver_metadata(
    spec: &job_spec::JobSpecV1,
) -> Result<(DriverRefV1, DriverRefV1, TargetDriverCapabilitiesV1), anyhow::Error> {
    let canonical = job_spec::translate_v1_to_v2(spec)?;

    let source_driver = DriverRefV1 {
        kind: canonical.source.driver_type,
        version: canonical.source.version,
    };
    let target_driver = DriverRefV1 {
        kind: canonical.target.driver_type,
        version: canonical.target.version,
    };

    let target_capabilities = target_capabilities_for_driver(&target_driver)?;

    Ok((source_driver, target_driver, target_capabilities))
}

fn parse_advertised_drivers(capabilities_json: &str) -> Option<AgentDriverCatalogPayload> {
    let payload: AgentHelloPayload = serde_json::from_str(capabilities_json).ok()?;
    payload.capabilities.drivers
}

fn has_driver(drivers: &[DriverRefV1], required: &DriverRefV1) -> bool {
    drivers
        .iter()
        .any(|driver| driver.kind == required.kind && driver.version == required.version)
}

fn format_driver_list(drivers: &[DriverRefV1]) -> String {
    if drivers.is_empty() {
        return "<empty>".to_string();
    }

    drivers
        .iter()
        .map(|driver| format!("{}@{}", driver.kind, driver.version))
        .collect::<Vec<_>>()
        .join(", ")
}

fn validate_agent_driver_support(
    advertised: Option<&AgentDriverCatalogPayload>,
    required_source: &DriverRefV1,
    required_target: &DriverRefV1,
) -> Result<(), anyhow::Error> {
    // Compatibility mode: old agents may not advertise driver metadata at all.
    let Some(advertised) = advertised else {
        return Ok(());
    };

    if !has_driver(&advertised.source, required_source) {
        anyhow::bail!(
            "agent_driver_capability_mismatch: missing source driver {}@{} (advertised: {})",
            required_source.kind,
            required_source.version,
            format_driver_list(&advertised.source)
        );
    }

    if !has_driver(&advertised.target, required_target) {
        anyhow::bail!(
            "agent_driver_capability_mismatch: missing target driver {}@{} (advertised: {})",
            required_target.kind,
            required_target.version,
            format_driver_list(&advertised.target)
        );
    }

    Ok(())
}

async fn ensure_agent_supports_required_drivers(
    db: &SqlitePool,
    agent_id: &str,
    source_driver: &DriverRefV1,
    target_driver: &DriverRefV1,
) -> Result<(), anyhow::Error> {
    let row = sqlx::query("SELECT capabilities_json FROM agents WHERE id = ? LIMIT 1")
        .bind(agent_id)
        .fetch_optional(db)
        .await?;

    let advertised = row
        .and_then(|row| row.get::<Option<String>, _>("capabilities_json"))
        .as_deref()
        .and_then(parse_advertised_drivers);

    validate_agent_driver_support(advertised.as_ref(), source_driver, target_driver)
}

pub(super) async fn dispatch_run_to_agent(
    args: DispatchRunToAgentArgs<'_>,
) -> Result<(), anyhow::Error> {
    let DispatchRunToAgentArgs {
        db,
        secrets,
        agent_manager,
        run_events_bus,
        job,
        run_id,
        started_at,
        spec,
        agent_id,
    } = args;
    if !agent_manager.is_connected(agent_id).await {
        anyhow::bail!("agent not connected");
    }

    let (source_driver, target_driver, target_capabilities) = build_task_driver_metadata(&spec)?;
    ensure_agent_supports_required_drivers(db, agent_id, &source_driver, &target_driver).await?;

    run_events::append_and_broadcast(
        db,
        run_events_bus,
        run_id,
        "info",
        "dispatch",
        "dispatch",
        Some(serde_json::json!({
            "agent_id": agent_id,
            "source_driver": format!("{}@{}", source_driver.kind, source_driver.version),
            "target_driver": format!("{}@{}", target_driver.kind, target_driver.version)
        })),
    )
    .await?;

    let resolved =
        crate::agent_job_resolver::resolve_job_spec_for_agent(db, secrets, agent_id, spec).await?;
    let task = BackupRunTaskV1 {
        run_id: run_id.to_string(),
        job_id: job.id.clone(),
        started_at: started_at.unix_timestamp(),
        spec: resolved,
        source_driver: Some(source_driver),
        target_driver: Some(target_driver),
        target_capabilities: Some(target_capabilities),
    };

    // Use run_id as task_id for idempotency.
    let msg = HubToAgentMessageV1::Task {
        v: PROTOCOL_VERSION,
        task_id: run_id.to_string(),
        task: Box::new(task),
    };

    let payload = serde_json::to_value(&msg)?;
    agent_tasks_repo::upsert_task(db, run_id, agent_id, run_id, "sent", &payload).await?;

    agent_manager.send_json(agent_id, &msg).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use bastion_core::job_spec;

    use super::{
        AgentDriverCatalogPayload, DriverRefV1, build_task_driver_metadata,
        parse_advertised_drivers, validate_agent_driver_support,
    };

    #[test]
    fn parse_advertised_drivers_returns_none_for_legacy_payload() {
        let legacy = serde_json::json!({
            "type": "hello",
            "v": 1,
            "capabilities": {
                "backup": ["filesystem", "sqlite", "vaultwarden"],
                "control": ["fs_list"]
            }
        });

        let parsed = parse_advertised_drivers(&serde_json::to_string(&legacy).expect("json"));
        assert!(parsed.is_none());
    }

    #[test]
    fn validate_agent_driver_support_is_compatible_when_drivers_not_advertised() {
        let required_source = DriverRefV1 {
            kind: "filesystem".to_string(),
            version: 1,
        };
        let required_target = DriverRefV1 {
            kind: "webdav".to_string(),
            version: 1,
        };

        validate_agent_driver_support(None, &required_source, &required_target)
            .expect("legacy compatibility mode");
    }

    #[test]
    fn validate_agent_driver_support_rejects_missing_target_driver() {
        let advertised = AgentDriverCatalogPayload {
            source: vec![DriverRefV1 {
                kind: "filesystem".to_string(),
                version: 1,
            }],
            target: vec![DriverRefV1 {
                kind: "local_dir".to_string(),
                version: 1,
            }],
        };
        let required_source = DriverRefV1 {
            kind: "filesystem".to_string(),
            version: 1,
        };
        let required_target = DriverRefV1 {
            kind: "webdav".to_string(),
            version: 1,
        };

        let err =
            validate_agent_driver_support(Some(&advertised), &required_source, &required_target)
                .expect_err("must reject missing target driver");
        assert!(
            err.to_string().contains("missing target driver"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn build_task_driver_metadata_extracts_driver_envelopes() {
        let spec = job_spec::JobSpecV1::Filesystem {
            v: 1,
            pipeline: Default::default(),
            notifications: Default::default(),
            retention: Default::default(),
            source: job_spec::FilesystemSource {
                pre_scan: true,
                paths: vec!["/tmp".to_string()],
                root: "".to_string(),
                include: vec![],
                exclude: vec![],
                symlink_policy: Default::default(),
                hardlink_policy: Default::default(),
                error_policy: Default::default(),
                snapshot_mode: Default::default(),
                snapshot_provider: None,
                consistency_policy: Default::default(),
                consistency_fail_threshold: None,
                upload_on_consistency_failure: None,
            },
            target: job_spec::TargetV1::LocalDir {
                base_dir: "/tmp/out".to_string(),
                part_size_bytes: 1024 * 1024,
            },
        };

        let (source, target, target_caps) = build_task_driver_metadata(&spec).expect("metadata");
        assert_eq!(source.kind, "filesystem");
        assert_eq!(source.version, 1);
        assert_eq!(target.kind, "local_dir");
        assert_eq!(target.version, 1);
        assert!(target_caps.supports_archive_rolling_upload);
        assert!(target_caps.supports_restore_reader);
    }
}
