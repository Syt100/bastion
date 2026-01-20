use std::collections::HashSet;
use std::path::{Path, PathBuf};

use bastion_core::job_spec::{FilesystemSource, FsErrorPolicy, FsSymlinkPolicy};
use bastion_core::progress::ProgressUnitsV1;
use walkdir::WalkDir;

use super::FilesystemBuildIssues;
use super::util::{archive_prefix_for_path, compile_globset, join_archive_path};

fn meta_for_policy(
    path: &Path,
    policy: FsSymlinkPolicy,
) -> Result<std::fs::Metadata, std::io::Error> {
    if policy == FsSymlinkPolicy::Follow {
        std::fs::metadata(path)
    } else {
        std::fs::symlink_metadata(path)
    }
}

pub(super) fn scan_filesystem_source(
    source: &FilesystemSource,
    issues: &mut FilesystemBuildIssues,
    mut progress: Option<&mut super::FilesystemBuildProgressCtx<'_>>,
) -> Result<ProgressUnitsV1, anyhow::Error> {
    let exclude = compile_globset(&source.exclude)?;
    let include = compile_globset(&source.include)?;
    let has_includes = !source.include.is_empty();
    let follow_links = source.symlink_policy == FsSymlinkPolicy::Follow;

    let mut totals = ProgressUnitsV1::default();

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
        for p in raw_paths {
            if covered_dirs.iter().any(|dir| p.strip_prefix(dir).is_ok()) {
                continue;
            }

            if let Ok(meta) = meta_for_policy(&p, source.symlink_policy)
                && meta.is_dir()
            {
                covered_dirs.push(p.clone());
            }

            scan_source_path(
                source,
                p.as_path(),
                &exclude,
                &include,
                has_includes,
                follow_links,
                issues,
                &mut totals,
                progress.as_deref_mut(),
            )?;
        }

        return Ok(totals);
    }

    scan_legacy_root(
        source,
        Path::new(source.root.trim()),
        &exclude,
        &include,
        has_includes,
        follow_links,
        issues,
        &mut totals,
        progress.as_deref_mut(),
    )?;

    Ok(totals)
}

#[allow(clippy::too_many_arguments)]
fn scan_source_path(
    source: &FilesystemSource,
    path: &Path,
    exclude: &globset::GlobSet,
    include: &globset::GlobSet,
    has_includes: bool,
    follow_links: bool,
    issues: &mut FilesystemBuildIssues,
    totals: &mut ProgressUnitsV1,
    mut progress: Option<&mut super::FilesystemBuildProgressCtx<'_>>,
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

    let meta = match meta_for_policy(path, source.symlink_policy) {
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
            totals.dirs = totals.dirs.saturating_add(1);
            if let Some(ctx) = progress.as_deref_mut() {
                ctx.done = *totals;
                ctx.maybe_emit(false);
            }
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

                totals.files = totals.files.saturating_add(1);
                totals.bytes = totals.bytes.saturating_add(meta.len());
                if let Some(ctx) = progress.as_deref_mut() {
                    ctx.done = *totals;
                    ctx.maybe_emit(false);
                }
                continue;
            }

            if entry.file_type().is_dir() {
                totals.dirs = totals.dirs.saturating_add(1);
                if let Some(ctx) = progress.as_deref_mut() {
                    ctx.done = *totals;
                    ctx.maybe_emit(false);
                }
                continue;
            }

            if entry.file_type().is_symlink() {
                totals.files = totals.files.saturating_add(1);
                if let Some(ctx) = progress.as_deref_mut() {
                    ctx.done = *totals;
                    ctx.maybe_emit(false);
                }
                continue;
            }
        }

        return Ok(());
    }

    // Single file / symlink source.
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
        return Ok(());
    }

    if meta.is_file() {
        if has_includes && !include.is_match(&archive_path) {
            return Ok(());
        }
        totals.files = totals.files.saturating_add(1);
        totals.bytes = totals.bytes.saturating_add(meta.len());
        if let Some(ctx) = progress.as_deref_mut() {
            ctx.done = *totals;
            ctx.maybe_emit(false);
        }
        return Ok(());
    }

    if meta.file_type().is_symlink() {
        totals.files = totals.files.saturating_add(1);
        if let Some(ctx) = progress.as_deref_mut() {
            ctx.done = *totals;
            ctx.maybe_emit(false);
        }
        return Ok(());
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn scan_legacy_root(
    source: &FilesystemSource,
    root: &Path,
    exclude: &globset::GlobSet,
    include: &globset::GlobSet,
    has_includes: bool,
    follow_links: bool,
    issues: &mut FilesystemBuildIssues,
    totals: &mut ProgressUnitsV1,
    mut progress: Option<&mut super::FilesystemBuildProgressCtx<'_>>,
) -> Result<(), anyhow::Error> {
    if root.as_os_str().is_empty() {
        anyhow::bail!("filesystem.source.root is required");
    }

    let meta = match meta_for_policy(root, source.symlink_policy) {
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
            return Ok(());
        }

        if meta.is_file() {
            if has_includes && !include.is_match(name) {
                return Ok(());
            }
            totals.files = totals.files.saturating_add(1);
            totals.bytes = totals.bytes.saturating_add(meta.len());
        } else {
            totals.files = totals.files.saturating_add(1);
        }

        if let Some(ctx) = progress.as_deref_mut() {
            ctx.done = *totals;
            ctx.maybe_emit(false);
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

            totals.files = totals.files.saturating_add(1);
            totals.bytes = totals.bytes.saturating_add(meta.len());
            if let Some(ctx) = progress.as_deref_mut() {
                ctx.done = *totals;
                ctx.maybe_emit(false);
            }
            continue;
        }

        if entry.file_type().is_dir() {
            totals.dirs = totals.dirs.saturating_add(1);
            if let Some(ctx) = progress.as_deref_mut() {
                ctx.done = *totals;
                ctx.maybe_emit(false);
            }
            continue;
        }

        if entry.file_type().is_symlink() {
            totals.files = totals.files.saturating_add(1);
            if let Some(ctx) = progress.as_deref_mut() {
                ctx.done = *totals;
                ctx.maybe_emit(false);
            }
            continue;
        }
    }

    Ok(())
}
