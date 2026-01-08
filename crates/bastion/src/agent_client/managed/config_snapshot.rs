use std::path::Path;

use base64::Engine as _;
use serde::{Deserialize, Serialize};

use bastion_core::agent_protocol::JobConfigV1;
use bastion_storage::secrets::{EncryptedSecret, SecretsCrypto};

use super::super::{MANAGED_CONFIG_KIND, MANAGED_CONFIG_NAME};
use super::ManagedConfigPlainV1;
use super::io::write_json_pretty_atomic;
use super::paths::managed_config_path;

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct ManagedConfigFileV1 {
    pub(super) v: u32,
    pub(super) node_id: String,
    pub(super) snapshot_id: String,
    pub(super) issued_at: i64,
    pub(super) saved_at: i64,
    pub(super) encrypted: EncryptedBlobV1,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct EncryptedBlobV1 {
    pub(super) kid: u32,
    pub(super) nonce_b64: String,
    pub(super) ciphertext_b64: String,
}

pub(in super::super) fn save_managed_config_snapshot(
    data_dir: &Path,
    node_id: &str,
    snapshot_id: &str,
    issued_at: i64,
    jobs: &[JobConfigV1],
) -> Result<(), anyhow::Error> {
    let crypto = SecretsCrypto::load_or_create(data_dir)?;

    let plain = ManagedConfigPlainV1 {
        v: 1,
        snapshot_id: snapshot_id.to_string(),
        issued_at,
        jobs: jobs.to_vec(),
    };
    let bytes = serde_json::to_vec(&plain)?;

    let encrypted = crypto.encrypt(node_id, MANAGED_CONFIG_KIND, MANAGED_CONFIG_NAME, &bytes)?;
    let encrypted = EncryptedBlobV1 {
        kid: encrypted.kid,
        nonce_b64: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(encrypted.nonce),
        ciphertext_b64: base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(encrypted.ciphertext),
    };

    let saved_at = time::OffsetDateTime::now_utc().unix_timestamp();
    let doc = ManagedConfigFileV1 {
        v: 1,
        node_id: node_id.to_string(),
        snapshot_id: snapshot_id.to_string(),
        issued_at,
        saved_at,
        encrypted,
    };

    let path = managed_config_path(data_dir);
    write_json_pretty_atomic(&path, &doc)?;
    Ok(())
}

pub(in super::super) fn load_managed_config_snapshot(
    data_dir: &Path,
    node_id: &str,
) -> Result<Option<ManagedConfigPlainV1>, anyhow::Error> {
    let path = managed_config_path(data_dir);
    let bytes = match std::fs::read(&path) {
        Ok(v) => v,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };

    let doc: ManagedConfigFileV1 = serde_json::from_slice(&bytes)?;
    if doc.v != 1 {
        anyhow::bail!("unsupported managed config snapshot version: {}", doc.v);
    }

    let nonce = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(doc.encrypted.nonce_b64.as_bytes())?;
    let ciphertext = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(doc.encrypted.ciphertext_b64.as_bytes())?;

    let nonce: [u8; 24] = nonce
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid nonce length"))?;

    let crypto = SecretsCrypto::load_or_create(data_dir)?;
    let secret = EncryptedSecret {
        kid: doc.encrypted.kid,
        nonce,
        ciphertext,
    };
    let plaintext = crypto.decrypt(node_id, MANAGED_CONFIG_KIND, MANAGED_CONFIG_NAME, &secret)?;
    let plain: ManagedConfigPlainV1 = serde_json::from_slice(&plaintext)?;
    Ok(Some(plain))
}
