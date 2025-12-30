use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use base64::Engine as _;
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use time::OffsetDateTime;

const KEYRING_VERSION: u32 = 1;
const ACTIVE_KID_START: u32 = 1;
const MASTER_KEY_FILE: &str = "master.key";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyEntryFile {
    kid: u32,
    key_b64: String,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyringFile {
    version: u32,
    active_kid: u32,
    keys: Vec<KeyEntryFile>,
}

#[derive(Debug, Clone)]
struct KeyEntry {
    kid: u32,
    key: [u8; 32],
    created_at: i64,
}

#[derive(Debug, Clone)]
pub struct EncryptedSecret {
    pub kid: u32,
    pub nonce: [u8; 24],
    pub ciphertext: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SecretsCrypto {
    active_kid: u32,
    keys: HashMap<u32, KeyEntry>,
}

impl SecretsCrypto {
    pub fn load_or_create(data_dir: &Path) -> Result<Self, anyhow::Error> {
        let path = data_dir.join(MASTER_KEY_FILE);
        let keyring = if path.exists() {
            read_keyring(&path)?
        } else {
            let keyring = generate_keyring();
            write_keyring_atomic(&path, &keyring)?;
            keyring
        };

        let mut keys = HashMap::new();
        for entry in keyring.keys {
            keys.insert(
                entry.kid,
                KeyEntry {
                    kid: entry.kid,
                    key: decode_key(&entry.key_b64)?,
                    created_at: entry.created_at,
                },
            );
        }

        Ok(Self {
            active_kid: keyring.active_kid,
            keys,
        })
    }

    pub fn active_kid(&self) -> u32 {
        self.active_kid
    }

    pub fn encrypt(
        &self,
        kind: &str,
        name: &str,
        plaintext: &[u8],
    ) -> Result<EncryptedSecret, anyhow::Error> {
        let key = self
            .keys
            .get(&self.active_kid)
            .ok_or_else(|| io::Error::other("active key not found"))?;

        let derived = derive_secrets_key(&key.key)?;
        let cipher = XChaCha20Poly1305::new((&derived).into());

        let mut nonce = [0_u8; 24];
        rand::rng().fill_bytes(&mut nonce);

        let aad = format!("{kind}:{name}");
        let ciphertext = cipher
            .encrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: plaintext,
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        Ok(EncryptedSecret {
            kid: key.kid,
            nonce,
            ciphertext,
        })
    }

    pub fn decrypt(
        &self,
        kind: &str,
        name: &str,
        secret: &EncryptedSecret,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let key = self
            .keys
            .get(&secret.kid)
            .ok_or_else(|| io::Error::other("key id not found"))?;

        let derived = derive_secrets_key(&key.key)?;
        let cipher = XChaCha20Poly1305::new((&derived).into());

        let aad = format!("{kind}:{name}");
        let plaintext = cipher
            .decrypt(
                XNonce::from_slice(&secret.nonce),
                Payload {
                    msg: &secret.ciphertext,
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        Ok(plaintext)
    }
}

fn generate_keyring() -> KeyringFile {
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

fn read_keyring(path: &Path) -> Result<KeyringFile, anyhow::Error> {
    let bytes = fs::read(path)?;
    let keyring: KeyringFile = serde_json::from_slice(&bytes)?;
    if keyring.version != KEYRING_VERSION {
        return Err(io::Error::other("unsupported master.key version").into());
    }
    Ok(keyring)
}

fn write_keyring_atomic(path: &Path, keyring: &KeyringFile) -> Result<(), anyhow::Error> {
    let bytes = serde_json::to_vec_pretty(keyring)?;
    let tmp = temp_path(path);
    fs::write(&tmp, bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))?;
    }

    fs::rename(tmp, path)?;
    Ok(())
}

fn temp_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("master.key");
    path.with_file_name(format!("{file_name}.tmp"))
}

fn decode_key(key_b64: &str) -> Result<[u8; 32], anyhow::Error> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(key_b64)?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid key length"))?;
    Ok(bytes)
}

fn derive_secrets_key(master_key: &[u8; 32]) -> Result<[u8; 32], anyhow::Error> {
    let hk = Hkdf::<Sha256>::new(None, master_key);
    let mut out = [0_u8; 32];
    hk.expand(b"secrets-v1", &mut out)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(out)
}
