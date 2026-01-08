use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use bastion_core::manifest::{HashAlgorithm, ManifestV1};

use super::access::TargetAccess;

pub(super) async fn fetch_parts(
    access: &TargetAccess,
    manifest: &ManifestV1,
    staging_dir: &Path,
) -> Result<Vec<PathBuf>, anyhow::Error> {
    let mut parts = Vec::with_capacity(manifest.artifacts.len());
    for part in &manifest.artifacts {
        match access {
            TargetAccess::Webdav { client, run_url } => {
                let dst = staging_dir.join(&part.name);
                if let Ok(meta) = tokio::fs::metadata(&dst).await
                    && meta.len() == part.size
                {
                    parts.push(dst);
                    continue;
                }

                let url = run_url.join(&part.name)?;
                client.get_to_file(&url, &dst, Some(part.size), 3).await?;
                parts.push(dst);
            }
            TargetAccess::LocalDir { run_dir } => {
                parts.push(run_dir.join(&part.name));
            }
        }
    }

    // Verify part hashes (blocking).
    let expected = manifest
        .artifacts
        .iter()
        .map(|p| (p.size, p.hash_alg.clone(), p.hash.clone()))
        .collect::<Vec<_>>();
    let parts_clone = parts.clone();
    tokio::task::spawn_blocking(move || verify_parts(&parts_clone, &expected)).await??;

    Ok(parts)
}

fn verify_parts(
    parts: &[PathBuf],
    expected: &[(u64, HashAlgorithm, String)],
) -> Result<(), anyhow::Error> {
    for (idx, path) in parts.iter().enumerate() {
        let (size, alg, hash) = expected
            .get(idx)
            .ok_or_else(|| anyhow::anyhow!("missing expected part info"))?;
        let meta = std::fs::metadata(path)?;
        if meta.len() != *size {
            anyhow::bail!(
                "part size mismatch for {}: expected {}, got {}",
                path.display(),
                size,
                meta.len()
            );
        }
        match alg {
            HashAlgorithm::Blake3 => {
                let computed = hash_file_blake3(path)?;
                if &computed != hash {
                    anyhow::bail!(
                        "part hash mismatch for {}: expected {}, got {}",
                        path.display(),
                        hash,
                        computed
                    );
                }
            }
            other => anyhow::bail!("unsupported part hash algorithm: {other:?}"),
        }
    }
    Ok(())
}

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
