use std::fs::File;
use std::io::Write;
use std::path::Path;

use tempfile::tempdir;
use time::OffsetDateTime;
use uuid::Uuid;

use super::entries_index::{ListChildrenFromEntriesIndexOptions, list_children_from_entries_index};
use super::unpack::{PayloadDecryption, restore_from_parts, safe_join};
use super::{ConflictPolicy, RestoreSelection};
use crate::backup::{BuildPipelineOptions, PayloadEncryption};
use bastion_core::job_spec::{FilesystemSource, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy};
use bastion_core::manifest::ArtifactFormatV1;

#[test]
fn safe_join_rejects_parent() {
    let base = Path::new("/tmp");
    assert!(safe_join(base, Path::new("../etc")).is_none());
}

#[test]
fn restore_from_parts_extracts_tar_zstd() {
    // Build a small tar.zst split into one part.
    let tmp = tempdir().unwrap();
    let part = tmp.path().join("payload.part000001");

    let file = File::create(&part).unwrap();
    let mut encoder = zstd::Encoder::new(file, 3).unwrap();
    {
        let mut tar = tar::Builder::new(&mut encoder);
        let src = tmp.path().join("hello.txt");
        std::fs::write(&src, b"hi").unwrap();
        tar.append_path_with_name(&src, Path::new("hello.txt"))
            .unwrap();
        tar.finish().unwrap();
    }
    encoder.finish().unwrap();

    let dest = tmp.path().join("out");
    restore_from_parts(
        &[part],
        &dest,
        ConflictPolicy::Overwrite,
        PayloadDecryption::None,
        None,
    )
    .unwrap();
    let out = std::fs::read(dest.join("hello.txt")).unwrap();
    assert_eq!(out, b"hi");
}

#[test]
fn restore_from_parts_respects_selected_files() {
    let tmp = tempdir().unwrap();
    let part = tmp.path().join("payload.part000001");

    let file = File::create(&part).unwrap();
    let mut encoder = zstd::Encoder::new(file, 3).unwrap();
    {
        let mut tar = tar::Builder::new(&mut encoder);
        let a = tmp.path().join("a.txt");
        let b = tmp.path().join("b.txt");
        std::fs::write(&a, b"a").unwrap();
        std::fs::write(&b, b"b").unwrap();
        tar.append_path_with_name(&a, Path::new("a.txt")).unwrap();
        tar.append_path_with_name(&b, Path::new("b.txt")).unwrap();
        tar.finish().unwrap();
    }
    encoder.finish().unwrap();

    let dest = tmp.path().join("out_partial_file");
    let sel = RestoreSelection {
        files: vec!["a.txt".to_string()],
        dirs: vec![],
    };
    restore_from_parts(
        &[part],
        &dest,
        ConflictPolicy::Overwrite,
        PayloadDecryption::None,
        Some(&sel),
    )
    .unwrap();

    assert!(dest.join("a.txt").exists());
    assert!(!dest.join("b.txt").exists());
}

#[test]
fn restore_from_parts_respects_selected_dirs() {
    let tmp = tempdir().unwrap();
    let part = tmp.path().join("payload.part000001");

    let file = File::create(&part).unwrap();
    let mut encoder = zstd::Encoder::new(file, 3).unwrap();
    {
        let mut tar = tar::Builder::new(&mut encoder);
        let a = tmp.path().join("a.txt");
        let b = tmp.path().join("b.txt");
        let c = tmp.path().join("c.txt");
        std::fs::write(&a, b"a").unwrap();
        std::fs::write(&b, b"b").unwrap();
        std::fs::write(&c, b"c").unwrap();
        tar.append_path_with_name(&a, Path::new("dir/a.txt"))
            .unwrap();
        tar.append_path_with_name(&b, Path::new("dir/b.txt"))
            .unwrap();
        tar.append_path_with_name(&c, Path::new("c.txt")).unwrap();
        tar.finish().unwrap();
    }
    encoder.finish().unwrap();

    let dest = tmp.path().join("out_partial_dir");
    let sel = RestoreSelection {
        files: vec![],
        dirs: vec!["dir".to_string()],
    };
    restore_from_parts(
        &[part],
        &dest,
        ConflictPolicy::Overwrite,
        PayloadDecryption::None,
        Some(&sel),
    )
    .unwrap();

    assert!(dest.join("dir").join("a.txt").exists());
    assert!(dest.join("dir").join("b.txt").exists());
    assert!(!dest.join("c.txt").exists());
}

#[test]
fn restore_raw_tree_to_local_fs_copies_files() {
    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().join("data");
    std::fs::create_dir_all(&data_dir).unwrap();

    let src_root = tmp.path().join("src");
    std::fs::create_dir_all(&src_root).unwrap();
    std::fs::write(src_root.join("hello.txt"), b"hi").unwrap();

    let job_id = Uuid::new_v4().to_string();
    let run_id = Uuid::new_v4().to_string();

    let source = FilesystemSource {
        pre_scan: true,
        paths: Vec::new(),
        root: src_root.to_string_lossy().to_string(),
        include: Vec::new(),
        exclude: Vec::new(),
        symlink_policy: FsSymlinkPolicy::Keep,
        hardlink_policy: FsHardlinkPolicy::Copy,
        error_policy: FsErrorPolicy::FailFast,
    };

    let build = crate::backup::filesystem::build_filesystem_run(
        &data_dir,
        &job_id,
        &run_id,
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

    let target_base = tmp.path().join("target");
    std::fs::create_dir_all(&target_base).unwrap();
    let run_dir = bastion_targets::local_dir::store_run(
        &target_base,
        &job_id,
        &run_id,
        &build.artifacts,
        None,
    )
    .unwrap();

    let entries_index_path = run_dir.join(bastion_core::backup_format::ENTRIES_INDEX_NAME);
    let staging_dir = tmp.path().join("staging");
    std::fs::create_dir_all(&staging_dir).unwrap();

    let dest_dir = tmp.path().join("out_raw");
    let local_source = super::sources::LocalDirSource::new(run_dir);
    super::raw_tree::restore_raw_tree_to_local_fs(
        &local_source,
        &entries_index_path,
        &staging_dir,
        &dest_dir,
        ConflictPolicy::Overwrite,
        None,
        None,
    )
    .unwrap();

    let out = std::fs::read(dest_dir.join("hello.txt")).unwrap();
    assert_eq!(out, b"hi");
}

#[test]
fn restore_from_parts_conflict_skip_keeps_existing_files() {
    let tmp = tempdir().unwrap();
    let part = tmp.path().join("payload.part000001");

    let file = File::create(&part).unwrap();
    let mut encoder = zstd::Encoder::new(file, 3).unwrap();
    {
        let mut tar = tar::Builder::new(&mut encoder);
        let src = tmp.path().join("hello.txt");
        std::fs::write(&src, b"from-archive").unwrap();
        tar.append_path_with_name(&src, Path::new("hello.txt"))
            .unwrap();
        tar.finish().unwrap();
    }
    encoder.finish().unwrap();

    let dest = tmp.path().join("out_conflict_skip");
    std::fs::create_dir_all(&dest).unwrap();
    std::fs::write(dest.join("hello.txt"), b"existing").unwrap();

    restore_from_parts(
        &[part],
        &dest,
        ConflictPolicy::Skip,
        PayloadDecryption::None,
        None,
    )
    .unwrap();

    let out = std::fs::read(dest.join("hello.txt")).unwrap();
    assert_eq!(out, b"existing");
}

#[test]
fn restore_from_parts_conflict_fail_errors_on_existing_files() {
    let tmp = tempdir().unwrap();
    let part = tmp.path().join("payload.part000001");

    let file = File::create(&part).unwrap();
    let mut encoder = zstd::Encoder::new(file, 3).unwrap();
    {
        let mut tar = tar::Builder::new(&mut encoder);
        let src = tmp.path().join("hello.txt");
        std::fs::write(&src, b"from-archive").unwrap();
        tar.append_path_with_name(&src, Path::new("hello.txt"))
            .unwrap();
        tar.finish().unwrap();
    }
    encoder.finish().unwrap();

    let dest = tmp.path().join("out_conflict_fail");
    std::fs::create_dir_all(&dest).unwrap();
    std::fs::write(dest.join("hello.txt"), b"existing").unwrap();

    let err = restore_from_parts(
        &[part],
        &dest,
        ConflictPolicy::Fail,
        PayloadDecryption::None,
        None,
    )
    .unwrap_err();
    assert!(err.to_string().contains("restore conflict"));
}

#[test]
fn restore_from_parts_extracts_tar_zstd_age() {
    use age::secrecy::ExposeSecret as _;

    let tmp = tempdir().unwrap();
    let data_dir = tmp.path().join("data");
    std::fs::create_dir_all(&data_dir).unwrap();

    let src_root = tmp.path().join("src");
    std::fs::create_dir_all(&src_root).unwrap();
    std::fs::write(src_root.join("hello.txt"), b"hi").unwrap();

    let identity = age::x25519::Identity::generate();
    let recipient = identity.to_public().to_string();
    let identity_str = identity.to_string().expose_secret().to_string();

    let encryption = PayloadEncryption::AgeX25519 {
        recipient,
        key_name: "k".to_string(),
    };

    let job_id = Uuid::new_v4().to_string();
    let run_id = Uuid::new_v4().to_string();
    let source = FilesystemSource {
        pre_scan: true,
        paths: Vec::new(),
        root: src_root.to_string_lossy().to_string(),
        include: Vec::new(),
        exclude: Vec::new(),
        symlink_policy: FsSymlinkPolicy::Keep,
        hardlink_policy: FsHardlinkPolicy::Copy,
        error_policy: FsErrorPolicy::FailFast,
    };

    let build = crate::backup::filesystem::build_filesystem_run(
        &data_dir,
        &job_id,
        &run_id,
        OffsetDateTime::now_utc(),
        &source,
        BuildPipelineOptions {
            artifact_format: ArtifactFormatV1::ArchiveV1,
            encryption: &encryption,
            part_size_bytes: 4 * 1024 * 1024,
        },
        None,
        None,
    )
    .unwrap();
    assert_eq!(build.issues.errors_total, 0);

    let manifest_bytes = std::fs::read(&build.artifacts.manifest_path).unwrap();
    let manifest =
        serde_json::from_slice::<bastion_core::manifest::ManifestV1>(&manifest_bytes).unwrap();
    assert_eq!(manifest.pipeline.encryption, "age");
    assert_eq!(manifest.pipeline.encryption_key.as_deref(), Some("k"));

    let part_paths = build
        .artifacts
        .parts
        .iter()
        .map(|p| p.path.clone())
        .collect::<Vec<_>>();

    let dest = tmp.path().join("out_age");
    restore_from_parts(
        &part_paths,
        &dest,
        ConflictPolicy::Overwrite,
        PayloadDecryption::AgeX25519 {
            identity: identity_str,
        },
        None,
    )
    .unwrap();

    let out = std::fs::read(dest.join("hello.txt")).unwrap();
    assert_eq!(out, b"hi");
}

#[test]
fn entries_children_lists_unique_children() {
    #[derive(serde::Serialize)]
    struct Rec<'a> {
        path: &'a str,
        kind: &'a str,
        size: u64,
        hash_alg: Option<&'a str>,
        hash: Option<&'a str>,
    }

    let tmp = tempdir().unwrap();
    let entries_path = tmp.path().join("entries.jsonl.zst");

    let file = File::create(&entries_path).unwrap();
    let mut enc = zstd::Encoder::new(file, 3).unwrap();
    for rec in [
        Rec {
            path: ".env",
            kind: "file",
            size: 1,
            hash_alg: Some("blake3"),
            hash: Some("dot"),
        },
        Rec {
            path: "etc",
            kind: "dir",
            size: 0,
            hash_alg: None,
            hash: None,
        },
        Rec {
            path: "etc/hosts",
            kind: "file",
            size: 2,
            hash_alg: Some("blake3"),
            hash: Some("x"),
        },
        Rec {
            path: "etc/ssh/sshd_config",
            kind: "file",
            size: 3,
            hash_alg: Some("blake3"),
            hash: Some("y"),
        },
        Rec {
            path: "var/log/syslog",
            kind: "file",
            size: 4,
            hash_alg: Some("blake3"),
            hash: Some("z"),
        },
    ] {
        let line = serde_json::to_vec(&rec).unwrap();
        enc.write_all(&line).unwrap();
        enc.write_all(b"\n").unwrap();
    }
    enc.finish().unwrap();

    let root = list_children_from_entries_index(
        &entries_path,
        ListChildrenFromEntriesIndexOptions {
            prefix: "".to_string(),
            cursor: 0,
            limit: 100,
            q: None,
            kind: None,
            hide_dotfiles: false,
            min_size_bytes: None,
            max_size_bytes: None,
            type_sort_file_first: false,
        },
    )
    .unwrap();
    assert_eq!(root.prefix, "");
    assert!(
        root.entries
            .iter()
            .any(|e| e.path == ".env" && e.kind == "file")
    );
    assert!(
        root.entries
            .iter()
            .any(|e| e.path == "etc" && e.kind == "dir")
    );
    assert!(
        root.entries
            .iter()
            .any(|e| e.path == "var" && e.kind == "dir")
    );

    let etc = list_children_from_entries_index(
        &entries_path,
        ListChildrenFromEntriesIndexOptions {
            prefix: "etc".to_string(),
            cursor: 0,
            limit: 100,
            q: None,
            kind: None,
            hide_dotfiles: false,
            min_size_bytes: None,
            max_size_bytes: None,
            type_sort_file_first: false,
        },
    )
    .unwrap();
    assert!(
        etc.entries
            .iter()
            .any(|e| e.path == "etc/hosts" && e.kind == "file")
    );
    assert!(
        etc.entries
            .iter()
            .any(|e| e.path == "etc/ssh" && e.kind == "dir")
    );

    let ssh = list_children_from_entries_index(
        &entries_path,
        ListChildrenFromEntriesIndexOptions {
            prefix: "etc/ssh".to_string(),
            cursor: 0,
            limit: 100,
            q: None,
            kind: None,
            hide_dotfiles: false,
            min_size_bytes: None,
            max_size_bytes: None,
            type_sort_file_first: false,
        },
    )
    .unwrap();
    assert!(
        ssh.entries
            .iter()
            .any(|e| e.path == "etc/ssh/sshd_config" && e.kind == "file")
    );

    let etc_files = list_children_from_entries_index(
        &entries_path,
        ListChildrenFromEntriesIndexOptions {
            prefix: "etc".to_string(),
            cursor: 0,
            limit: 100,
            q: None,
            kind: Some("file".to_string()),
            hide_dotfiles: false,
            min_size_bytes: None,
            max_size_bytes: None,
            type_sort_file_first: false,
        },
    )
    .unwrap();
    assert_eq!(
        etc_files.entries.iter().filter(|e| e.kind == "dir").count(),
        0
    );
    assert!(etc_files.entries.iter().any(|e| e.path == "etc/hosts"));

    let etc_search = list_children_from_entries_index(
        &entries_path,
        ListChildrenFromEntriesIndexOptions {
            prefix: "etc".to_string(),
            cursor: 0,
            limit: 100,
            q: Some("SSH".to_string()),
            kind: None,
            hide_dotfiles: false,
            min_size_bytes: None,
            max_size_bytes: None,
            type_sort_file_first: false,
        },
    )
    .unwrap();
    assert_eq!(etc_search.entries.len(), 1);
    assert_eq!(etc_search.entries[0].path, "etc/ssh");

    let root_hide = list_children_from_entries_index(
        &entries_path,
        ListChildrenFromEntriesIndexOptions {
            prefix: "".to_string(),
            cursor: 0,
            limit: 100,
            q: None,
            kind: None,
            hide_dotfiles: true,
            min_size_bytes: None,
            max_size_bytes: None,
            type_sort_file_first: false,
        },
    )
    .unwrap();
    assert!(!root_hide.entries.iter().any(|e| e.path == ".env"));

    let root_min_size = list_children_from_entries_index(
        &entries_path,
        ListChildrenFromEntriesIndexOptions {
            prefix: "".to_string(),
            cursor: 0,
            limit: 100,
            q: None,
            kind: None,
            hide_dotfiles: false,
            min_size_bytes: Some(2),
            max_size_bytes: None,
            type_sort_file_first: false,
        },
    )
    .unwrap();
    assert!(
        root_min_size
            .entries
            .iter()
            .any(|e| e.path == "etc" && e.kind == "dir")
    );
    assert!(!root_min_size.entries.iter().any(|e| e.path == ".env"));

    let root_file_first = list_children_from_entries_index(
        &entries_path,
        ListChildrenFromEntriesIndexOptions {
            prefix: "".to_string(),
            cursor: 0,
            limit: 100,
            q: None,
            kind: None,
            hide_dotfiles: false,
            min_size_bytes: None,
            max_size_bytes: None,
            type_sort_file_first: true,
        },
    )
    .unwrap();
    assert_eq!(root_file_first.entries[0].path, ".env");
}
