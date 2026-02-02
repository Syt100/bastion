use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tempfile::tempdir;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::backup::{BuildPipelineOptions, PayloadEncryption};
use bastion_core::job_spec::{FilesystemSource, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy};
use bastion_core::manifest::ArtifactFormatV1;
use bastion_core::progress::ProgressUnitsV1;

use super::FilesystemBuildIssues;
use super::build_filesystem_run;
use super::scan::scan_filesystem_source;
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
        pre_scan: true,
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
        BuildPipelineOptions {
            artifact_format: ArtifactFormatV1::ArchiveV1,
            encryption: &PayloadEncryption::None,
            part_size_bytes: 4 * 1024 * 1024,
        },
        None,
        None,
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
fn filesystem_paths_can_build_raw_tree_single_file() {
    let tmp = tempdir().expect("tempdir");
    let data_dir = tmp.path().join("data");
    std::fs::create_dir_all(&data_dir).unwrap();

    let src = tmp.path().join("hello.txt");
    std::fs::write(&src, b"hi").unwrap();

    let expected = archive_prefix_for_path(&src).unwrap();

    let source = FilesystemSource {
        pre_scan: true,
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
        BuildPipelineOptions {
            artifact_format: ArtifactFormatV1::RawTreeV1,
            encryption: &PayloadEncryption::None,
            part_size_bytes: 4 * 1024 * 1024,
        },
        None,
        None,
    )
    .unwrap();
    assert_eq!(build.issues.errors_total, 0);
    assert!(build.artifacts.parts.is_empty());

    let stage_dir = build
        .artifacts
        .manifest_path
        .parent()
        .expect("manifest parent");
    let mut dst = stage_dir.join("data");
    for seg in expected.split('/') {
        dst.push(seg);
    }
    assert_eq!(std::fs::read(&dst).unwrap(), b"hi");

    let index_paths = list_index_paths(&build.artifacts.entries_index_path);
    assert!(index_paths.contains(&expected));

    let manifest_bytes = std::fs::read(&build.artifacts.manifest_path).unwrap();
    let manifest: bastion_core::manifest::ManifestV1 =
        serde_json::from_slice(&manifest_bytes).unwrap();
    assert_eq!(manifest.pipeline.format, ArtifactFormatV1::RawTreeV1);
    assert!(manifest.artifacts.is_empty());
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
        pre_scan: true,
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
        BuildPipelineOptions {
            artifact_format: ArtifactFormatV1::ArchiveV1,
            encryption: &PayloadEncryption::None,
            part_size_bytes: 4 * 1024 * 1024,
        },
        None,
        None,
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
        pre_scan: true,
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
        BuildPipelineOptions {
            artifact_format: ArtifactFormatV1::ArchiveV1,
            encryption: &PayloadEncryption::None,
            part_size_bytes: 4 * 1024 * 1024,
        },
        None,
        None,
    )
    .unwrap();
    assert_eq!(build.issues.errors_total, 0);

    let part = build.artifacts.parts[0].path.as_path();
    let tar_paths = list_tar_paths(part);
    assert!(tar_paths.contains(&"hello.txt".to_string()));
}

#[test]
fn archive_parts_can_be_deleted_during_packaging() {
    let tmp = tempdir().expect("tempdir");
    let data_dir = tmp.path().join("data");
    std::fs::create_dir_all(&data_dir).unwrap();

    // Use pseudo-random bytes so tar+zstd output stays large enough to rotate parts.
    let src = tmp.path().join("blob.bin");
    let mut buf = vec![0_u8; 256 * 1024];
    let mut x: u32 = 0x1234_5678;
    for b in buf.iter_mut() {
        // xorshift32
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *b = (x & 0xff) as u8;
    }
    std::fs::write(&src, &buf).unwrap();

    let source = FilesystemSource {
        pre_scan: true,
        paths: vec![src.to_string_lossy().to_string()],
        root: String::new(),
        include: Vec::new(),
        exclude: Vec::new(),
        symlink_policy: FsSymlinkPolicy::Keep,
        hardlink_policy: FsHardlinkPolicy::Copy,
        error_policy: FsErrorPolicy::FailFast,
    };

    let parts_seen = Arc::new(AtomicUsize::new(0));
    let parts_seen_cb = parts_seen.clone();
    let on_part_finished = Box::new(
        move |part: crate::backup::LocalArtifact| -> std::io::Result<()> {
            parts_seen_cb.fetch_add(1, Ordering::SeqCst);

            let stage_dir = part
                .path
                .parent()
                .ok_or_else(|| std::io::Error::other("part path has no parent"))?;

            // At the moment a part is finalized, we should only have that single part on disk (rolling
            // upload deletes parts immediately).
            let part_files = std::fs::read_dir(stage_dir)?
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|name| name.starts_with("payload.part"))
                .count();
            assert_eq!(part_files, 1);

            std::fs::remove_file(&part.path)?;
            Ok(())
        },
    );

    let build = build_filesystem_run(
        &data_dir,
        &Uuid::new_v4().to_string(),
        &Uuid::new_v4().to_string(),
        OffsetDateTime::now_utc(),
        &source,
        BuildPipelineOptions {
            artifact_format: ArtifactFormatV1::ArchiveV1,
            encryption: &PayloadEncryption::None,
            // Force many part rotations so the callback is exercised.
            part_size_bytes: 64,
        },
        None,
        Some(on_part_finished),
    )
    .unwrap();
    assert_eq!(build.issues.errors_total, 0);

    assert!(
        parts_seen.load(Ordering::SeqCst) > 1,
        "expected multiple part rotations"
    );

    // The build should succeed even though part files were deleted during packaging.
    let stage_dir = build
        .artifacts
        .manifest_path
        .parent()
        .expect("manifest parent");
    let remaining_parts = std::fs::read_dir(stage_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name.starts_with("payload.part"))
        .count();
    assert_eq!(remaining_parts, 0);
}

#[test]
fn scan_legacy_root_respects_include_patterns_for_files() -> Result<(), anyhow::Error> {
    let tmp = tempdir().expect("tempdir");
    let root = tmp.path().join("root");
    std::fs::create_dir_all(root.join("sub")).unwrap();

    std::fs::write(root.join("a.txt"), b"ab").unwrap();
    std::fs::write(root.join("b.log"), b"x").unwrap();
    std::fs::write(root.join("sub").join("c.txt"), b"cde").unwrap();

    let source = FilesystemSource {
        pre_scan: true,
        paths: Vec::new(),
        root: root.to_string_lossy().to_string(),
        include: vec!["a.txt".to_string()],
        exclude: Vec::new(),
        symlink_policy: FsSymlinkPolicy::Keep,
        hardlink_policy: FsHardlinkPolicy::Copy,
        error_policy: FsErrorPolicy::FailFast,
    };

    let mut issues = FilesystemBuildIssues::default();
    let totals = scan_filesystem_source(&source, &mut issues, None)?;
    assert_eq!(issues.errors_total, 0);

    // The directory entry is still counted even though its contents are filtered by include globs.
    assert_eq!(
        totals,
        ProgressUnitsV1 {
            dirs: 1,
            files: 1,
            bytes: 2,
        }
    );
    Ok(())
}

#[test]
fn scan_legacy_root_excludes_directory_and_skips_descendants() -> Result<(), anyhow::Error> {
    let tmp = tempdir().expect("tempdir");
    let root = tmp.path().join("root");
    std::fs::create_dir_all(root.join("sub")).unwrap();

    std::fs::write(root.join("a.txt"), b"ab").unwrap();
    std::fs::write(root.join("b.log"), b"x").unwrap();
    std::fs::write(root.join("sub").join("c.txt"), b"cde").unwrap();

    let source = FilesystemSource {
        pre_scan: true,
        paths: Vec::new(),
        root: root.to_string_lossy().to_string(),
        include: Vec::new(),
        exclude: vec!["sub".to_string()],
        symlink_policy: FsSymlinkPolicy::Keep,
        hardlink_policy: FsHardlinkPolicy::Copy,
        error_policy: FsErrorPolicy::FailFast,
    };

    let mut issues = FilesystemBuildIssues::default();
    let totals = scan_filesystem_source(&source, &mut issues, None)?;
    assert_eq!(issues.errors_total, 0);

    // Excluding a directory should skip the directory entry and everything under it.
    assert_eq!(
        totals,
        ProgressUnitsV1 {
            dirs: 0,
            files: 2,
            bytes: 3,
        }
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn scan_legacy_root_symlink_policy_skip_ignores_symlink_entries() -> Result<(), anyhow::Error> {
    use std::os::unix::fs as unix_fs;

    let tmp = tempdir().expect("tempdir");
    let root = tmp.path().join("root");
    std::fs::create_dir_all(&root).unwrap();

    std::fs::write(root.join("real.txt"), b"ab").unwrap();
    unix_fs::symlink(root.join("real.txt"), root.join("link.txt")).unwrap();

    let source_keep = FilesystemSource {
        pre_scan: true,
        paths: Vec::new(),
        root: root.to_string_lossy().to_string(),
        include: Vec::new(),
        exclude: Vec::new(),
        symlink_policy: FsSymlinkPolicy::Keep,
        hardlink_policy: FsHardlinkPolicy::Copy,
        error_policy: FsErrorPolicy::FailFast,
    };
    let mut issues = FilesystemBuildIssues::default();
    let totals_keep = scan_filesystem_source(&source_keep, &mut issues, None)?;
    assert_eq!(issues.errors_total, 0);
    assert_eq!(
        totals_keep,
        ProgressUnitsV1 {
            dirs: 0,
            files: 2,
            bytes: 2,
        }
    );

    let source_skip = FilesystemSource {
        symlink_policy: FsSymlinkPolicy::Skip,
        ..source_keep
    };
    let mut issues = FilesystemBuildIssues::default();
    let totals_skip = scan_filesystem_source(&source_skip, &mut issues, None)?;
    assert_eq!(issues.errors_total, 0);
    assert_eq!(
        totals_skip,
        ProgressUnitsV1 {
            dirs: 0,
            files: 1,
            bytes: 2,
        }
    );

    Ok(())
}
