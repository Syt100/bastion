use std::fs::File;
use std::io::Read;
use std::path::Path;

pub(super) fn hash_file(path: &Path) -> Result<String, anyhow::Error> {
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
    use super::hash_file;

    #[test]
    fn hash_file_matches_blake3_hash_for_known_contents() -> Result<(), anyhow::Error> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("data.bin");
        let contents = b"hello world";
        std::fs::write(&path, contents)?;

        let got = hash_file(&path)?;
        let expected = blake3::hash(contents).to_hex().to_string();
        assert_eq!(got, expected);
        Ok(())
    }

    #[test]
    fn hash_file_matches_blake3_hash_for_empty_file() -> Result<(), anyhow::Error> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("empty.bin");
        std::fs::write(&path, b"")?;

        let got = hash_file(&path)?;
        let expected = blake3::hash(&[]).to_hex().to_string();
        assert_eq!(got, expected);
        Ok(())
    }
}
