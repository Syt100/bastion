use std::path::Path;

use serde::{Deserialize, Serialize};

use bastion_core::agent_protocol::{BackupAgeIdentitySecretV1, WebdavSecretV1};
use bastion_storage::secrets::{EncryptedSecret, SecretsCrypto};

use super::io::write_json_pretty_atomic;
use super::paths::managed_secrets_path;

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct ManagedSecretsFileV1 {
    pub(super) v: u32,
    pub(super) node_id: String,
    pub(super) issued_at: i64,
    pub(super) saved_at: i64,
    #[serde(default)]
    pub(super) webdav: Vec<ManagedWebdavSecretV1>,
    #[serde(default)]
    pub(super) backup_age_identities: Vec<ManagedBackupAgeIdentitySecretV1>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct ManagedWebdavSecretV1 {
    pub(super) name: String,
    pub(super) updated_at: i64,
    pub(super) kid: u32,
    pub(super) nonce: Vec<u8>,
    pub(super) ciphertext: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct ManagedBackupAgeIdentitySecretV1 {
    pub(super) name: String,
    pub(super) updated_at: i64,
    pub(super) kid: u32,
    pub(super) nonce: Vec<u8>,
    pub(super) ciphertext: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebdavSecretPayload {
    username: String,
    password: String,
}

pub(in super::super) fn save_managed_secrets_snapshot(
    data_dir: &Path,
    node_id: &str,
    issued_at: i64,
    webdav: &[WebdavSecretV1],
    backup_age_identities: &[BackupAgeIdentitySecretV1],
) -> Result<(), anyhow::Error> {
    let crypto = SecretsCrypto::load_or_create(data_dir)?;

    let saved_at = time::OffsetDateTime::now_utc().unix_timestamp();
    let mut webdav_entries = Vec::with_capacity(webdav.len());
    for secret in webdav {
        let payload = WebdavSecretPayload {
            username: secret.username.clone(),
            password: secret.password.clone(),
        };
        let bytes = serde_json::to_vec(&payload)?;
        let encrypted = crypto.encrypt(node_id, "webdav", &secret.name, &bytes)?;
        webdav_entries.push(ManagedWebdavSecretV1 {
            name: secret.name.clone(),
            updated_at: secret.updated_at,
            kid: encrypted.kid,
            nonce: encrypted.nonce.to_vec(),
            ciphertext: encrypted.ciphertext,
        });
    }

    let mut age_entries = Vec::with_capacity(backup_age_identities.len());
    for secret in backup_age_identities {
        let identity = secret.identity.trim();
        if identity.is_empty() {
            continue;
        }
        let encrypted =
            crypto.encrypt(node_id, "backup_age_identity", &secret.name, identity.as_bytes())?;
        age_entries.push(ManagedBackupAgeIdentitySecretV1 {
            name: secret.name.clone(),
            updated_at: secret.updated_at,
            kid: encrypted.kid,
            nonce: encrypted.nonce.to_vec(),
            ciphertext: encrypted.ciphertext,
        });
    }

    let doc = ManagedSecretsFileV1 {
        v: 1,
        node_id: node_id.to_string(),
        issued_at,
        saved_at,
        webdav: webdav_entries,
        backup_age_identities: age_entries,
    };

    let path = managed_secrets_path(data_dir);
    write_json_pretty_atomic(&path, &doc)?;
    Ok(())
}

pub(in super::super) fn load_managed_backup_age_identity(
    data_dir: &Path,
    key_name: &str,
) -> Result<Option<String>, anyhow::Error> {
    let key_name = key_name.trim();
    if key_name.is_empty() {
        return Ok(None);
    }

    let path = managed_secrets_path(data_dir);
    let bytes = match std::fs::read(&path) {
        Ok(v) => v,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };

    let doc: ManagedSecretsFileV1 = serde_json::from_slice(&bytes)?;
    if doc.v != 1 {
        anyhow::bail!("unsupported managed secrets snapshot version: {}", doc.v);
    }
    let node_id = doc.node_id.as_str();

    let Some(entry) = doc
        .backup_age_identities
        .iter()
        .find(|e| e.name == key_name)
    else {
        return Ok(None);
    };

    let nonce: [u8; 24] = entry
        .nonce
        .clone()
        .try_into()
        .map_err(|_| anyhow::anyhow!("invalid nonce length"))?;
    let secret = EncryptedSecret {
        kid: entry.kid,
        nonce,
        ciphertext: entry.ciphertext.clone(),
    };

    let crypto = SecretsCrypto::load_or_create(data_dir)?;
    let plaintext = crypto.decrypt(node_id, "backup_age_identity", key_name, &secret)?;
    let identity = String::from_utf8(plaintext)?.trim().to_string();
    if identity.is_empty() {
        return Ok(None);
    }
    Ok(Some(identity))
}
