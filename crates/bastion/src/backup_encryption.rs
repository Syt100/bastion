use age::secrecy::ExposeSecret as _;
use sqlx::SqlitePool;

use crate::backup::PayloadEncryption;
use crate::job_spec;
use crate::secrets::SecretsCrypto;
use crate::secrets_repo;

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
        secrets_repo::get_secret(db, secrets, BACKUP_AGE_IDENTITY_KIND, key_name).await?
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
        return Ok(existing);
    }

    let identity = age::x25519::Identity::generate();
    let identity_str = identity.to_string();
    secrets_repo::upsert_secret(
        db,
        secrets,
        BACKUP_AGE_IDENTITY_KIND,
        key_name,
        identity_str.expose_secret().as_bytes(),
    )
    .await?;

    Ok(identity_str.expose_secret().to_string())
}

pub async fn ensure_payload_encryption(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    pipeline: &job_spec::PipelineV1,
) -> Result<PayloadEncryption, anyhow::Error> {
    match &pipeline.encryption {
        job_spec::EncryptionV1::None => Ok(PayloadEncryption::None),
        job_spec::EncryptionV1::AgeX25519 { key_name } => {
            use std::str::FromStr as _;

            let identity_str = ensure_age_identity(db, secrets, key_name).await?;
            let identity = age::x25519::Identity::from_str(identity_str.trim())
                .map_err(|e| anyhow::anyhow!(e))?;
            let recipient = identity.to_public().to_string();
            Ok(PayloadEncryption::AgeX25519 {
                recipient,
                key_name: key_name.trim().to_string(),
            })
        }
    }
}
