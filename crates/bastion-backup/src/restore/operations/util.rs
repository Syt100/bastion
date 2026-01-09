use std::path::{Path, PathBuf};

use bastion_core::manifest::ManifestV1;
use sqlx::SqlitePool;

use bastion_storage::secrets::SecretsCrypto;

use super::super::unpack::PayloadDecryption;

pub(in crate::restore::operations) async fn resolve_payload_decryption(
    db: &SqlitePool,
    secrets: &SecretsCrypto,
    manifest: &ManifestV1,
) -> Result<PayloadDecryption, anyhow::Error> {
    match manifest.pipeline.encryption.as_str() {
        "none" => Ok(PayloadDecryption::None),
        "age" => {
            let key_name = manifest
                .pipeline
                .encryption_key
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| anyhow::anyhow!("missing manifest.pipeline.encryption_key"))?;

            let identity = crate::backup_encryption::get_age_identity(db, secrets, key_name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing backup age identity: {}", key_name))?;
            Ok(PayloadDecryption::AgeX25519 { identity })
        }
        other => anyhow::bail!("unsupported manifest.pipeline.encryption: {}", other),
    }
}

pub(in crate::restore::operations) fn operation_dir(data_dir: &Path, op_id: &str) -> PathBuf {
    data_dir.join("operations").join(op_id)
}
