use base64::Engine as _;
use rand::RngCore;
use sha2::{Digest, Sha256};

pub fn generate_token_b64_urlsafe(size: usize) -> String {
    let mut bytes = vec![0_u8; size];
    rand::rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub fn sha256_b64_urlsafe(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize())
}

pub fn sha256_urlsafe_token(token: &str) -> Result<Vec<u8>, anyhow::Error> {
    let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| anyhow::anyhow!("invalid token encoding"))?;

    let mut hasher = Sha256::new();
    hasher.update(raw);
    Ok(hasher.finalize().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_token_b64_urlsafe_decodes_to_requested_size_and_has_no_padding() {
        let token = generate_token_b64_urlsafe(32);
        assert!(!token.contains('='));
        assert!(!token.contains('+'));
        assert!(!token.contains('/'));

        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(token.as_bytes())
            .expect("token should be valid base64url");
        assert_eq!(decoded.len(), 32);
    }

    #[test]
    fn sha256_b64_urlsafe_is_valid_base64url_and_is_32_bytes_when_decoded() {
        let digest_b64 = sha256_b64_urlsafe(b"abc");
        assert!(!digest_b64.contains('='));

        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(digest_b64.as_bytes())
            .expect("digest should be valid base64url");
        assert_eq!(decoded.len(), 32);

        let mut hasher = Sha256::new();
        hasher.update(b"abc");
        assert_eq!(decoded, hasher.finalize().to_vec());
    }

    #[test]
    fn sha256_urlsafe_token_hashes_the_decoded_token() -> Result<(), anyhow::Error> {
        let raw = vec![1_u8, 2, 3, 4, 5, 6];
        let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&raw);

        let hashed = sha256_urlsafe_token(&token)?;

        let mut hasher = Sha256::new();
        hasher.update(&raw);
        assert_eq!(hashed, hasher.finalize().to_vec());
        Ok(())
    }

    #[test]
    fn sha256_urlsafe_token_rejects_invalid_base64() {
        let err = sha256_urlsafe_token("%%%").expect_err("expected error");
        assert!(err.to_string().contains("invalid token encoding"));
    }
}
