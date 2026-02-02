use age::secrecy::ExposeSecret as _;
use sqlx::SqlitePool;
use tracing::{debug, info};

use crate::backup::PayloadEncryption;
use bastion_core::HUB_NODE_ID;
use bastion_core::job_spec;
use bastion_storage::secrets::SecretsCrypto;
use bastion_storage::secrets_repo;

pub const BACKUP_AGE_IDENTITY_KIND: &str = "backup_age_identity";

pub async fn get_age_identity(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    key_name: &str,
) -> Result<Option<String>, anyhow::Error> {
    let key_name = key_name.trim();
    if key_name.is_empty() {
        return Ok(None);
    }

    let Some(bytes) =
        secrets_repo::get_secret(db, secrets, HUB_NODE_ID, BACKUP_AGE_IDENTITY_KIND, key_name)
            .await?
    else {
        return Ok(None);
    };

    let identity = String::from_utf8(bytes)?.trim().to_string();
    if identity.is_empty() {
        return Ok(None);
    }
    Ok(Some(identity))
}

pub async fn ensure_age_identity(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    key_name: &str,
) -> Result<String, anyhow::Error> {
    let key_name = key_name.trim();
    if key_name.is_empty() {
        anyhow::bail!("backup age identity key_name is empty");
    }

    if let Some(existing) = get_age_identity(db, secrets, key_name).await? {
        debug!(key_name = %key_name, "using existing backup age identity");
        return Ok(existing);
    }

    let identity = age::x25519::Identity::generate();
    let identity_str = identity.to_string();
    secrets_repo::upsert_secret(
        db,
        secrets,
        HUB_NODE_ID,
        BACKUP_AGE_IDENTITY_KIND,
        key_name,
        identity_str.expose_secret().as_bytes(),
    )
    .await?;

    info!(key_name = %key_name, "created backup age identity");
    Ok(identity_str.expose_secret().to_string())
}

pub async fn distribute_age_identity_to_node(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    node_id: &str,
    key_name: &str,
) -> Result<(), anyhow::Error> {
    let node_id = node_id.trim();
    if node_id.is_empty() {
        anyhow::bail!("node_id is required");
    }

    let key_name = key_name.trim();
    if key_name.is_empty() {
        anyhow::bail!("backup age identity key_name is empty");
    }

    let Some(bytes) =
        secrets_repo::get_secret(db, secrets, HUB_NODE_ID, BACKUP_AGE_IDENTITY_KIND, key_name)
            .await?
    else {
        anyhow::bail!("missing backup age identity: {}", key_name);
    };

    // Re-encrypt under the destination node scope (associated data includes node_id).
    secrets_repo::upsert_secret(
        db,
        secrets,
        node_id,
        BACKUP_AGE_IDENTITY_KIND,
        key_name,
        &bytes,
    )
    .await?;

    Ok(())
}

pub async fn ensure_payload_encryption(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    pipeline: &job_spec::PipelineV1,
) -> Result<PayloadEncryption, anyhow::Error> {
    if pipeline.format == bastion_core::manifest::ArtifactFormatV1::RawTreeV1
        && !matches!(pipeline.encryption, job_spec::EncryptionV1::None)
    {
        anyhow::bail!("pipeline.encryption is not supported when pipeline.format is raw_tree_v1");
    }

    match &pipeline.encryption {
        job_spec::EncryptionV1::None => Ok(PayloadEncryption::None),
        job_spec::EncryptionV1::AgeX25519 { key_name } => {
            use std::str::FromStr as _;

            let identity_str = ensure_age_identity(db, secrets, key_name).await?;
            let identity = age::x25519::Identity::from_str(identity_str.trim())
                .map_err(|e| anyhow::anyhow!(e))?;
            let recipient = identity.to_public().to_string();
            debug!(key_name = %key_name.trim(), "resolved payload encryption");
            Ok(PayloadEncryption::AgeX25519 {
                recipient,
                key_name: key_name.trim().to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use tempfile::TempDir;

    use bastion_core::manifest::ArtifactFormatV1;
    use bastion_storage::{db, secrets::SecretsCrypto, secrets_repo};

    use super::{
        BACKUP_AGE_IDENTITY_KIND, distribute_age_identity_to_node, ensure_age_identity,
        ensure_payload_encryption, get_age_identity,
    };

    #[tokio::test]
    async fn get_age_identity_returns_none_for_blank_key_name() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        assert!(
            get_age_identity(&pool, &crypto, " ")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn ensure_age_identity_rejects_blank_key_name() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        assert!(ensure_age_identity(&pool, &crypto, " ").await.is_err());
    }

    #[tokio::test]
    async fn ensure_age_identity_creates_and_persists_identity() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        let created = ensure_age_identity(&pool, &crypto, "primary")
            .await
            .unwrap();
        assert!(!created.trim().is_empty());

        // Stored value should be readable and parseable as an age x25519 identity.
        let stored = get_age_identity(&pool, &crypto, "primary")
            .await
            .unwrap()
            .expect("identity stored");
        assert_eq!(stored.trim(), created.trim());
        age::x25519::Identity::from_str(stored.trim()).unwrap();

        // Second call should return the stored identity (no rotation on read).
        let second = ensure_age_identity(&pool, &crypto, "primary")
            .await
            .unwrap();
        assert_eq!(second.trim(), created.trim());
    }

    #[tokio::test]
    async fn distribute_age_identity_to_node_fails_when_missing() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        let err = distribute_age_identity_to_node(&pool, &crypto, "node1", "missing")
            .await
            .unwrap_err();
        assert!(format!("{err:#}").contains("missing backup age identity"));
    }

    #[tokio::test]
    async fn distribute_age_identity_to_node_reencrypts_under_destination_scope() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        ensure_age_identity(&pool, &crypto, "primary")
            .await
            .unwrap();
        let hub_plain = secrets_repo::get_secret(
            &pool,
            &crypto,
            bastion_core::HUB_NODE_ID,
            BACKUP_AGE_IDENTITY_KIND,
            "primary",
        )
        .await
        .unwrap()
        .expect("hub secret");

        distribute_age_identity_to_node(&pool, &crypto, "node1", "primary")
            .await
            .unwrap();
        let node_plain =
            secrets_repo::get_secret(&pool, &crypto, "node1", BACKUP_AGE_IDENTITY_KIND, "primary")
                .await
                .unwrap()
                .expect("node secret");

        assert_eq!(node_plain, hub_plain);
        let node_str = String::from_utf8(node_plain).unwrap();
        age::x25519::Identity::from_str(node_str.trim()).unwrap();
    }

    #[tokio::test]
    async fn ensure_payload_encryption_none_is_noop() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        let pipeline = bastion_core::job_spec::PipelineV1 {
            format: ArtifactFormatV1::ArchiveV1,
            encryption: bastion_core::job_spec::EncryptionV1::None,
        };

        let enc = ensure_payload_encryption(&pool, &crypto, &pipeline)
            .await
            .unwrap();
        assert!(matches!(enc, crate::backup::PayloadEncryption::None));
    }

    #[tokio::test]
    async fn ensure_payload_encryption_age_returns_recipient_and_key() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        let pipeline = bastion_core::job_spec::PipelineV1 {
            format: ArtifactFormatV1::ArchiveV1,
            encryption: bastion_core::job_spec::EncryptionV1::AgeX25519 {
                key_name: "primary".to_string(),
            },
        };

        let enc = ensure_payload_encryption(&pool, &crypto, &pipeline)
            .await
            .unwrap();
        match enc {
            crate::backup::PayloadEncryption::AgeX25519 {
                recipient,
                key_name,
            } => {
                assert_eq!(key_name, "primary");
                assert!(recipient.starts_with("age1"));
                age::x25519::Recipient::from_str(&recipient).unwrap();
            }
            other => panic!("unexpected payload encryption: {other:?}"),
        }
    }

    #[tokio::test]
    async fn ensure_payload_encryption_rejects_raw_tree_with_encryption() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();
        let crypto = SecretsCrypto::load_or_create(tmp.path()).unwrap();

        let pipeline = bastion_core::job_spec::PipelineV1 {
            format: ArtifactFormatV1::RawTreeV1,
            encryption: bastion_core::job_spec::EncryptionV1::AgeX25519 {
                key_name: "primary".to_string(),
            },
        };

        assert!(
            ensure_payload_encryption(&pool, &crypto, &pipeline)
                .await
                .is_err()
        );
    }
}
