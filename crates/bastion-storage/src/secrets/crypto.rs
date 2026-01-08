use std::{collections::HashMap, io, path::Path};

use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

use super::MASTER_KEY_FILE;

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
            super::keyring::read_keyring(&path)?
        } else {
            let keyring = super::keyring::generate_keyring();
            super::keyring::write_keyring_atomic(&path, &keyring)?;
            keyring
        };

        super::keyring::validate_keyring(&keyring)?;

        let mut keys = HashMap::new();
        for entry in keyring.keys {
            if keys.contains_key(&entry.kid) {
                return Err(io::Error::other("duplicate key id in master.key").into());
            }
            keys.insert(
                entry.kid,
                KeyEntry {
                    kid: entry.kid,
                    key: super::keyring::decode_key(&entry.key_b64)?,
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

fn derive_secrets_key(master_key: &[u8; 32]) -> Result<[u8; 32], anyhow::Error> {
    let hk = Hkdf::<Sha256>::new(None, master_key);
    let mut out = [0_u8; 32];
    hk.expand(b"secrets-v1", &mut out)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(out)
}
