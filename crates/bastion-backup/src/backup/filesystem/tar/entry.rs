use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;

use bastion_core::job_spec::{FilesystemSource, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy};
use bastion_core::manifest::HashAlgorithm;

use super::super::FilesystemBuildIssues;
use super::super::entries_index::{EntriesIndexWriter, EntryRecord, write_entry_record};
use super::super::util::hash_file;

pub(super) fn source_meta_for_policy(
    path: &Path,
    policy: FsSymlinkPolicy,
) -> Result<std::fs::Metadata, std::io::Error> {
    if policy == FsSymlinkPolicy::Follow {
        std::fs::metadata(path)
    } else {
        std::fs::symlink_metadata(path)
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_file_entry<W: Write>(
    tar: &mut ::tar::Builder<W>,
    fs_path: &Path,
    archive_path: &str,
    meta: &std::fs::Metadata,
    is_symlink_path: bool,
    source: &FilesystemSource,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    hardlink_index: &mut HashMap<FileId, HardlinkRecord>,
    seen_archive_paths: &mut HashSet<String>,
    progress: Option<&mut super::super::FilesystemBuildProgressCtx<'_>>,
) -> Result<(), anyhow::Error> {
    if seen_archive_paths.contains(archive_path) {
        issues.record_warning(format!("duplicate archive path (file): {archive_path}"));
        return Ok(());
    }

    let size = meta.len();
    if source.hardlink_policy == FsHardlinkPolicy::Keep
        && !is_symlink_path
        && hardlink_candidate(meta)
        && let Some(id) = file_id_for_meta(meta)
    {
        if let Some(existing) = hardlink_index.get(&id) {
            let mut header = ::tar::Header::new_gnu();
            header.set_metadata_in_mode(meta, ::tar::HeaderMode::Complete);
            header.set_entry_type(::tar::EntryType::hard_link());
            header.set_size(0);

            if let Err(error) = tar.append_link(
                &mut header,
                Path::new(archive_path),
                Path::new(&existing.first_path),
            ) {
                let msg = format!("archive error (hardlink): {archive_path}: {error}");
                if source.error_policy == FsErrorPolicy::FailFast {
                    return Err(anyhow::anyhow!(msg));
                }
                issues.record_error(msg);
                return Ok(());
            }

            seen_archive_paths.insert(archive_path.to_string());
            write_entry_record(
                entries_writer,
                entries_count,
                EntryRecord {
                    path: archive_path.to_string(),
                    kind: "file".to_string(),
                    size: existing.size,
                    hash_alg: Some(HashAlgorithm::Blake3),
                    hash: Some(existing.hash.clone()),
                    mtime: None,
                    mode: None,
                    uid: None,
                    gid: None,
                    xattrs: None,
                    symlink_target: None,
                    hardlink_group: None,
                },
                progress,
            )?;
            return Ok(());
        }

        let hash = match hash_file(fs_path) {
            Ok(h) => h,
            Err(error) => {
                let msg = format!("hash error: {archive_path}: {error}");
                if source.error_policy == FsErrorPolicy::FailFast {
                    return Err(anyhow::anyhow!(msg));
                }
                issues.record_error(msg);
                return Ok(());
            }
        };
        hardlink_index.insert(
            id,
            HardlinkRecord {
                first_path: archive_path.to_string(),
                size,
                hash: hash.clone(),
            },
        );

        if let Err(error) = tar.append_path_with_name(fs_path, Path::new(archive_path)) {
            let msg = format!("archive error: {archive_path}: {error}");
            if source.error_policy == FsErrorPolicy::FailFast {
                return Err(anyhow::anyhow!(msg));
            }
            issues.record_error(msg);
            return Ok(());
        }

        seen_archive_paths.insert(archive_path.to_string());
        write_entry_record(
            entries_writer,
            entries_count,
            EntryRecord {
                path: archive_path.to_string(),
                kind: "file".to_string(),
                size,
                hash_alg: Some(HashAlgorithm::Blake3),
                hash: Some(hash),
                mtime: None,
                mode: None,
                uid: None,
                gid: None,
                xattrs: None,
                symlink_target: None,
                hardlink_group: None,
            },
            progress,
        )?;
        return Ok(());
    }

    let hash = match hash_file(fs_path) {
        Ok(h) => h,
        Err(error) => {
            let msg = format!("hash error: {archive_path}: {error}");
            if source.error_policy == FsErrorPolicy::FailFast {
                return Err(anyhow::anyhow!(msg));
            }
            issues.record_error(msg);
            return Ok(());
        }
    };

    if let Err(error) = tar.append_path_with_name(fs_path, Path::new(archive_path)) {
        let msg = format!("archive error: {archive_path}: {error}");
        if source.error_policy == FsErrorPolicy::FailFast {
            return Err(anyhow::anyhow!(msg));
        }
        issues.record_error(msg);
        return Ok(());
    }

    seen_archive_paths.insert(archive_path.to_string());
    write_entry_record(
        entries_writer,
        entries_count,
        EntryRecord {
            path: archive_path.to_string(),
            kind: "file".to_string(),
            size,
            hash_alg: Some(HashAlgorithm::Blake3),
            hash: Some(hash),
            mtime: None,
            mode: None,
            uid: None,
            gid: None,
            xattrs: None,
            symlink_target: None,
            hardlink_group: None,
        },
        progress,
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_dir_entry<W: Write>(
    tar: &mut ::tar::Builder<W>,
    fs_path: &Path,
    archive_path: &str,
    source: &FilesystemSource,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    seen_archive_paths: &mut HashSet<String>,
    progress: Option<&mut super::super::FilesystemBuildProgressCtx<'_>>,
) -> Result<(), anyhow::Error> {
    if seen_archive_paths.contains(archive_path) {
        issues.record_warning(format!("duplicate archive path (dir): {archive_path}"));
        return Ok(());
    }

    if let Err(error) = tar.append_path_with_name(fs_path, Path::new(archive_path)) {
        let msg = format!("archive error: {archive_path}: {error}");
        if source.error_policy == FsErrorPolicy::FailFast {
            return Err(anyhow::anyhow!(msg));
        }
        issues.record_error(msg);
        return Ok(());
    }

    seen_archive_paths.insert(archive_path.to_string());
    write_entry_record(
        entries_writer,
        entries_count,
        EntryRecord {
            path: archive_path.to_string(),
            kind: "dir".to_string(),
            size: 0,
            hash_alg: None,
            hash: None,
            mtime: None,
            mode: None,
            uid: None,
            gid: None,
            xattrs: None,
            symlink_target: None,
            hardlink_group: None,
        },
        progress,
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_symlink_entry<W: Write>(
    tar: &mut ::tar::Builder<W>,
    fs_path: &Path,
    archive_path: &str,
    source: &FilesystemSource,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    seen_archive_paths: &mut HashSet<String>,
    progress: Option<&mut super::super::FilesystemBuildProgressCtx<'_>>,
) -> Result<(), anyhow::Error> {
    if seen_archive_paths.contains(archive_path) {
        issues.record_warning(format!("duplicate archive path (symlink): {archive_path}"));
        return Ok(());
    }

    if let Err(error) = tar.append_path_with_name(fs_path, Path::new(archive_path)) {
        let msg = format!("archive error: {archive_path}: {error}");
        if source.error_policy == FsErrorPolicy::FailFast {
            return Err(anyhow::anyhow!(msg));
        }
        issues.record_error(msg);
        return Ok(());
    }

    seen_archive_paths.insert(archive_path.to_string());
    write_entry_record(
        entries_writer,
        entries_count,
        EntryRecord {
            path: archive_path.to_string(),
            kind: "symlink".to_string(),
            size: 0,
            hash_alg: None,
            hash: None,
            mtime: None,
            mode: None,
            uid: None,
            gid: None,
            xattrs: None,
            symlink_target: None,
            hardlink_group: None,
        },
        progress,
    )?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct FileId {
    dev: u64,
    ino: u64,
}

#[cfg(unix)]
fn file_id_for_meta(meta: &std::fs::Metadata) -> Option<FileId> {
    use std::os::unix::fs::MetadataExt as _;
    Some(FileId {
        dev: meta.dev(),
        ino: meta.ino(),
    })
}

#[cfg(not(unix))]
fn file_id_for_meta(_meta: &std::fs::Metadata) -> Option<FileId> {
    None
}

#[cfg(unix)]
fn hardlink_candidate(meta: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt as _;
    meta.nlink() > 1
}

#[cfg(not(unix))]
fn hardlink_candidate(_meta: &std::fs::Metadata) -> bool {
    false
}

#[derive(Debug, Clone)]
pub(super) struct HardlinkRecord {
    pub(super) first_path: String,
    pub(super) size: u64,
    pub(super) hash: String,
}
