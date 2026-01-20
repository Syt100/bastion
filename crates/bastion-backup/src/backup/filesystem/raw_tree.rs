use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use base64::Engine as _;
use bastion_core::job_spec::{FilesystemSource, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy};
use bastion_core::manifest::HashAlgorithm;
use walkdir::WalkDir;

use super::FilesystemBuildIssues;
use super::entries_index::{EntriesIndexWriter, EntryRecord, write_entry_record};
use super::util::{archive_prefix_for_path, compile_globset, join_archive_path};

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct RawTreeBuildStats {
    pub(super) data_files: u64,
    pub(super) data_bytes: u64,
}

pub(super) fn write_raw_tree(
    stage_dir: &Path,
    source: &FilesystemSource,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
) -> Result<RawTreeBuildStats, anyhow::Error> {
    let data_dir = stage_dir.join("data");
    std::fs::create_dir_all(&data_dir)?;

    let exclude = compile_globset(&source.exclude)?;
    let include = compile_globset(&source.include)?;
    let has_includes = !source.include.is_empty();

    #[cfg(not(unix))]
    if source.hardlink_policy == FsHardlinkPolicy::Keep {
        issues.record_warning(
            "hardlink_policy=keep is not supported on this platform; storing as copies",
        );
    }

    let follow_links = source.symlink_policy == FsSymlinkPolicy::Follow;

    let mut stats = RawTreeBuildStats::default();
    let mut hardlink_index = HashMap::<FileId, String>::new();
    let mut seen_archive_paths = HashSet::<String>::new();

    let using_paths = source.paths.iter().any(|p| !p.trim().is_empty());
    if using_paths {
        let mut raw_paths = source
            .paths
            .iter()
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .map(PathBuf::from)
            .collect::<Vec<_>>();

        // Preserve input order while removing exact duplicates.
        let mut seen_inputs = HashSet::<String>::new();
        raw_paths.retain(|p| seen_inputs.insert(p.to_string_lossy().replace('\\', "/")));

        // Remove selections already covered by a previously selected directory (best-effort).
        let mut covered_dirs = Vec::<PathBuf>::new();
        let mut removed = Vec::<String>::new();
        for p in raw_paths {
            if covered_dirs.iter().any(|dir| p.strip_prefix(dir).is_ok()) {
                removed.push(p.to_string_lossy().to_string());
                continue;
            }

            if let Ok(meta) = source_meta_for_policy(&p, source.symlink_policy)
                && meta.is_dir()
            {
                covered_dirs.push(p.clone());
            }

            write_source_entry(
                &p,
                source,
                &exclude,
                &include,
                has_includes,
                follow_links,
                &data_dir,
                entries_writer,
                entries_count,
                issues,
                &mut stats,
                &mut hardlink_index,
                &mut seen_archive_paths,
            )?;
        }

        if !removed.is_empty() {
            let sample = removed
                .iter()
                .take(5)
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            issues.record_warning(format!(
                "deduplicated {} overlapping source path(s) (sample: {})",
                removed.len(),
                sample
            ));
        }
    } else {
        let root = PathBuf::from(source.root.trim());
        write_legacy_root(
            &root,
            source,
            &exclude,
            &include,
            has_includes,
            follow_links,
            &data_dir,
            entries_writer,
            entries_count,
            issues,
            &mut stats,
            &mut hardlink_index,
            &mut seen_archive_paths,
        )?;
    }

    Ok(stats)
}

#[allow(clippy::too_many_arguments)]
fn write_legacy_root(
    root: &Path,
    source: &FilesystemSource,
    exclude: &globset::GlobSet,
    include: &globset::GlobSet,
    has_includes: bool,
    follow_links: bool,
    data_dir: &Path,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    stats: &mut RawTreeBuildStats,
    hardlink_index: &mut HashMap<FileId, String>,
    seen_archive_paths: &mut HashSet<String>,
) -> Result<(), anyhow::Error> {
    if root.as_os_str().is_empty() {
        anyhow::bail!("filesystem.source.root is required");
    }

    let meta = match source_meta_for_policy(root, source.symlink_policy) {
        Ok(m) => m,
        Err(error) => {
            let msg = format!("metadata error: {}: {error}", root.display());
            if source.error_policy == FsErrorPolicy::FailFast {
                return Err(anyhow::anyhow!(msg));
            }
            issues.record_error(msg);
            return Ok(());
        }
    };

    if meta.is_file() || meta.file_type().is_symlink() {
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|n| !n.trim().is_empty())
            .unwrap_or("file");

        if exclude.is_match(name) {
            return Ok(());
        }
        if meta.file_type().is_symlink() && source.symlink_policy == FsSymlinkPolicy::Skip {
            let target = std::fs::read_link(root)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "<unknown>".to_string());
            issues.record_warning(format!("skipped symlink: {name} -> {target}"));
            return Ok(());
        }

        let is_symlink_path = std::fs::symlink_metadata(root)
            .ok()
            .is_some_and(|m| m.file_type().is_symlink());

        if meta.is_file() {
            if has_includes && !include.is_match(name) {
                return Ok(());
            }
            write_file_entry(
                root,
                name,
                &meta,
                is_symlink_path,
                source,
                data_dir,
                entries_writer,
                entries_count,
                issues,
                stats,
                hardlink_index,
                seen_archive_paths,
            )?;
        } else {
            write_symlink_entry(
                root,
                name,
                source,
                data_dir,
                entries_writer,
                entries_count,
                issues,
                seen_archive_paths,
            )?;
        }

        return Ok(());
    }

    if !meta.is_dir() {
        let msg = format!("unsupported root file type: {}", root.display());
        if source.error_policy == FsErrorPolicy::FailFast {
            return Err(anyhow::anyhow!(msg));
        }
        issues.record_error(msg);
        return Ok(());
    }

    let mut iter = WalkDir::new(root).follow_links(follow_links).into_iter();
    while let Some(next) = iter.next() {
        let entry = match next {
            Ok(e) => e,
            Err(error) => {
                let msg = if let Some(path) = error.path() {
                    format!("walk error: {}: {}", path.display(), error)
                } else {
                    format!("walk error: {error}")
                };
                if source.error_policy == FsErrorPolicy::FailFast {
                    return Err(anyhow::anyhow!(msg));
                }
                issues.record_error(msg);
                continue;
            }
        };
        if entry.path() == root {
            continue;
        }

        let rel = match entry.path().strip_prefix(root) {
            Ok(v) => v,
            Err(error) => {
                let msg = format!(
                    "path error: {} is not under root {}: {error}",
                    entry.path().display(),
                    root.display()
                );
                if source.error_policy == FsErrorPolicy::FailFast {
                    return Err(anyhow::anyhow!(msg));
                }
                issues.record_error(msg);
                continue;
            }
        };
        let archive_path = rel.to_string_lossy().replace('\\', "/");
        if archive_path.is_empty() {
            continue;
        }

        let is_dir = entry.file_type().is_dir();
        if exclude.is_match(&archive_path)
            || (is_dir && exclude.is_match(format!("{archive_path}/")))
        {
            if is_dir {
                iter.skip_current_dir();
            }
            continue;
        }

        let is_symlink_path = entry.path_is_symlink();
        if is_symlink_path && source.symlink_policy == FsSymlinkPolicy::Skip {
            let target = std::fs::read_link(entry.path())
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "<unknown>".to_string());
            issues.record_warning(format!("skipped symlink: {archive_path} -> {target}"));
            continue;
        }

        if entry.file_type().is_file() {
            if has_includes && !include.is_match(&archive_path) {
                continue;
            }

            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(error) => {
                    let msg = format!("metadata error: {archive_path}: {error}");
                    if source.error_policy == FsErrorPolicy::FailFast {
                        return Err(anyhow::anyhow!(msg));
                    }
                    issues.record_error(msg);
                    continue;
                }
            };

            write_file_entry(
                entry.path(),
                &archive_path,
                &meta,
                is_symlink_path,
                source,
                data_dir,
                entries_writer,
                entries_count,
                issues,
                stats,
                hardlink_index,
                seen_archive_paths,
            )?;
            continue;
        }

        if entry.file_type().is_dir() {
            write_dir_entry(
                entry.path(),
                &archive_path,
                source,
                data_dir,
                entries_writer,
                entries_count,
                issues,
                seen_archive_paths,
            )?;
            continue;
        }

        if entry.file_type().is_symlink() {
            write_symlink_entry(
                entry.path(),
                &archive_path,
                source,
                data_dir,
                entries_writer,
                entries_count,
                issues,
                seen_archive_paths,
            )?;
            continue;
        }

        let msg = format!("unsupported file type: {archive_path}");
        if source.error_policy == FsErrorPolicy::FailFast {
            return Err(anyhow::anyhow!(msg));
        }
        issues.record_error(msg);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_source_entry(
    path: &Path,
    source: &FilesystemSource,
    exclude: &globset::GlobSet,
    include: &globset::GlobSet,
    has_includes: bool,
    follow_links: bool,
    data_dir: &Path,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    stats: &mut RawTreeBuildStats,
    hardlink_index: &mut HashMap<FileId, String>,
    seen_archive_paths: &mut HashSet<String>,
) -> Result<(), anyhow::Error> {
    let prefix = match archive_prefix_for_path(path) {
        Ok(v) => v,
        Err(error) => {
            let msg = format!("archive path error: {}: {error:#}", path.display());
            if source.error_policy == FsErrorPolicy::FailFast {
                return Err(anyhow::anyhow!(msg));
            }
            issues.record_error(msg);
            return Ok(());
        }
    };
    let meta = match source_meta_for_policy(path, source.symlink_policy) {
        Ok(m) => m,
        Err(error) => {
            let msg = format!("metadata error: {}: {error}", path.display());
            if source.error_policy == FsErrorPolicy::FailFast {
                return Err(anyhow::anyhow!(msg));
            }
            issues.record_error(msg);
            return Ok(());
        }
    };

    if meta.is_dir() {
        // Include the directory entry itself (except filesystem root which maps to empty prefix).
        if !prefix.is_empty()
            && !exclude.is_match(&prefix)
            && !exclude.is_match(format!("{prefix}/"))
        {
            write_dir_entry(
                path,
                &prefix,
                source,
                data_dir,
                entries_writer,
                entries_count,
                issues,
                seen_archive_paths,
            )?;
        }

        let mut iter = WalkDir::new(path).follow_links(follow_links).into_iter();
        while let Some(next) = iter.next() {
            let entry = match next {
                Ok(e) => e,
                Err(error) => {
                    let msg = if let Some(path) = error.path() {
                        format!("walk error: {}: {}", path.display(), error)
                    } else {
                        format!("walk error: {error}")
                    };
                    if source.error_policy == FsErrorPolicy::FailFast {
                        return Err(anyhow::anyhow!(msg));
                    }
                    issues.record_error(msg);
                    continue;
                }
            };
            if entry.path() == path {
                continue;
            }

            let rel = match entry.path().strip_prefix(path) {
                Ok(v) => v,
                Err(error) => {
                    let msg = format!(
                        "path error: {} is not under root {}: {error}",
                        entry.path().display(),
                        path.display()
                    );
                    if source.error_policy == FsErrorPolicy::FailFast {
                        return Err(anyhow::anyhow!(msg));
                    }
                    issues.record_error(msg);
                    continue;
                }
            };
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let archive_path = join_archive_path(&prefix, &rel_str);
            if archive_path.is_empty() {
                continue;
            }

            let is_dir = entry.file_type().is_dir();
            if exclude.is_match(&archive_path)
                || (is_dir && exclude.is_match(format!("{archive_path}/")))
            {
                if is_dir {
                    iter.skip_current_dir();
                }
                continue;
            }

            let is_symlink_path = entry.path_is_symlink();
            if is_symlink_path && source.symlink_policy == FsSymlinkPolicy::Skip {
                let target = std::fs::read_link(entry.path())
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "<unknown>".to_string());
                issues.record_warning(format!("skipped symlink: {archive_path} -> {target}"));
                continue;
            }

            if entry.file_type().is_file() {
                if has_includes && !include.is_match(&archive_path) {
                    continue;
                }

                let meta = match entry.metadata() {
                    Ok(m) => m,
                    Err(error) => {
                        let msg = format!("metadata error: {archive_path}: {error}");
                        if source.error_policy == FsErrorPolicy::FailFast {
                            return Err(anyhow::anyhow!(msg));
                        }
                        issues.record_error(msg);
                        continue;
                    }
                };

                write_file_entry(
                    entry.path(),
                    &archive_path,
                    &meta,
                    is_symlink_path,
                    source,
                    data_dir,
                    entries_writer,
                    entries_count,
                    issues,
                    stats,
                    hardlink_index,
                    seen_archive_paths,
                )?;
                continue;
            }

            if entry.file_type().is_dir() {
                write_dir_entry(
                    entry.path(),
                    &archive_path,
                    source,
                    data_dir,
                    entries_writer,
                    entries_count,
                    issues,
                    seen_archive_paths,
                )?;
                continue;
            }

            if entry.file_type().is_symlink() {
                write_symlink_entry(
                    entry.path(),
                    &archive_path,
                    source,
                    data_dir,
                    entries_writer,
                    entries_count,
                    issues,
                    seen_archive_paths,
                )?;
                continue;
            }

            let msg = format!("unsupported file type: {archive_path}");
            if source.error_policy == FsErrorPolicy::FailFast {
                return Err(anyhow::anyhow!(msg));
            }
            issues.record_error(msg);
        }
        return Ok(());
    }

    // Single file / symlink source
    let archive_path = prefix;
    if archive_path.is_empty() {
        let msg = format!(
            "invalid source path: {} has no archive path",
            path.display()
        );
        if source.error_policy == FsErrorPolicy::FailFast {
            return Err(anyhow::anyhow!(msg));
        }
        issues.record_error(msg);
        return Ok(());
    }

    if exclude.is_match(&archive_path) {
        return Ok(());
    }
    if meta.file_type().is_symlink() && source.symlink_policy == FsSymlinkPolicy::Skip {
        let target = std::fs::read_link(path)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "<unknown>".to_string());
        issues.record_warning(format!("skipped symlink: {archive_path} -> {target}"));
        return Ok(());
    }

    let is_symlink_path = std::fs::symlink_metadata(path)
        .ok()
        .is_some_and(|m| m.file_type().is_symlink());

    if meta.is_file() {
        if has_includes && !include.is_match(&archive_path) {
            return Ok(());
        }
        write_file_entry(
            path,
            &archive_path,
            &meta,
            is_symlink_path,
            source,
            data_dir,
            entries_writer,
            entries_count,
            issues,
            stats,
            hardlink_index,
            seen_archive_paths,
        )?;
        return Ok(());
    }

    if meta.file_type().is_symlink() {
        write_symlink_entry(
            path,
            &archive_path,
            source,
            data_dir,
            entries_writer,
            entries_count,
            issues,
            seen_archive_paths,
        )?;
        return Ok(());
    }

    let msg = format!("unsupported file type: {archive_path}");
    if source.error_policy == FsErrorPolicy::FailFast {
        return Err(anyhow::anyhow!(msg));
    }
    issues.record_error(msg);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_file_entry(
    fs_path: &Path,
    archive_path: &str,
    meta: &std::fs::Metadata,
    is_symlink_path: bool,
    source: &FilesystemSource,
    data_dir: &Path,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    stats: &mut RawTreeBuildStats,
    hardlink_index: &mut HashMap<FileId, String>,
    seen_archive_paths: &mut HashSet<String>,
) -> Result<(), anyhow::Error> {
    if seen_archive_paths.contains(archive_path) {
        issues.record_warning(format!("duplicate archive path (file): {archive_path}"));
        return Ok(());
    }

    let size = meta.len();

    let hardlink_group = if source.hardlink_policy == FsHardlinkPolicy::Keep
        && !is_symlink_path
        && hardlink_candidate(meta)
        && let Some(id) = file_id_for_meta(meta)
    {
        Some(
            hardlink_index
                .entry(id)
                .or_insert_with(|| format!("{}:{}", id.dev, id.ino))
                .clone(),
        )
    } else {
        None
    };

    let dst_path = data_path_for_archive_path(data_dir, archive_path);
    let parent = dst_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("invalid destination path: {}", dst_path.display()))?;
    if let Err(error) = std::fs::create_dir_all(parent) {
        let msg = format!("create dir error: {}: {error}", parent.display());
        if source.error_policy == FsErrorPolicy::FailFast {
            return Err(anyhow::anyhow!(msg));
        }
        issues.record_error(msg);
        return Ok(());
    }

    let file_name = dst_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid destination file name"))?;
    let tmp = dst_path.with_file_name(format!("{file_name}.partial"));
    let _ = std::fs::remove_file(&tmp);

    let (written, hash) = match copy_file_and_hash(fs_path, &tmp) {
        Ok(v) => v,
        Err(error) => {
            let msg = format!("copy error: {archive_path}: {error}");
            if source.error_policy == FsErrorPolicy::FailFast {
                return Err(anyhow::anyhow!(msg));
            }
            issues.record_error(msg);
            return Ok(());
        }
    };
    if written != size {
        let msg = format!("copy size mismatch: {archive_path}: expected {size}, got {written}");
        if source.error_policy == FsErrorPolicy::FailFast {
            return Err(anyhow::anyhow!(msg));
        }
        issues.record_error(msg);
        let _ = std::fs::remove_file(&tmp);
        return Ok(());
    }

    let _ = std::fs::remove_file(&dst_path);
    if let Err(error) = std::fs::rename(&tmp, &dst_path) {
        let msg = format!("rename error: {archive_path}: {error}");
        if source.error_policy == FsErrorPolicy::FailFast {
            return Err(anyhow::anyhow!(msg));
        }
        issues.record_error(msg);
        let _ = std::fs::remove_file(&tmp);
        return Ok(());
    }

    stats.data_files = stats.data_files.saturating_add(1);
    stats.data_bytes = stats.data_bytes.saturating_add(size);

    let (mtime, mode, uid, gid) = meta_fields(meta);
    let xattrs = xattrs_for_path(fs_path);

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
            mtime,
            mode,
            uid,
            gid,
            xattrs,
            symlink_target: None,
            hardlink_group,
        },
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_dir_entry(
    fs_path: &Path,
    archive_path: &str,
    source: &FilesystemSource,
    data_dir: &Path,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    seen_archive_paths: &mut HashSet<String>,
) -> Result<(), anyhow::Error> {
    if seen_archive_paths.contains(archive_path) {
        issues.record_warning(format!("duplicate archive path (dir): {archive_path}"));
        return Ok(());
    }

    let dst_dir = data_path_for_archive_path(data_dir, archive_path);
    if let Err(error) = std::fs::create_dir_all(&dst_dir) {
        let msg = format!("create dir error: {}: {error}", dst_dir.display());
        if source.error_policy == FsErrorPolicy::FailFast {
            return Err(anyhow::anyhow!(msg));
        }
        issues.record_error(msg);
        return Ok(());
    }

    let meta = match source_meta_for_policy(fs_path, source.symlink_policy) {
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

    let (mtime, mode, uid, gid) = meta_fields(&meta);
    let xattrs = xattrs_for_path(fs_path);

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
            mtime,
            mode,
            uid,
            gid,
            xattrs,
            symlink_target: None,
            hardlink_group: None,
        },
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_symlink_entry(
    fs_path: &Path,
    archive_path: &str,
    source: &FilesystemSource,
    _data_dir: &Path,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    seen_archive_paths: &mut HashSet<String>,
) -> Result<(), anyhow::Error> {
    if seen_archive_paths.contains(archive_path) {
        issues.record_warning(format!("duplicate archive path (symlink): {archive_path}"));
        return Ok(());
    }

    let target = std::fs::read_link(fs_path)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "<unknown>".to_string());

    let meta = match std::fs::symlink_metadata(fs_path) {
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
    let (mtime, mode, uid, gid) = meta_fields(&meta);
    let xattrs = xattrs_for_path(fs_path);

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
            mtime,
            mode,
            uid,
            gid,
            xattrs,
            symlink_target: Some(target),
            hardlink_group: None,
        },
    )?;
    Ok(())
}

fn data_path_for_archive_path(data_dir: &Path, archive_path: &str) -> PathBuf {
    let mut out = data_dir.to_path_buf();
    for seg in archive_path.split('/').filter(|s| !s.is_empty()) {
        out.push(seg);
    }
    out
}

fn copy_file_and_hash(src: &Path, dst: &Path) -> Result<(u64, String), anyhow::Error> {
    let mut input = File::open(src)?;
    let mut out = File::create(dst)?;

    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; 1024 * 1024];

    let mut written = 0u64;
    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        out.write_all(&buf[..n])?;
        written = written.saturating_add(n as u64);
    }
    out.flush()?;

    let hash = hasher.finalize().to_hex().to_string();
    Ok((written, hash))
}

fn meta_fields(meta: &std::fs::Metadata) -> (Option<u64>, Option<u32>, Option<u64>, Option<u64>) {
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt as _;
        let mode = Some(meta.mode());
        let uid = Some(meta.uid() as u64);
        let gid = Some(meta.gid() as u64);
        (mtime, mode, uid, gid)
    }

    #[cfg(not(unix))]
    {
        let _ = meta;
        (mtime, None, None, None)
    }
}

#[cfg(unix)]
fn xattrs_for_path(path: &Path) -> Option<BTreeMap<String, String>> {
    let names = xattr::list(path).ok()?;
    let mut out = BTreeMap::<String, String>::new();

    for name in names {
        let name_str = name.to_string_lossy().to_string();
        let value = match xattr::get(path, &name) {
            Ok(Some(v)) => v,
            Ok(None) => continue,
            Err(_) => continue,
        };
        out.insert(
            name_str,
            base64::engine::general_purpose::STANDARD.encode(value),
        );
    }

    if out.is_empty() { None } else { Some(out) }
}

#[cfg(not(unix))]
fn xattrs_for_path(_path: &Path) -> Option<BTreeMap<String, String>> {
    None
}

fn source_meta_for_policy(
    path: &Path,
    policy: FsSymlinkPolicy,
) -> Result<std::fs::Metadata, std::io::Error> {
    if policy == FsSymlinkPolicy::Follow {
        std::fs::metadata(path)
    } else {
        std::fs::symlink_metadata(path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FileId {
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
