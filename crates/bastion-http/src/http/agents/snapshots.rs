use bastion_core::agent;
use bastion_core::agent_protocol::{
    HubToAgentMessageV1, JobConfigV1, OverlapPolicyV1, PROTOCOL_VERSION, WebdavSecretV1,
};
use bastion_core::job_spec;
use serde::Deserialize;
use sqlx::SqlitePool;

use bastion_engine::agent_job_resolver;
use bastion_engine::agent_manager::AgentManager;
use bastion_storage::jobs_repo;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;

#[derive(Debug, Deserialize)]
struct WebdavSecretPayload {
    username: String,
    password: String,
}

pub(super) async fn send_node_secrets_snapshot(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    node_id: &str,
) -> Result<(), anyhow::Error> {
    let list = secrets_repo::list_secrets(db, node_id, "webdav").await?;

    let mut webdav = Vec::with_capacity(list.len());
    for entry in list {
        let Some(bytes) =
            secrets_repo::get_secret(db, secrets, node_id, "webdav", &entry.name).await?
        else {
            continue;
        };
        let payload: WebdavSecretPayload = serde_json::from_slice(&bytes)?;
        webdav.push(WebdavSecretV1 {
            name: entry.name,
            username: payload.username,
            password: payload.password,
            updated_at: entry.updated_at,
        });
    }

    let msg = HubToAgentMessageV1::SecretsSnapshot {
        v: PROTOCOL_VERSION,
        node_id: node_id.to_string(),
        issued_at: time::OffsetDateTime::now_utc().unix_timestamp(),
        webdav,
    };

    agent_manager.send_json(node_id, &msg).await?;
    Ok(())
}

pub(in crate::http) async fn send_node_config_snapshot(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    node_id: &str,
) -> Result<(), anyhow::Error> {
    let jobs = jobs_repo::list_jobs_for_agent(db, node_id).await?;

    let mut configs = Vec::with_capacity(jobs.len());
    for job in jobs {
        let spec = match job_spec::parse_value(&job.spec) {
            Ok(v) => v,
            Err(error) => {
                tracing::warn!(
                    node_id = %node_id,
                    job_id = %job.id,
                    error = %error,
                    "invalid job spec; skipping agent config snapshot job"
                );
                continue;
            }
        };
        if let Err(error) = job_spec::validate(&spec) {
            tracing::warn!(
                node_id = %node_id,
                job_id = %job.id,
                error = %error,
                "invalid job spec; skipping agent config snapshot job"
            );
            continue;
        }

        let resolved =
            agent_job_resolver::resolve_job_spec_for_agent(db, secrets, node_id, spec).await?;

        let overlap_policy = match job.overlap_policy {
            jobs_repo::OverlapPolicy::Reject => OverlapPolicyV1::Reject,
            jobs_repo::OverlapPolicy::Queue => OverlapPolicyV1::Queue,
        };

        configs.push(JobConfigV1 {
            job_id: job.id,
            name: job.name,
            schedule: job.schedule,
            schedule_timezone: Some(job.schedule_timezone),
            overlap_policy,
            updated_at: job.updated_at,
            spec: resolved,
        });
    }

    configs.sort_by(|a, b| a.job_id.cmp(&b.job_id));
    let snapshot_id = agent::sha256_b64_urlsafe(&serde_json::to_vec(&configs)?);

    let msg = HubToAgentMessageV1::ConfigSnapshot {
        v: PROTOCOL_VERSION,
        node_id: node_id.to_string(),
        snapshot_id: snapshot_id.clone(),
        issued_at: time::OffsetDateTime::now_utc().unix_timestamp(),
        jobs: configs,
    };

    let _ = agent_manager
        .send_config_snapshot_json(node_id, &snapshot_id, &msg)
        .await?;
    Ok(())
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
                paths: vec![],
                root: "/".to_string(),
                include: vec![],
                exclude: vec![],
                symlink_policy: Default::default(),
                hardlink_policy: Default::default(),
                error_policy: Default::default(),
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
