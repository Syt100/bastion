use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};

use bastion_core::job_spec::{FilesystemSource, FsErrorPolicy, FsSymlinkPolicy};
use walkdir::WalkDir;

use super::super::FilesystemBuildIssues;
use super::super::entries_index::EntriesIndexWriter;
use super::super::util::{archive_prefix_for_path, compile_globset, join_archive_path};
use super::entry::{
    FileId, HardlinkRecord, source_meta_for_policy, write_dir_entry, write_file_entry,
    write_symlink_entry,
};

pub(super) fn write_tar_entries<W: Write>(
    tar: &mut ::tar::Builder<W>,
    source: &FilesystemSource,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
) -> Result<(), anyhow::Error> {
    tar.follow_symlinks(source.symlink_policy == FsSymlinkPolicy::Follow);

    let exclude = compile_globset(&source.exclude)?;
    let include = compile_globset(&source.include)?;
    let has_includes = !source.include.is_empty();

    #[cfg(not(unix))]
    if source.hardlink_policy == bastion_core::job_spec::FsHardlinkPolicy::Keep {
        issues.record_warning(
            "hardlink_policy=keep is not supported on this platform; storing as copies",
        );
    }

    let mut hardlink_index = HashMap::<FileId, HardlinkRecord>::new();
    let mut seen_archive_paths = HashSet::<String>::new();

    let follow_links = source.symlink_policy == FsSymlinkPolicy::Follow;

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
                tar,
                &p,
                source,
                &exclude,
                &include,
                has_includes,
                follow_links,
                entries_writer,
                entries_count,
                issues,
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
            tar,
            &root,
            source,
            &exclude,
            &include,
            has_includes,
            follow_links,
            entries_writer,
            entries_count,
            issues,
            &mut hardlink_index,
            &mut seen_archive_paths,
        )?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_source_entry<W: Write>(
    tar: &mut ::tar::Builder<W>,
    path: &Path,
    source: &FilesystemSource,
    exclude: &globset::GlobSet,
    include: &globset::GlobSet,
    has_includes: bool,
    follow_links: bool,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    hardlink_index: &mut HashMap<FileId, HardlinkRecord>,
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
                tar,
                path,
                &prefix,
                source,
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
                    tar,
                    entry.path(),
                    &archive_path,
                    &meta,
                    is_symlink_path,
                    source,
                    entries_writer,
                    entries_count,
                    issues,
                    hardlink_index,
                    seen_archive_paths,
                )?;
                continue;
            }

            if entry.file_type().is_dir() {
                write_dir_entry(
                    tar,
                    entry.path(),
                    &archive_path,
                    source,
                    entries_writer,
                    entries_count,
                    issues,
                    seen_archive_paths,
                )?;
                continue;
            }

            if entry.file_type().is_symlink() {
                write_symlink_entry(
                    tar,
                    entry.path(),
                    &archive_path,
                    source,
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
            tar,
            path,
            &archive_path,
            &meta,
            is_symlink_path,
            source,
            entries_writer,
            entries_count,
            issues,
            hardlink_index,
            seen_archive_paths,
        )?;
        return Ok(());
    }

    if meta.file_type().is_symlink() {
        write_symlink_entry(
            tar,
            path,
            &archive_path,
            source,
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
fn write_legacy_root<W: Write>(
    tar: &mut ::tar::Builder<W>,
    root: &Path,
    source: &FilesystemSource,
    exclude: &globset::GlobSet,
    include: &globset::GlobSet,
    has_includes: bool,
    follow_links: bool,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    hardlink_index: &mut HashMap<FileId, HardlinkRecord>,
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
                tar,
                root,
                name,
                &meta,
                is_symlink_path,
                source,
                entries_writer,
                entries_count,
                issues,
                hardlink_index,
                seen_archive_paths,
            )?;
        } else {
            write_symlink_entry(
                tar,
                root,
                name,
                source,
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
                tar,
                entry.path(),
                &archive_path,
                &meta,
                is_symlink_path,
                source,
                entries_writer,
                entries_count,
                issues,
                hardlink_index,
                seen_archive_paths,
            )?;
            continue;
        }

        if entry.file_type().is_dir() {
            write_dir_entry(
                tar,
                entry.path(),
                &archive_path,
                source,
                entries_writer,
                entries_count,
                issues,
                seen_archive_paths,
            )?;
            continue;
        }

        if entry.file_type().is_symlink() {
            write_symlink_entry(
                tar,
                entry.path(),
                &archive_path,
                source,
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
