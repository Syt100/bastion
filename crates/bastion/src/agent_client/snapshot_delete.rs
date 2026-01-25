#[derive(Debug)]
pub(super) enum SnapshotDeleteResult {
    Deleted,
    NotFound,
    Failed {
        kind: &'static str,
        error: anyhow::Error,
    },
}

pub(super) fn delete_local_snapshot_dir(
    base_dir: &str,
    job_id: &str,
    run_id: &str,
) -> SnapshotDeleteResult {
    use bastion_backup::{COMPLETE_NAME, ENTRIES_INDEX_NAME, MANIFEST_NAME};

    let run_dir = std::path::Path::new(base_dir).join(job_id).join(run_id);
    if !run_dir.exists() {
        return SnapshotDeleteResult::NotFound;
    }

    let mut looks_like_bastion = false;
    if run_dir.join(COMPLETE_NAME).exists()
        || run_dir.join(MANIFEST_NAME).exists()
        || run_dir.join(ENTRIES_INDEX_NAME).exists()
    {
        looks_like_bastion = true;
    } else if let Ok(entries) = std::fs::read_dir(&run_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("payload.part") || name.ends_with(".partial") {
                looks_like_bastion = true;
                break;
            }
        }
    }
    if !looks_like_bastion {
        return SnapshotDeleteResult::Failed {
            kind: "config",
            error: anyhow::anyhow!("local snapshot dir did not look like bastion data"),
        };
    }

    match std::fs::remove_dir_all(&run_dir) {
        Ok(()) => SnapshotDeleteResult::Deleted,
        Err(error) => SnapshotDeleteResult::Failed {
            kind: "unknown",
            error: anyhow::Error::from(error),
        },
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::{SnapshotDeleteResult, delete_local_snapshot_dir};

    #[test]
    fn delete_is_idempotent_when_dir_missing() {
        let tmp = TempDir::new().expect("tmp");
        let res = delete_local_snapshot_dir(tmp.path().to_str().unwrap(), "job", "run");
        assert!(matches!(res, SnapshotDeleteResult::NotFound));
    }

    #[test]
    fn delete_requires_bastion_markers() {
        let tmp = TempDir::new().expect("tmp");
        let base = tmp.path();
        let job_id = "job";
        let run_id = "run";

        let dir = base.join(job_id).join(run_id);
        std::fs::create_dir_all(&dir).expect("mkdir");

        match delete_local_snapshot_dir(base.to_str().unwrap(), job_id, run_id) {
            SnapshotDeleteResult::Failed { kind, .. } => assert_eq!(kind, "config"),
            other => panic!("unexpected result: {other:?}"),
        }

        std::fs::write(dir.join(bastion_backup::COMPLETE_NAME), b"{}").expect("write");
        match delete_local_snapshot_dir(base.to_str().unwrap(), job_id, run_id) {
            SnapshotDeleteResult::Deleted => {}
            other => panic!("unexpected result: {other:?}"),
        }
        assert!(!dir.exists());
    }
}
