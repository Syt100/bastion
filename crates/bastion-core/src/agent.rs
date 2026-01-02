use base64::Engine as _;
use rand::RngCore;
use sha2::{Digest, Sha256};

pub fn generate_token_b64_urlsafe(size: usize) -> String {
    let mut bytes = vec![0_u8; size];
    rand::rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub fn sha256_urlsafe_token(token: &str) -> Result<Vec<u8>, anyhow::Error> {
    let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| anyhow::anyhow!("invalid token encoding"))?;

    let mut hasher = Sha256::new();
    hasher.update(raw);
    Ok(hasher.finalize().to_vec())
}
