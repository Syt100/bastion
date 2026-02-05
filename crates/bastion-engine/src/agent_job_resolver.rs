use sqlx::SqlitePool;

use bastion_backup::backup_encryption;
use bastion_core::agent_protocol::{
    EncryptionResolvedV1, JobSpecResolvedV1, PipelineResolvedV1, TargetResolvedV1,
};
use bastion_core::job_spec;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;
use bastion_targets::WebdavCredentials;

pub async fn resolve_job_spec_for_agent(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    node_id: &str,
    spec: job_spec::JobSpecV1,
) -> Result<JobSpecResolvedV1, anyhow::Error> {
    match spec {
        job_spec::JobSpecV1::Filesystem {
            v,
            pipeline,
            notifications: _,
            source,
            target,
            ..
        } => Ok(JobSpecResolvedV1::Filesystem {
            v,
            pipeline: resolve_pipeline_for_agent(db, secrets, &pipeline).await?,
            source,
            target: resolve_target_for_agent(db, secrets, node_id, target).await?,
        }),
        job_spec::JobSpecV1::Sqlite {
            v,
            pipeline,
            notifications: _,
            source,
            target,
            ..
        } => Ok(JobSpecResolvedV1::Sqlite {
            v,
            pipeline: resolve_pipeline_for_agent(db, secrets, &pipeline).await?,
            source,
            target: resolve_target_for_agent(db, secrets, node_id, target).await?,
        }),
        job_spec::JobSpecV1::Vaultwarden {
            v,
            pipeline,
            notifications: _,
            source,
            target,
            ..
        } => Ok(JobSpecResolvedV1::Vaultwarden {
            v,
            pipeline: resolve_pipeline_for_agent(db, secrets, &pipeline).await?,
            source,
            target: resolve_target_for_agent(db, secrets, node_id, target).await?,
        }),
    }
}

async fn resolve_pipeline_for_agent(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    pipeline: &job_spec::PipelineV1,
) -> Result<PipelineResolvedV1, anyhow::Error> {
    let format = pipeline.format.clone();
    let encryption = backup_encryption::ensure_payload_encryption(db, secrets, pipeline).await?;
    let encryption = match encryption {
        bastion_backup::backup::PayloadEncryption::None => EncryptionResolvedV1::None,
        bastion_backup::backup::PayloadEncryption::AgeX25519 {
            recipient,
            key_name,
        } => EncryptionResolvedV1::AgeX25519 {
            recipient,
            key_name,
        },
    };
    Ok(PipelineResolvedV1 { format, encryption })
}

async fn resolve_target_for_agent(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    node_id: &str,
    target: job_spec::TargetV1,
) -> Result<TargetResolvedV1, anyhow::Error> {
    match target {
        job_spec::TargetV1::Webdav {
            base_url,
            secret_name,
            part_size_bytes,
        } => {
            let cred_bytes = secrets_repo::get_secret(db, secrets, node_id, "webdav", &secret_name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing webdav secret: {secret_name}"))?;
            let credentials = WebdavCredentials::from_json(&cred_bytes)?;
            Ok(TargetResolvedV1::Webdav {
                base_url,
                username: credentials.username,
                password: credentials.password,
                part_size_bytes,
            })
        }
        job_spec::TargetV1::LocalDir {
            base_dir,
            part_size_bytes,
        } => Ok(TargetResolvedV1::LocalDir {
            base_dir,
            part_size_bytes,
        }),
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use bastion_core::agent_protocol::{JobSpecResolvedV1, TargetResolvedV1};
    use bastion_core::job_spec;
    use bastion_storage::db;
    use bastion_storage::secrets::SecretsCrypto;
    use bastion_storage::secrets_repo;

    use super::resolve_job_spec_for_agent;

    #[tokio::test]
    async fn resolves_webdav_secret_for_agent_node() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");
        let crypto = SecretsCrypto::load_or_create(temp.path()).expect("crypto");

        secrets_repo::upsert_secret(
            &pool,
            &crypto,
            "agent1",
            "webdav",
            "primary",
            &serde_json::to_vec(&serde_json::json!({"username":"u","password":"p"})).unwrap(),
        )
        .await
        .expect("upsert secret");

        let spec = job_spec::JobSpecV1::Filesystem {
            v: 1,
            pipeline: Default::default(),
            notifications: Default::default(),
            retention: Default::default(),
            source: job_spec::FilesystemSource {
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
            target: job_spec::TargetV1::Webdav {
                base_url: "http://example.com/backup".to_string(),
                secret_name: "primary".to_string(),
                part_size_bytes: 1024,
            },
        };

        let resolved = resolve_job_spec_for_agent(&pool, &crypto, "agent1", spec)
            .await
            .expect("resolve");

        let JobSpecResolvedV1::Filesystem { target, .. } = resolved else {
            panic!("unexpected resolved type");
        };
        let TargetResolvedV1::Webdav {
            base_url,
            username,
            password,
            part_size_bytes,
        } = target
        else {
            panic!("expected webdav target");
        };
        assert_eq!(base_url, "http://example.com/backup");
        assert_eq!(username, "u");
        assert_eq!(password, "p");
        assert_eq!(part_size_bytes, 1024);
    }

    #[tokio::test]
    async fn local_dir_target_is_passed_through() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");
        let crypto = SecretsCrypto::load_or_create(temp.path()).expect("crypto");

        let spec = job_spec::JobSpecV1::Filesystem {
            v: 1,
            pipeline: Default::default(),
            notifications: Default::default(),
            retention: Default::default(),
            source: job_spec::FilesystemSource {
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
            target: job_spec::TargetV1::LocalDir {
                base_dir: "/tmp".to_string(),
                part_size_bytes: 2048,
            },
        };

        let resolved = resolve_job_spec_for_agent(&pool, &crypto, "agent1", spec)
            .await
            .expect("resolve");

        let JobSpecResolvedV1::Filesystem { target, .. } = resolved else {
            panic!("unexpected resolved type");
        };
        let TargetResolvedV1::LocalDir {
            base_dir,
            part_size_bytes,
        } = target
        else {
            panic!("expected local_dir target");
        };
        assert_eq!(base_dir, "/tmp");
        assert_eq!(part_size_bytes, 2048);
    }
}
