use sqlx::SqlitePool;

use bastion_engine::agent_manager::AgentManager;
use bastion_engine::agent_snapshots;
use bastion_storage::secrets::SecretsCrypto;

pub(super) async fn send_node_secrets_snapshot(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    node_id: &str,
) -> Result<(), anyhow::Error> {
    agent_snapshots::send_node_secrets_snapshot(db, secrets, agent_manager, node_id).await
}

pub(in crate::http) async fn send_node_config_snapshot(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    node_id: &str,
) -> Result<(), anyhow::Error> {
    agent_snapshots::send_node_config_snapshot(db, secrets, agent_manager, node_id).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn config_snapshot_id_is_deterministic_and_order_independent() {
        use bastion_core::agent;
        use bastion_core::agent_protocol::{
            JobConfigV1, JobSpecResolvedV1, OverlapPolicyV1, TargetResolvedV1,
        };

        let spec = JobSpecResolvedV1::Filesystem {
            v: 1,
            pipeline: Default::default(),
            source: bastion_core::job_spec::FilesystemSource {
                pre_scan: true,
                paths: vec![],
                root: "/".to_string(),
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
            target: TargetResolvedV1::LocalDir {
                base_dir: "/tmp".to_string(),
                part_size_bytes: 1024,
            },
        };

        let a = JobConfigV1 {
            job_id: "a".to_string(),
            name: "a".to_string(),
            schedule: Some("*/5 * * * *".to_string()),
            schedule_timezone: Some("UTC".to_string()),
            overlap_policy: OverlapPolicyV1::Queue,
            updated_at: 1,
            spec: spec.clone(),
        };
        let b = JobConfigV1 {
            job_id: "b".to_string(),
            name: "b".to_string(),
            schedule: None,
            schedule_timezone: Some("UTC".to_string()),
            overlap_policy: OverlapPolicyV1::Reject,
            updated_at: 2,
            spec: spec.clone(),
        };

        let mut v1 = vec![a.clone(), b.clone()];
        v1.sort_by(|x, y| x.job_id.cmp(&y.job_id));
        let id1 = agent::sha256_b64_urlsafe(&serde_json::to_vec(&v1).unwrap());

        let mut v2 = vec![b, a];
        v2.sort_by(|x, y| x.job_id.cmp(&y.job_id));
        let id2 = agent::sha256_b64_urlsafe(&serde_json::to_vec(&v2).unwrap());

        assert_eq!(id1, id2);

        let mut v3 = v1;
        v3[0].name = "changed".to_string();
        let id3 = agent::sha256_b64_urlsafe(&serde_json::to_vec(&v3).unwrap());
        assert_ne!(id1, id3);
    }
}
