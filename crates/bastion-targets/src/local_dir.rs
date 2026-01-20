use std::path::{Path, PathBuf};

use tracing::{debug, info};

use bastion_core::backup_format::{
    COMPLETE_NAME, ENTRIES_INDEX_NAME, LocalRunArtifacts, MANIFEST_NAME,
};
use bastion_core::manifest::{ArtifactFormatV1, ManifestV1};

pub fn store_run(
    base_dir: &Path,
    job_id: &str,
    run_id: &str,
    artifacts: &LocalRunArtifacts,
) -> Result<PathBuf, anyhow::Error> {
    let manifest_bytes = std::fs::read(&artifacts.manifest_path)?;
    let manifest: ManifestV1 = serde_json::from_slice(&manifest_bytes)?;
    let artifact_format = manifest.pipeline.format;

    let parts_count = artifacts.parts.len();
    let parts_bytes: u64 = artifacts.parts.iter().map(|p| p.size).sum();
    info!(
        job_id = %job_id,
        run_id = %run_id,
        base_dir = %base_dir.display(),
        artifact_format = ?artifact_format,
        parts_count,
        parts_bytes,
        "storing run to local dir target"
    );

    let run_dir = base_dir.join(job_id).join(run_id);
    std::fs::create_dir_all(&run_dir)?;

    for part in &artifacts.parts {
        let dst = run_dir.join(&part.name);
        copy_if_needed(&part.path, &dst, part.size)?;
    }

    let entries_size = std::fs::metadata(&artifacts.entries_index_path)?.len();
    copy_if_needed(
        &artifacts.entries_index_path,
        &run_dir.join(ENTRIES_INDEX_NAME),
        entries_size,
    )?;

    let manifest_size = std::fs::metadata(&artifacts.manifest_path)?.len();
    copy_if_needed(
        &artifacts.manifest_path,
        &run_dir.join(MANIFEST_NAME),
        manifest_size,
    )?;

    if artifact_format == ArtifactFormatV1::RawTreeV1 {
        let stage_dir = artifacts
            .manifest_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("invalid manifest path"))?;
        let src = stage_dir.join("data");
        let dst = run_dir.join("data");
        copy_dir_recursive_if_needed(&src, &dst)?;
    }

    // Completion marker must be written last.
    let complete_size = std::fs::metadata(&artifacts.complete_path)?.len();
    copy_if_needed(
        &artifacts.complete_path,
        &run_dir.join(COMPLETE_NAME),
        complete_size,
    )?;

    info!(
        job_id = %job_id,
        run_id = %run_id,
        run_dir = %run_dir.display(),
        "stored run to local dir target"
    );
    Ok(run_dir)
}

fn copy_if_needed(src: &Path, dst: &Path, expected_size: u64) -> Result<(), anyhow::Error> {
    if let Ok(meta) = std::fs::metadata(dst)
        && meta.len() == expected_size
    {
        debug!(
            src = %src.display(),
            dst = %dst.display(),
            expected_size,
            "skipping existing target file"
        );
        return Ok(());
    }

    let file_name = dst
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid destination file name"))?;
    let tmp = dst.with_file_name(format!("{file_name}.partial"));
    let _ = std::fs::remove_file(&tmp);

    debug!(
        src = %src.display(),
        dst = %dst.display(),
        expected_size,
        "copying file to local dir target"
    );
    std::fs::copy(src, &tmp)?;
    let actual_size = std::fs::metadata(&tmp)?.len();
    if actual_size != expected_size {
        anyhow::bail!(
            "copied file size mismatch for {}: expected {}, got {}",
            dst.display(),
            expected_size,
            actual_size
        );
    }

    let _ = std::fs::remove_file(dst);
    std::fs::rename(&tmp, dst)?;
    Ok(())
}

fn copy_dir_recursive_if_needed(src: &Path, dst: &Path) -> Result<(), anyhow::Error> {
    let meta = match std::fs::metadata(src) {
        Ok(m) => m,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(anyhow::Error::new(error)),
    };
    if !meta.is_dir() {
        anyhow::bail!("raw-tree data path is not a directory: {}", src.display());
    }

    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive_if_needed(&src_path, &dst_path)?;
            continue;
        }

        if ty.is_file() {
            let size = std::fs::metadata(&src_path)?.len();
            copy_if_needed(&src_path, &dst_path, size)?;
            continue;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use bastion_core::manifest::HashAlgorithm;
    use tempfile::tempdir;

    use bastion_core::backup_format::{LocalArtifact, LocalRunArtifacts};

    use super::store_run;

    #[test]
    fn store_run_copies_files_and_is_resumable() {
        let tmp = tempdir().unwrap();
        let stage = tmp.path().join("stage");
        std::fs::create_dir_all(&stage).unwrap();

        let part_path = stage.join("payload.part000001");
        std::fs::write(&part_path, b"hello").unwrap();

        let entries_path = stage.join("entries.jsonl.zst");
        std::fs::write(&entries_path, b"entries").unwrap();

        let manifest_path = stage.join("manifest.json");
        std::fs::write(
            &manifest_path,
            serde_json::to_vec(&serde_json::json!({
              "format_version": 1,
              "job_id": "00000000-0000-0000-0000-000000000000",
              "run_id": "00000000-0000-0000-0000-000000000000",
              "started_at": "2025-12-30T12:00:00Z",
              "ended_at": "2025-12-30T12:00:01Z",
              "pipeline": {
                "format": "archive_v1",
                "tar": "pax",
                "compression": "zstd",
                "encryption": "none",
                "split_bytes": 0
              },
              "artifacts": [],
              "entry_index": { "name": "entries.jsonl.zst", "count": 0 }
            }))
            .unwrap(),
        )
        .unwrap();

        let complete_path = stage.join("complete.json");
        std::fs::write(&complete_path, b"{}").unwrap();

        let artifacts = LocalRunArtifacts {
            run_dir: stage.clone(),
            parts: vec![LocalArtifact {
                name: "payload.part000001".to_string(),
                path: part_path.clone(),
                size: 5,
                hash_alg: HashAlgorithm::Blake3,
                hash: "deadbeef".to_string(),
            }],
            entries_index_path: entries_path.clone(),
            entries_count: 1,
            manifest_path: manifest_path.clone(),
            complete_path: complete_path.clone(),
        };

        let dest_base = tmp.path().join("dest");
        let run_dir = store_run(&dest_base, "job1", "run1", &artifacts).unwrap();

        assert_eq!(run_dir, dest_base.join("job1").join("run1"));
        assert!(run_dir.join("payload.part000001").exists());
        assert!(run_dir.join("entries.jsonl.zst").exists());
        assert!(run_dir.join("manifest.json").exists());
        assert!(run_dir.join("complete.json").exists());

        // Re-run should skip already-present files (no error).
        store_run(&dest_base, "job1", "run1", &artifacts).unwrap();
    }
}
