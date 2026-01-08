use std::{fs, io, path::Path};

use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine as _;
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::{KEYPACK_AAD, KEYPACK_VERSION, MASTER_KEY_FILE, SecretsCrypto};

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
    let keyring_file = super::keyring::read_keyring(&path)?;
    super::keyring::validate_keyring(&keyring_file)?;

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
    super::io::write_file_atomic(out_path, &bytes)?;

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

    let keyring: super::keyring::KeyringFile = serde_json::from_slice(&plaintext)?;
    super::keyring::validate_keyring(&keyring)?;

    let path = data_dir.join(MASTER_KEY_FILE);
    if path.exists() && !force {
        anyhow::bail!("master.key already exists (use --force to overwrite)");
    }
    super::keyring::write_keyring_atomic(&path, &keyring)?;
    Ok(())
}
