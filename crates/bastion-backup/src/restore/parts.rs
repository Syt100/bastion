use std::fs::File;
use std::io::Read;
use std::path::Path;

pub(super) fn hash_file_blake3(path: &Path) -> Result<String, anyhow::Error> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::hash_file_blake3;

    #[test]
    fn hash_file_blake3_matches_expected() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("a.bin");
        std::fs::write(&path, b"hello").unwrap();

        let expected = blake3::hash(b"hello").to_hex().to_string();
        assert_eq!(hash_file_blake3(&path).unwrap(), expected);
    }

    #[test]
    fn hash_file_blake3_hashes_empty_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("empty.bin");
        std::fs::write(&path, b"").unwrap();

        let expected = blake3::hash(b"").to_hex().to_string();
        assert_eq!(hash_file_blake3(&path).unwrap(), expected);
    }
}
