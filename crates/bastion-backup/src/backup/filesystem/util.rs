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

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::{
        archive_prefix_for_path, compile_globset, hash_file, join_archive_path, write_json,
    };

    #[test]
    fn join_archive_path_handles_empty_inputs() {
        assert_eq!(join_archive_path("", "a/b"), "a/b");
        assert_eq!(join_archive_path("p", ""), "p");
        assert_eq!(join_archive_path("", ""), "");
        assert_eq!(join_archive_path("p", "a"), "p/a");
    }

    #[test]
    fn archive_prefix_for_path_rejects_parent_dir() {
        assert!(archive_prefix_for_path(std::path::Path::new("../etc")).is_err());
    }

    #[test]
    fn archive_prefix_for_path_strips_root_dir() {
        let prefix = archive_prefix_for_path(std::path::Path::new("/tmp/foo")).unwrap();
        assert_eq!(prefix, "tmp/foo");
    }

    #[test]
    fn compile_globset_matches_expected() {
        let set = compile_globset(&["*.txt".to_string(), "dir/**".to_string()]).unwrap();
        assert!(set.is_match("a.txt"));
        assert!(!set.is_match("a.log"));
        assert!(set.is_match("dir/a.log"));
        assert!(compile_globset(&["[".to_string()]).is_err());
    }

    #[test]
    fn hash_file_matches_blake3_hash() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("a.bin");
        std::fs::write(&path, b"hello").unwrap();

        let expected = blake3::hash(b"hello").to_hex().to_string();
        assert_eq!(hash_file(&path).unwrap(), expected);
    }

    #[test]
    fn write_json_writes_pretty_json() {
        #[derive(serde::Serialize)]
        struct V<'a> {
            k: &'a str,
        }

        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("v.json");
        write_json(&path, &V { k: "v" }).unwrap();

        let raw = std::fs::read(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        assert_eq!(parsed, serde_json::json!({ "k": "v" }));
    }
}
