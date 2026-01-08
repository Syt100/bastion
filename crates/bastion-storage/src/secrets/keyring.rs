use std::{fs, io, path::Path};

use base64::Engine as _;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::{ACTIVE_KID_START, KEYRING_VERSION, MASTER_KEY_FILE};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct KeyEntryFile {
    pub(super) kid: u32,
    pub(super) key_b64: String,
    pub(super) created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct KeyringFile {
    pub(super) version: u32,
    pub(super) active_kid: u32,
    pub(super) keys: Vec<KeyEntryFile>,
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyRotationResult {
    pub previous_kid: u32,
    pub active_kid: u32,
    pub keys_count: usize,
}

pub(super) fn generate_keyring() -> KeyringFile {
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let mut key = [0_u8; 32];
    rand::rng().fill_bytes(&mut key);

    let key_b64 = base64::engine::general_purpose::STANDARD.encode(key);
    KeyringFile {
        version: KEYRING_VERSION,
        active_kid: ACTIVE_KID_START,
        keys: vec![KeyEntryFile {
            kid: ACTIVE_KID_START,
            key_b64,
            created_at: now,
        }],
    }
}

pub(super) fn read_keyring(path: &Path) -> Result<KeyringFile, anyhow::Error> {
    let bytes = fs::read(path)?;
    let keyring: KeyringFile = serde_json::from_slice(&bytes)?;
    if keyring.version != KEYRING_VERSION {
        return Err(io::Error::other("unsupported master.key version").into());
    }
    Ok(keyring)
}

pub(super) fn write_keyring_atomic(
    path: &Path,
    keyring: &KeyringFile,
) -> Result<(), anyhow::Error> {
    let bytes = serde_json::to_vec_pretty(keyring)?;
    let tmp = super::io::temp_path(path);
    fs::write(&tmp, bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))?;
    }

    fs::rename(tmp, path)?;
    Ok(())
}

pub(super) fn decode_key(key_b64: &str) -> Result<[u8; 32], anyhow::Error> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(key_b64)?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid key length"))?;
    Ok(bytes)
}

pub(super) fn validate_keyring(keyring: &KeyringFile) -> Result<(), anyhow::Error> {
    if keyring.version != KEYRING_VERSION {
        return Err(io::Error::other("unsupported master.key version").into());
    }

    if keyring.keys.is_empty() {
        return Err(io::Error::other("master.key has no keys").into());
    }

    if !keyring.keys.iter().any(|k| k.kid == keyring.active_kid) {
        return Err(io::Error::other("active_kid not found in master.key keys").into());
    }

    for entry in &keyring.keys {
        let _ = decode_key(&entry.key_b64)?;
    }

    Ok(())
}

pub fn rotate_master_key(data_dir: &Path) -> Result<KeyRotationResult, anyhow::Error> {
    let path = data_dir.join(MASTER_KEY_FILE);
    let mut keyring = if path.exists() {
        read_keyring(&path)?
    } else {
        generate_keyring()
    };

    validate_keyring(&keyring)?;

    let previous_kid = keyring.active_kid;
    let next_kid = keyring.keys.iter().map(|k| k.kid).max().unwrap_or(0) + 1;
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let mut key = [0_u8; 32];
    rand::rng().fill_bytes(&mut key);
    let key_b64 = base64::engine::general_purpose::STANDARD.encode(key);

    keyring.keys.push(KeyEntryFile {
        kid: next_kid,
        key_b64,
        created_at: now,
    });
    keyring.active_kid = next_kid;

    write_keyring_atomic(&path, &keyring)?;

    Ok(KeyRotationResult {
        previous_kid,
        active_kid: next_kid,
        keys_count: keyring.keys.len(),
    })
}
