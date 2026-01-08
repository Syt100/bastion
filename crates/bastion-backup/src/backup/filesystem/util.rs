use std::fs::File;
use std::io::Read;
use std::path::{Component, Path};

use serde::Serialize;

pub(super) fn compile_globset(patterns: &[String]) -> Result<globset::GlobSet, anyhow::Error> {
    let mut builder = globset::GlobSetBuilder::new();
    for p in patterns {
        builder.add(globset::Glob::new(p)?);
    }
    Ok(builder.build()?)
}

pub(super) fn join_archive_path(prefix: &str, rel: &str) -> String {
    if prefix.is_empty() {
        rel.to_string()
    } else if rel.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}/{rel}")
    }
}

pub(super) fn archive_prefix_for_path(path: &Path) -> Result<String, anyhow::Error> {
    let mut components = Vec::<String>::new();
    for comp in path.components() {
        match comp {
            Component::Prefix(prefix) => {
                #[cfg(windows)]
                {
                    use std::path::Prefix as P;
                    match prefix.kind() {
                        P::Disk(letter) | P::VerbatimDisk(letter) => {
                            components.push((letter as char).to_ascii_uppercase().to_string());
                        }
                        P::UNC(server, share) | P::VerbatimUNC(server, share) => {
                            components.push("UNC".to_string());
                            components.push(server.to_string_lossy().to_string());
                            components.push(share.to_string_lossy().to_string());
                        }
                        _ => {
                            components.push(prefix.as_os_str().to_string_lossy().to_string());
                        }
                    }
                }
                #[cfg(not(windows))]
                {
                    components.push(prefix.as_os_str().to_string_lossy().to_string());
                }
            }
            Component::RootDir => {}
            Component::CurDir => {}
            Component::ParentDir => {
                anyhow::bail!("source path must not contain '..': {}", path.display())
            }
            Component::Normal(p) => {
                let s = p.to_string_lossy();
                if s.is_empty() {
                    continue;
                }
                components.push(s.to_string());
            }
        }
    }

    Ok(components.join("/"))
}

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

pub(super) fn write_json(path: &Path, value: &impl Serialize) -> Result<(), anyhow::Error> {
    let bytes = serde_json::to_vec_pretty(value)?;
    std::fs::write(path, bytes)?;
    Ok(())
}
