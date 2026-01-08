use std::path::Path;

use serde::{Deserialize, Serialize};

use bastion_core::agent_protocol::WebdavSecretV1;
use bastion_storage::secrets::SecretsCrypto;

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
struct WebdavSecretPayload {
    username: String,
    password: String,
}

pub(in super::super) fn save_managed_secrets_snapshot(
    data_dir: &Path,
    node_id: &str,
    issued_at: i64,
    webdav: &[WebdavSecretV1],
) -> Result<(), anyhow::Error> {
    let crypto = SecretsCrypto::load_or_create(data_dir)?;

    let saved_at = time::OffsetDateTime::now_utc().unix_timestamp();
    let mut entries = Vec::with_capacity(webdav.len());
    for secret in webdav {
        let payload = WebdavSecretPayload {
            username: secret.username.clone(),
            password: secret.password.clone(),
        };
        let bytes = serde_json::to_vec(&payload)?;
        let encrypted = crypto.encrypt(node_id, "webdav", &secret.name, &bytes)?;
        entries.push(ManagedWebdavSecretV1 {
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
        webdav: entries,
    };

    let path = managed_secrets_path(data_dir);
    write_json_pretty_atomic(&path, &doc)?;
    Ok(())
}
