use std::path::{Path, PathBuf};
use std::process::Command;

use super::FilesystemReadMapping;

#[derive(Debug, Clone)]
pub struct SourceSnapshotHandle {
    pub provider: String,
    pub original_root: PathBuf,
    pub snapshot_root: PathBuf,
}

impl SourceSnapshotHandle {
    pub fn read_mapping(&self) -> FilesystemReadMapping {
        FilesystemReadMapping {
            original_root: self.original_root.clone(),
            read_root: self.snapshot_root.clone(),
        }
    }

    pub fn cleanup(&self) -> Result<(), anyhow::Error> {
        match self.provider.as_str() {
            "btrfs" => delete_btrfs_snapshot(&self.snapshot_root),
            other => anyhow::bail!("unknown snapshot provider: {other}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SnapshotUnavailable {
    pub provider: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum SnapshotAttempt {
    Ready(SourceSnapshotHandle),
    Unavailable(SnapshotUnavailable),
}

#[derive(Debug, Clone)]
struct SnapshotSettings {
    btrfs_enabled: bool,
    allowlist_prefixes: Vec<PathBuf>,
}

impl SnapshotSettings {
    fn from_env() -> Self {
        let allowlist = std::env::var("BASTION_FS_SNAPSHOT_ALLOWLIST").unwrap_or_default();
        let allowlist_prefixes = allowlist
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect::<Vec<_>>();

        Self {
            btrfs_enabled: env_bool("BASTION_FS_SNAPSHOT_BTRFS_ENABLED"),
            allowlist_prefixes,
        }
    }
}

pub fn attempt_source_snapshot(
    root: &Path,
    run_dir: &Path,
    provider_override: Option<&str>,
) -> SnapshotAttempt {
    let settings = SnapshotSettings::from_env();
    attempt_source_snapshot_inner(root, run_dir, provider_override, &settings)
}

fn attempt_source_snapshot_inner(
    root: &Path,
    run_dir: &Path,
    provider_override: Option<&str>,
    settings: &SnapshotSettings,
) -> SnapshotAttempt {
    let provider = match provider_override {
        Some(p) if p.trim().is_empty() => None,
        Some(p) => Some(p.trim().to_string()),
        None => None,
    };

    let provider_id = match provider.as_deref() {
        None => "btrfs",
        Some("btrfs") => "btrfs",
        Some(other) => {
            return SnapshotAttempt::Unavailable(SnapshotUnavailable {
                provider: Some(other.to_string()),
                reason: "unsupported snapshot provider".to_string(),
            });
        }
    };

    let probe = probe_btrfs(root, settings);
    if !probe.supported {
        return SnapshotAttempt::Unavailable(SnapshotUnavailable {
            provider: Some(provider_id.to_string()),
            reason: probe
                .reason
                .unwrap_or_else(|| "snapshot provider unavailable".to_string()),
        });
    }

    let snapshot_root = run_dir.join("source_snapshot").join(provider_id);
    match std::fs::remove_dir_all(&snapshot_root) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return SnapshotAttempt::Unavailable(SnapshotUnavailable {
                provider: Some(provider_id.to_string()),
                reason: format!("failed to clean existing snapshot dir: {error}"),
            });
        }
    };

    if let Some(parent) = snapshot_root.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        return SnapshotAttempt::Unavailable(SnapshotUnavailable {
            provider: Some(provider_id.to_string()),
            reason: format!("failed to create snapshot parent dir: {error}"),
        });
    }

    if let Err(error) = create_btrfs_snapshot(root, &snapshot_root) {
        return SnapshotAttempt::Unavailable(SnapshotUnavailable {
            provider: Some(provider_id.to_string()),
            reason: format!("snapshot create failed: {error}"),
        });
    }

    SnapshotAttempt::Ready(SourceSnapshotHandle {
        provider: provider_id.to_string(),
        original_root: root.to_path_buf(),
        snapshot_root,
    })
}

struct ProbeResult {
    supported: bool,
    reason: Option<String>,
}

fn probe_btrfs(root: &Path, settings: &SnapshotSettings) -> ProbeResult {
    if !settings.btrfs_enabled {
        return ProbeResult {
            supported: false,
            reason: Some("btrfs snapshot provider disabled by config".to_string()),
        };
    }

    if let Err(reason) = validate_allowlist_root(root, &settings.allowlist_prefixes) {
        return ProbeResult {
            supported: false,
            reason: Some(reason),
        };
    }

    let meta = match std::fs::metadata(root) {
        Ok(m) => m,
        Err(error) => {
            return ProbeResult {
                supported: false,
                reason: Some(format!("root metadata error: {error}")),
            };
        }
    };
    if !meta.is_dir() {
        return ProbeResult {
            supported: false,
            reason: Some("btrfs snapshots require a directory root".to_string()),
        };
    }

    let output = Command::new("btrfs")
        .arg("subvolume")
        .arg("show")
        .arg(root)
        .output();

    match output {
        Ok(out) if out.status.success() => ProbeResult {
            supported: true,
            reason: None,
        },
        Ok(out) => ProbeResult {
            supported: false,
            reason: Some(format!(
                "btrfs subvolume show failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ProbeResult {
            supported: false,
            reason: Some("btrfs command not found".to_string()),
        },
        Err(error) => ProbeResult {
            supported: false,
            reason: Some(format!("btrfs probe error: {error}")),
        },
    }
}

fn create_btrfs_snapshot(root: &Path, snapshot_root: &Path) -> Result<(), anyhow::Error> {
    let out = Command::new("btrfs")
        .arg("subvolume")
        .arg("snapshot")
        .arg("-r")
        .arg(root)
        .arg(snapshot_root)
        .output()?;
    if out.status.success() {
        return Ok(());
    }
    anyhow::bail!(
        "btrfs subvolume snapshot failed: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
}

fn delete_btrfs_snapshot(snapshot_root: &Path) -> Result<(), anyhow::Error> {
    let out = Command::new("btrfs")
        .arg("subvolume")
        .arg("delete")
        .arg(snapshot_root)
        .output()?;
    if out.status.success() {
        return Ok(());
    }
    anyhow::bail!(
        "btrfs subvolume delete failed: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
}

fn env_bool(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

fn validate_allowlist_root(root: &Path, prefixes: &[PathBuf]) -> Result<(), String> {
    if prefixes.is_empty() {
        return Err("snapshot allowlist is empty (set BASTION_FS_SNAPSHOT_ALLOWLIST)".to_string());
    }

    for prefix in prefixes {
        if root.starts_with(prefix) {
            return Ok(());
        }
    }
    Err("snapshot root is not in allowlist".to_string())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{
        SnapshotAttempt, SnapshotSettings, attempt_source_snapshot, attempt_source_snapshot_inner,
    };

    #[test]
    fn attempt_source_snapshot_rejects_unsupported_provider_override() {
        let tmp = tempdir().expect("tempdir");
        let root = tmp.path().join("root");
        std::fs::create_dir_all(&root).expect("create root");

        let attempt = attempt_source_snapshot(&root, tmp.path(), Some("nope"));
        match attempt {
            SnapshotAttempt::Unavailable(unavail) => {
                assert_eq!(unavail.provider.as_deref(), Some("nope"));
                assert!(unavail.reason.contains("unsupported snapshot provider"));
            }
            SnapshotAttempt::Ready(_) => panic!("expected unavailable"),
        }
    }

    #[test]
    fn attempt_source_snapshot_reports_missing_allowlist_when_enabled() {
        let tmp = tempdir().expect("tempdir");
        let root = tmp.path().join("root");
        let settings = SnapshotSettings {
            btrfs_enabled: true,
            allowlist_prefixes: Vec::new(),
        };

        let attempt = attempt_source_snapshot_inner(&root, tmp.path(), None, &settings);
        match attempt {
            SnapshotAttempt::Unavailable(unavail) => {
                assert_eq!(unavail.provider.as_deref(), Some("btrfs"));
                assert!(unavail.reason.contains("allowlist is empty"));
            }
            SnapshotAttempt::Ready(_) => panic!("expected unavailable"),
        }
    }

    #[test]
    fn attempt_source_snapshot_reports_root_not_in_allowlist() {
        let tmp = tempdir().expect("tempdir");
        let root = tmp.path().join("root");

        let non_matching = tmp.path().join("other-prefix");
        let settings = SnapshotSettings {
            btrfs_enabled: true,
            allowlist_prefixes: vec![non_matching],
        };

        let attempt = attempt_source_snapshot_inner(&root, tmp.path(), None, &settings);
        match attempt {
            SnapshotAttempt::Unavailable(unavail) => {
                assert_eq!(unavail.provider.as_deref(), Some("btrfs"));
                assert!(unavail.reason.contains("not in allowlist"));
            }
            SnapshotAttempt::Ready(_) => panic!("expected unavailable"),
        }
    }
}
