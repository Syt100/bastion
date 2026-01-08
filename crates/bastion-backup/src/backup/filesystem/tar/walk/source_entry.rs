use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;

use bastion_core::job_spec::{FilesystemSource, FsErrorPolicy, FsSymlinkPolicy};
use walkdir::WalkDir;

use super::{
    EntriesIndexWriter, FileId, FilesystemBuildIssues, HardlinkRecord, archive_prefix_for_path,
    join_archive_path, source_meta_for_policy, write_dir_entry, write_file_entry,
    write_symlink_entry,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn write_source_entry<W: Write>(
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
