use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::Path;

use bastion_core::manifest::{EntryIndexRef, ManifestV1, PipelineSettings};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tracing::info;
use uuid::Uuid;

use crate::backup::{
    COMPLETE_NAME, ENTRIES_INDEX_NAME, LocalRunArtifacts, MANIFEST_NAME, PayloadEncryption,
    stage_dir,
};
use bastion_core::job_spec::FilesystemSource;

mod entries_index;
mod tar;
mod util;

const MAX_FS_ISSUE_SAMPLES: usize = 50;

#[derive(Debug, Default)]
pub struct FilesystemBuildIssues {
    pub warnings_total: u64,
    pub errors_total: u64,
    pub sample_warnings: Vec<String>,
    pub sample_errors: Vec<String>,
}

impl FilesystemBuildIssues {
    fn record_warning(&mut self, msg: impl Into<String>) {
        self.warnings_total = self.warnings_total.saturating_add(1);
        if self.sample_warnings.len() < MAX_FS_ISSUE_SAMPLES {
            self.sample_warnings.push(msg.into());
        }
    }

    fn record_error(&mut self, msg: impl Into<String>) {
        self.errors_total = self.errors_total.saturating_add(1);
        if self.sample_errors.len() < MAX_FS_ISSUE_SAMPLES {
            self.sample_errors.push(msg.into());
        }
    }
}

#[derive(Debug)]
pub struct FilesystemRunBuild {
    pub artifacts: LocalRunArtifacts,
    pub issues: FilesystemBuildIssues,
}

pub fn build_filesystem_run(
    data_dir: &Path,
    job_id: &str,
    run_id: &str,
    started_at: OffsetDateTime,
    source: &FilesystemSource,
    encryption: &PayloadEncryption,
    part_size_bytes: u64,
) -> Result<FilesystemRunBuild, anyhow::Error> {
    let using_paths = source.paths.iter().any(|p| !p.trim().is_empty());
    info!(
        job_id = %job_id,
        run_id = %run_id,
        using_paths,
        paths_count = source.paths.len(),
        root = %source.root,
        include_rules = source.include.len(),
        exclude_rules = source.exclude.len(),
        symlink_policy = ?source.symlink_policy,
        hardlink_policy = ?source.hardlink_policy,
        error_policy = ?source.error_policy,
        encryption = ?encryption,
        part_size_bytes,
        "building filesystem backup artifacts"
    );

    let stage = stage_dir(data_dir, run_id);
    std::fs::create_dir_all(&stage)?;

    let entries_path = stage.join(ENTRIES_INDEX_NAME);
    let entries_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(entries_path)?;
    let entries_writer = BufWriter::new(entries_file);
    let mut entries_writer = zstd::Encoder::new(entries_writer, 3)?;
    let mut entries_count = 0u64;
    let mut issues = FilesystemBuildIssues::default();

    let parts = tar::write_tar_zstd_parts(
        &stage,
        source,
        encryption,
        &mut entries_writer,
        &mut entries_count,
        part_size_bytes,
        &mut issues,
    )?;
    entries_writer.finish()?;

    let ended_at = OffsetDateTime::now_utc();

    let job_uuid = Uuid::parse_str(job_id)?;
    let run_uuid = Uuid::parse_str(run_id)?;

    let manifest = ManifestV1 {
        format_version: ManifestV1::FORMAT_VERSION,
        job_id: job_uuid,
        run_id: run_uuid,
        started_at: started_at.format(&Rfc3339)?,
        ended_at: ended_at.format(&Rfc3339)?,
        pipeline: PipelineSettings {
            tar: "pax".to_string(),
            compression: "zstd".to_string(),
            encryption: match encryption {
                PayloadEncryption::None => "none".to_string(),
                PayloadEncryption::AgeX25519 { .. } => "age".to_string(),
            },
            encryption_key: match encryption {
                PayloadEncryption::None => None,
                PayloadEncryption::AgeX25519 { key_name, .. } => Some(key_name.clone()),
            },
            split_bytes: part_size_bytes,
        },
        artifacts: parts
            .iter()
            .map(|p| bastion_core::manifest::ArtifactPart {
                name: p.name.clone(),
                size: p.size,
                hash_alg: p.hash_alg.clone(),
                hash: p.hash.clone(),
            })
            .collect(),
        entry_index: EntryIndexRef {
            name: ENTRIES_INDEX_NAME.to_string(),
            count: entries_count,
        },
    };

    let manifest_path = stage.join(MANIFEST_NAME);
    let complete_path = stage.join(COMPLETE_NAME);

    util::write_json(&manifest_path, &manifest)?;
    util::write_json(&complete_path, &serde_json::json!({}))?;

    let parts_count = parts.len();
    let parts_bytes: u64 = parts.iter().map(|p| p.size).sum();
    info!(
        job_id = %job_id,
        run_id = %run_id,
        entries_count,
        parts_count,
        parts_bytes,
        warnings_total = issues.warnings_total,
        errors_total = issues.errors_total,
        "built filesystem backup artifacts"
    );

    Ok(FilesystemRunBuild {
        artifacts: LocalRunArtifacts {
            run_dir: stage.parent().unwrap_or(&stage).to_path_buf(),
            parts,
            entries_index_path: stage.join(ENTRIES_INDEX_NAME),
            entries_count,
            manifest_path,
            complete_path,
        },
        issues,
    })
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::Path;

    use tempfile::tempdir;
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::backup::PayloadEncryption;
    use bastion_core::job_spec::{
        FilesystemSource, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy,
    };

    use super::build_filesystem_run;
    use super::util::archive_prefix_for_path;

    fn list_tar_paths(part_path: &Path) -> Vec<String> {
        let file = File::open(part_path).expect("open part");
        let decoder = zstd::Decoder::new(file).expect("zstd decoder");
        let mut archive = ::tar::Archive::new(decoder);
        archive
            .entries()
            .expect("entries")
            .map(|e| {
                e.expect("entry")
                    .path()
                    .expect("path")
                    .to_string_lossy()
                    .to_string()
            })
            .collect()
    }

    fn list_index_paths(entries_path: &Path) -> Vec<String> {
        let raw = std::fs::read(entries_path).expect("read entries index");
        let decoded = zstd::decode_all(std::io::Cursor::new(raw)).expect("decode entries index");
        decoded
            .split(|b| *b == b'\n')
            .filter(|line| !line.is_empty())
            .filter_map(|line| serde_json::from_slice::<serde_json::Value>(line).ok())
            .filter_map(|v| {
                v.get("path")
                    .and_then(|p| p.as_str())
                    .map(|s| s.to_string())
            })
            .collect()
    }

    #[test]
    fn filesystem_paths_can_backup_single_file() {
        let tmp = tempdir().expect("tempdir");
        let data_dir = tmp.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let src = tmp.path().join("hello.txt");
        std::fs::write(&src, b"hi").unwrap();

        let expected = archive_prefix_for_path(&src).unwrap();

        let source = FilesystemSource {
            paths: vec![src.to_string_lossy().to_string()],
            root: String::new(),
            include: Vec::new(),
            exclude: Vec::new(),
            symlink_policy: FsSymlinkPolicy::Keep,
            hardlink_policy: FsHardlinkPolicy::Copy,
            error_policy: FsErrorPolicy::FailFast,
        };

        let build = build_filesystem_run(
            &data_dir,
            &Uuid::new_v4().to_string(),
            &Uuid::new_v4().to_string(),
            OffsetDateTime::now_utc(),
            &source,
            &PayloadEncryption::None,
            4 * 1024 * 1024,
        )
        .unwrap();
        assert_eq!(build.issues.errors_total, 0);

        let part_paths = build
            .artifacts
            .parts
            .iter()
            .map(|p| p.path.as_path())
            .collect::<Vec<_>>();
        assert_eq!(part_paths.len(), 1);
        let tar_paths = list_tar_paths(part_paths[0]);
        assert!(tar_paths.contains(&expected));

        let index_paths = list_index_paths(&build.artifacts.entries_index_path);
        assert!(index_paths.contains(&expected));
    }

    #[test]
    fn filesystem_paths_deduplicates_overlapping_sources() {
        let tmp = tempdir().expect("tempdir");
        let data_dir = tmp.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let dir = tmp.path().join("dir");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.txt");
        std::fs::write(&file, b"a").unwrap();

        let expected = format!("{}/a.txt", archive_prefix_for_path(&dir).unwrap());

        let source = FilesystemSource {
            paths: vec![
                dir.to_string_lossy().to_string(),
                file.to_string_lossy().to_string(),
            ],
            root: String::new(),
            include: Vec::new(),
            exclude: Vec::new(),
            symlink_policy: FsSymlinkPolicy::Keep,
            hardlink_policy: FsHardlinkPolicy::Copy,
            error_policy: FsErrorPolicy::FailFast,
        };

        let build = build_filesystem_run(
            &data_dir,
            &Uuid::new_v4().to_string(),
            &Uuid::new_v4().to_string(),
            OffsetDateTime::now_utc(),
            &source,
            &PayloadEncryption::None,
            4 * 1024 * 1024,
        )
        .unwrap();
        assert_eq!(build.issues.errors_total, 0);
        assert_eq!(build.issues.warnings_total, 1);
        assert!(
            build
                .issues
                .sample_warnings
                .iter()
                .any(|w| w.contains("deduplicated") && w.contains("overlapping source")),
            "missing dedupe warning: {:?}",
            build.issues.sample_warnings
        );

        let part = build.artifacts.parts[0].path.as_path();
        let tar_paths = list_tar_paths(part);
        assert_eq!(tar_paths.iter().filter(|p| *p == &expected).count(), 1);
    }

    #[test]
    fn legacy_root_can_backup_single_file() {
        let tmp = tempdir().expect("tempdir");
        let data_dir = tmp.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let src = tmp.path().join("hello.txt");
        std::fs::write(&src, b"hi").unwrap();

        let source = FilesystemSource {
            paths: Vec::new(),
            root: src.to_string_lossy().to_string(),
            include: Vec::new(),
            exclude: Vec::new(),
            symlink_policy: FsSymlinkPolicy::Keep,
            hardlink_policy: FsHardlinkPolicy::Copy,
            error_policy: FsErrorPolicy::FailFast,
        };

        let build = build_filesystem_run(
            &data_dir,
            &Uuid::new_v4().to_string(),
            &Uuid::new_v4().to_string(),
            OffsetDateTime::now_utc(),
            &source,
            &PayloadEncryption::None,
            4 * 1024 * 1024,
        )
        .unwrap();
        assert_eq!(build.issues.errors_total, 0);

        let part = build.artifacts.parts[0].path.as_path();
        let tar_paths = list_tar_paths(part);
        assert!(tar_paths.contains(&"hello.txt".to_string()));
    }
}
