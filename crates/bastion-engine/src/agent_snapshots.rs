use bastion_core::agent;
use bastion_core::agent_protocol::{
    BackupAgeIdentitySecretV1, HubToAgentMessageV1, JobConfigV1, OverlapPolicyV1, PROTOCOL_VERSION,
    WebdavSecretV1,
};
use bastion_core::job_spec;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::agent_job_resolver;
use crate::agent_manager::AgentManager;
use bastion_storage::agents_repo;
use bastion_storage::jobs_repo;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;

#[derive(Debug, Deserialize)]
struct WebdavSecretPayload {
    username: String,
    password: String,
}

pub async fn send_node_secrets_snapshot(
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

    let list = secrets_repo::list_secrets(db, node_id, "backup_age_identity").await?;
    let mut backup_age_identities = Vec::with_capacity(list.len());
    for entry in list {
        let Some(bytes) =
            secrets_repo::get_secret(db, secrets, node_id, "backup_age_identity", &entry.name)
                .await?
        else {
            continue;
        };

        let identity = String::from_utf8(bytes)?.trim().to_string();
        if identity.is_empty() {
            continue;
        }

        backup_age_identities.push(BackupAgeIdentitySecretV1 {
            name: entry.name,
            identity,
            updated_at: entry.updated_at,
        });
    }

    let msg = HubToAgentMessageV1::SecretsSnapshot {
        v: PROTOCOL_VERSION,
        node_id: node_id.to_string(),
        issued_at: time::OffsetDateTime::now_utc().unix_timestamp(),
        webdav,
        backup_age_identities,
    };

    agent_manager.send_json(node_id, &msg).await?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendConfigSnapshotOutcome {
    Sent,
    Unchanged,
    PendingOffline,
}

pub async fn send_node_config_snapshot(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    node_id: &str,
) -> Result<(), anyhow::Error> {
    let _ = send_node_config_snapshot_with_outcome(db, secrets, agent_manager, node_id).await?;
    Ok(())
}

pub async fn send_node_config_snapshot_with_outcome(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    agent_manager: &AgentManager,
    node_id: &str,
) -> Result<SendConfigSnapshotOutcome, anyhow::Error> {
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

    // Persist desired snapshot id regardless of whether the agent is currently connected.
    agents_repo::set_desired_config_snapshot(db, node_id, &snapshot_id).await?;

    let msg = HubToAgentMessageV1::ConfigSnapshot {
        v: PROTOCOL_VERSION,
        node_id: node_id.to_string(),
        snapshot_id: snapshot_id.clone(),
        issued_at: time::OffsetDateTime::now_utc().unix_timestamp(),
        jobs: configs,
    };

    match agent_manager
        .send_config_snapshot_json(node_id, &snapshot_id, &msg)
        .await
    {
        Ok(sent) => {
            agents_repo::clear_config_sync_error(db, node_id).await?;
            Ok(if sent {
                SendConfigSnapshotOutcome::Sent
            } else {
                SendConfigSnapshotOutcome::Unchanged
            })
        }
        Err(error) => {
            if error.to_string().contains("agent not connected") {
                return Ok(SendConfigSnapshotOutcome::PendingOffline);
            }
            agents_repo::record_config_sync_error(db, node_id, "send_failed", &error.to_string())
                .await?;
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Row;
    use tempfile::TempDir;

    use bastion_core::{agent, job_spec};
    use bastion_storage::jobs_repo::OverlapPolicy;
    use bastion_storage::secrets::SecretsCrypto;
    use bastion_storage::{db, jobs_repo};

    use super::{SendConfigSnapshotOutcome, send_node_config_snapshot_with_outcome};
    use crate::agent_manager::AgentManager;

    #[tokio::test]
    async fn offline_send_sets_desired_snapshot_and_returns_pending_offline() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");
        let crypto = SecretsCrypto::load_or_create(temp.path()).expect("crypto");

        let agent_id = "agent1";
        let agent_key = agent::generate_token_b64_urlsafe(32);
        let key_hash = agent::sha256_urlsafe_token(&agent_key).expect("hash");
        sqlx::query("INSERT INTO agents (id, name, key_hash, created_at) VALUES (?, NULL, ?, ?)")
            .bind(agent_id)
            .bind(key_hash)
            .bind(1i64)
            .execute(&pool)
            .await
            .expect("insert agent");

        let spec = job_spec::JobSpecV1::Filesystem {
            v: 1,
            pipeline: Default::default(),
            notifications: Default::default(),
            source: job_spec::FilesystemSource {
                paths: vec![],
                root: "/".to_string(),
                include: vec![],
                exclude: vec![],
                symlink_policy: Default::default(),
                hardlink_policy: Default::default(),
                error_policy: Default::default(),
            },
            target: job_spec::TargetV1::LocalDir {
                base_dir: "/tmp".to_string(),
                part_size_bytes: 2048,
            },
        };

        let _job = jobs_repo::create_job(
            &pool,
            "job1",
            Some(agent_id),
            None,
            Some("UTC"),
            OverlapPolicy::Reject,
            serde_json::to_value(spec).expect("spec json"),
        )
        .await
        .expect("create job");

        let agent_manager = AgentManager::default();
        let outcome =
            send_node_config_snapshot_with_outcome(&pool, &crypto, &agent_manager, agent_id)
                .await
                .expect("send");
        assert_eq!(outcome, SendConfigSnapshotOutcome::PendingOffline);

        let row = sqlx::query(
            "SELECT desired_config_snapshot_id, last_config_sync_error_kind FROM agents WHERE id = ? LIMIT 1",
        )
        .bind(agent_id)
        .fetch_one(&pool)
        .await
        .expect("fetch agent");
        assert!(
            row.get::<Option<String>, _>("desired_config_snapshot_id")
                .is_some()
        );
        assert!(
            row.get::<Option<String>, _>("last_config_sync_error_kind")
                .is_none()
        );
    }
}
