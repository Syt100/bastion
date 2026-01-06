use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use argon2::{Algorithm, Argon2, Params, Version};
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

const KEYPACK_VERSION: u32 = 1;
const KEYPACK_AAD: &[u8] = b"bastion-keypack-v1";

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
    #[allow(dead_code)]
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

        validate_keyring(&keyring)?;

        let mut keys = HashMap::new();
        for entry in keyring.keys {
            if keys.contains_key(&entry.kid) {
                return Err(io::Error::other("duplicate key id in master.key").into());
            }
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
        node_id: &str,
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

        let aad = format!("{node_id}:{kind}:{name}");
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
        node_id: &str,
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

        // v2 AAD includes node_id to prevent cross-node credential mixups.
        // v1 fallback: {kind}:{name} (kept for backward compatibility with existing databases).
        let aad_v2 = format!("{node_id}:{kind}:{name}");
        let try_v2 = cipher.decrypt(
            XNonce::from_slice(&secret.nonce),
            Payload {
                msg: &secret.ciphertext,
                aad: aad_v2.as_bytes(),
            },
        );

        match try_v2 {
            Ok(plaintext) => Ok(plaintext),
            Err(_) => {
                let aad_v1 = format!("{kind}:{name}");
                let plaintext = cipher
                    .decrypt(
                        XNonce::from_slice(&secret.nonce),
                        Payload {
                            msg: &secret.ciphertext,
                            aad: aad_v1.as_bytes(),
                        },
                    )
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                Ok(plaintext)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyRotationResult {
    pub previous_kid: u32,
    pub active_kid: u32,
    pub keys_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeypackFileV1 {
    version: u32,
    created_at: i64,
    kdf: KeypackKdfV1,
    cipher: KeypackCipherV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeypackKdfV1 {
    kind: String,
    salt_b64: String,
    mem_cost_kib: u32,
    time_cost: u32,
    parallelism: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeypackCipherV1 {
    kind: String,
    nonce_b64: String,
    ciphertext_b64: String,
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

pub fn export_keypack(
    data_dir: &Path,
    out_path: &Path,
    password: &str,
) -> Result<(), anyhow::Error> {
    if password.is_empty() {
        anyhow::bail!("password must not be empty");
    }

    let _ = SecretsCrypto::load_or_create(data_dir)?;
    let path = data_dir.join(MASTER_KEY_FILE);
    let keyring_file = read_keyring(&path)?;
    validate_keyring(&keyring_file)?;

    let plaintext = serde_json::to_vec_pretty(&keyring_file)?;

    let mut salt = [0_u8; 16];
    rand::rng().fill_bytes(&mut salt);

    let params =
        Params::new(64 * 1024, 3, 1, Some(32)).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let mem_cost_kib = params.m_cost();
    let time_cost = params.t_cost();
    let parallelism = params.p_cost();
    let mut derived = [0_u8; 32];
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(password.as_bytes(), &salt, &mut derived)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let cipher = XChaCha20Poly1305::new((&derived).into());
    let mut nonce = [0_u8; 24];
    rand::rng().fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: &plaintext,
                aad: KEYPACK_AAD,
            },
        )
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let pack = KeypackFileV1 {
        version: KEYPACK_VERSION,
        created_at: OffsetDateTime::now_utc().unix_timestamp(),
        kdf: KeypackKdfV1 {
            kind: "argon2id".to_string(),
            salt_b64: base64::engine::general_purpose::STANDARD.encode(salt),
            mem_cost_kib,
            time_cost,
            parallelism,
        },
        cipher: KeypackCipherV1 {
            kind: "xchacha20poly1305".to_string(),
            nonce_b64: base64::engine::general_purpose::STANDARD.encode(nonce),
            ciphertext_b64: base64::engine::general_purpose::STANDARD.encode(ciphertext),
        },
    };

    let bytes = serde_json::to_vec_pretty(&pack)?;
    write_file_atomic(out_path, &bytes)?;

    Ok(())
}

pub fn import_keypack(
    data_dir: &Path,
    in_path: &Path,
    password: &str,
    force: bool,
) -> Result<(), anyhow::Error> {
    if password.is_empty() {
        anyhow::bail!("password must not be empty");
    }

    let in_bytes = fs::read(in_path)?;
    let pack: KeypackFileV1 = serde_json::from_slice(&in_bytes)?;
    if pack.version != KEYPACK_VERSION {
        anyhow::bail!("unsupported keypack version");
    }
    if pack.kdf.kind != "argon2id" {
        anyhow::bail!("unsupported keypack kdf");
    }
    if pack.cipher.kind != "xchacha20poly1305" {
        anyhow::bail!("unsupported keypack cipher");
    }

    let salt = base64::engine::general_purpose::STANDARD.decode(&pack.kdf.salt_b64)?;
    let salt: [u8; 16] = salt
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid salt length"))?;

    let nonce = base64::engine::general_purpose::STANDARD.decode(&pack.cipher.nonce_b64)?;
    let nonce: [u8; 24] = nonce
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid nonce length"))?;

    let ciphertext =
        base64::engine::general_purpose::STANDARD.decode(&pack.cipher.ciphertext_b64)?;

    let params = Params::new(
        pack.kdf.mem_cost_kib,
        pack.kdf.time_cost,
        pack.kdf.parallelism,
        Some(32),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let mut derived = [0_u8; 32];
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(password.as_bytes(), &salt, &mut derived)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let cipher = XChaCha20Poly1305::new((&derived).into());
    let plaintext = cipher
        .decrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: &ciphertext,
                aad: KEYPACK_AAD,
            },
        )
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid password or keypack"))?;

    let keyring: KeyringFile = serde_json::from_slice(&plaintext)?;
    validate_keyring(&keyring)?;

    let path = data_dir.join(MASTER_KEY_FILE);
    if path.exists() && !force {
        anyhow::bail!("master.key already exists (use --force to overwrite)");
    }
    write_keyring_atomic(&path, &keyring)?;
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

fn validate_keyring(keyring: &KeyringFile) -> Result<(), anyhow::Error> {
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

fn derive_secrets_key(master_key: &[u8; 32]) -> Result<[u8; 32], anyhow::Error> {
    let hk = Hkdf::<Sha256>::new(None, master_key);
    let mut out = [0_u8; 32];
    hk.expand(b"secrets-v1", &mut out)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(out)
}

fn write_file_atomic(path: &Path, bytes: &[u8]) -> Result<(), anyhow::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

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

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::{
        EncryptedSecret, SecretsCrypto, export_keypack, import_keypack, rotate_master_key,
    };

    #[test]
    fn keypack_round_trip() {
        let temp = TempDir::new().unwrap();
        let data_dir = temp.path();

        let crypto1 = SecretsCrypto::load_or_create(data_dir).unwrap();
        let encrypted: EncryptedSecret = crypto1
            .encrypt("hub", "webdav", "primary", b"secret")
            .unwrap();

        let pack_path = data_dir.join("keypack.json");
        export_keypack(data_dir, &pack_path, "pw1").unwrap();

        let temp2 = TempDir::new().unwrap();
        import_keypack(temp2.path(), &pack_path, "pw1", false).unwrap();

        let crypto2 = SecretsCrypto::load_or_create(temp2.path()).unwrap();
        let plain = crypto2
            .decrypt("hub", "webdav", "primary", &encrypted)
            .unwrap();
        assert_eq!(plain, b"secret");

        assert!(import_keypack(temp2.path(), &pack_path, "pw1", false).is_err());
        import_keypack(temp2.path(), &pack_path, "pw1", true).unwrap();
    }

    #[test]
    fn keypack_wrong_password_fails() {
        let temp = TempDir::new().unwrap();
        let data_dir = temp.path();

        let pack_path = data_dir.join("keypack.json");
        export_keypack(data_dir, &pack_path, "pw1").unwrap();

        let temp2 = TempDir::new().unwrap();
        assert!(import_keypack(temp2.path(), &pack_path, "pw2", false).is_err());
    }

    #[test]
    fn rotate_preserves_old_keys() {
        let temp = TempDir::new().unwrap();
        let data_dir = temp.path();

        let crypto1 = SecretsCrypto::load_or_create(data_dir).unwrap();
        let encrypted1: EncryptedSecret = crypto1
            .encrypt("hub", "webdav", "primary", b"secret")
            .unwrap();
        assert_eq!(encrypted1.kid, crypto1.active_kid());

        let rotated = rotate_master_key(data_dir).unwrap();
        assert_ne!(rotated.previous_kid, rotated.active_kid);

        let crypto2 = SecretsCrypto::load_or_create(data_dir).unwrap();
        let encrypted2: EncryptedSecret = crypto2
            .encrypt("hub", "webdav", "primary", b"secret2")
            .unwrap();
        assert_eq!(encrypted2.kid, crypto2.active_kid());
        assert_ne!(encrypted1.kid, encrypted2.kid);

        let plain = crypto2
            .decrypt("hub", "webdav", "primary", &encrypted1)
            .unwrap();
        assert_eq!(plain, b"secret");
    }
}
