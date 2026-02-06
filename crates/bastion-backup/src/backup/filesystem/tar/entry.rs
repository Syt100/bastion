use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use bastion_core::job_spec::{FilesystemSource, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy};
use bastion_core::manifest::HashAlgorithm;

use crate::backup::hashing_reader::HashingReader;
use crate::backup::source_consistency::{
    SourceConsistencyTracker, detect_change_reason, fingerprint_for_meta,
};

use super::super::FilesystemBuildIssues;
use super::super::entries_index::{EntriesIndexWriter, EntryRecord, write_entry_record};

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
    is_symlink_path: bool,
    source: &FilesystemSource,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    consistency: &mut SourceConsistencyTracker,
    hardlink_index: &mut HashMap<FileId, HardlinkRecord>,
    seen_archive_paths: &mut HashSet<String>,
    progress: Option<&mut super::super::FilesystemBuildProgressCtx<'_>>,
) -> Result<(), anyhow::Error> {
    if seen_archive_paths.contains(archive_path) {
        issues.record_warning(format!("duplicate archive path (file): {archive_path}"));
        return Ok(());
    }

    let file = match File::open(fs_path) {
        Ok(f) => f,
        Err(error) => {
            let msg = format!("archive error: {archive_path}: {error}");
            if source.error_policy == FsErrorPolicy::FailFast {
                return Err(anyhow::anyhow!(msg));
            }
            issues.record_error(msg);
            return Ok(());
        }
    };
    let meta = match file.metadata() {
        Ok(m) => m,
        Err(error) => {
            let msg = format!("metadata error: {archive_path}: {error}");
            if source.error_policy == FsErrorPolicy::FailFast {
                return Err(anyhow::anyhow!(msg));
            }
            issues.record_error(msg);
            return Ok(());
        }
    };

    let size = meta.len();
    if source.hardlink_policy == FsHardlinkPolicy::Keep
        && !is_symlink_path
        && hardlink_candidate(&meta)
        && let Some(id) = file_id_for_meta(&meta)
        && let Some(existing) = hardlink_index.get(&id)
    {
        let mut header = ::tar::Header::new_gnu();
        header.set_metadata_in_mode(&meta, ::tar::HeaderMode::Complete);
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

    let before_fp = fingerprint_for_meta(&meta);

    #[cfg(all(test, unix))]
    super::super::test_hooks::run_after_file_open_hook(fs_path, archive_path);

    let mut reader = HashingReader::new(file);

    let mut header = ::tar::Header::new_gnu();
    header.set_metadata_in_mode(&meta, ::tar::HeaderMode::Complete);
    header.set_entry_type(::tar::EntryType::Regular);
    header.set_size(size);
    header.set_cksum();

    if let Err(error) = tar.append_data(&mut header, Path::new(archive_path), &mut reader) {
        let msg = format!("archive error: {archive_path}: {error}");
        if source.error_policy == FsErrorPolicy::FailFast {
            return Err(anyhow::anyhow!(msg));
        }
        issues.record_error(msg);
        let file = reader.into_inner();
        let after_handle_fp = file.metadata().ok().map(|m| fingerprint_for_meta(&m));
        let after_path_fp = source_meta_for_policy(fs_path, source.symlink_policy)
            .ok()
            .map(|m| fingerprint_for_meta(&m));
        consistency.record_read_error(
            archive_path,
            error.to_string(),
            Some(before_fp),
            after_handle_fp,
            after_path_fp,
        );
        return Ok(());
    }

    let hash = reader.finalize_hex();
    let file = reader.into_inner();
    let after_handle_fp = file.metadata().ok().map(|m| fingerprint_for_meta(&m));

    // Best-effort: if the file changes while we're reading it, record a warning for the run so
    // users can judge consistency risk (no snapshots).
    match source_meta_for_policy(fs_path, source.symlink_policy) {
        Ok(after_meta) => {
            let after_path_fp = fingerprint_for_meta(&after_meta);
            let replaced = before_fp.file_id.is_some()
                && after_path_fp.file_id.is_some()
                && before_fp.file_id != after_path_fp.file_id;

            if replaced {
                consistency.record_replaced(
                    archive_path,
                    Some(before_fp),
                    after_handle_fp,
                    Some(after_path_fp),
                );
            } else {
                let reason = after_handle_fp
                    .as_ref()
                    .and_then(|h| detect_change_reason(&before_fp, h))
                    .or_else(|| detect_change_reason(&before_fp, &after_path_fp));

                if let Some(reason) = reason {
                    consistency.record_changed(
                        archive_path,
                        reason,
                        Some(before_fp),
                        after_handle_fp,
                        Some(after_path_fp),
                    );
                }
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            consistency.record_deleted(archive_path, Some(before_fp), after_handle_fp);
        }
        Err(_) => {}
    }

    if source.hardlink_policy == FsHardlinkPolicy::Keep
        && !is_symlink_path
        && hardlink_candidate(&meta)
        && let Some(id) = file_id_for_meta(&meta)
    {
        hardlink_index.insert(
            id,
            HardlinkRecord {
                first_path: archive_path.to_string(),
                size,
                hash: hash.clone(),
            },
        );
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
