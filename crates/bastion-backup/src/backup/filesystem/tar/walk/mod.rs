use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;

use bastion_core::job_spec::{FilesystemSource, FsSymlinkPolicy};

use super::super::FilesystemBuildIssues;
use super::super::entries_index::EntriesIndexWriter;
use super::super::reborrow_progress;
use super::super::util::{archive_prefix_for_path, compile_globset, join_archive_path};
use super::entry::{
    FileId, HardlinkRecord, source_meta_for_policy, write_dir_entry, write_file_entry,
    write_symlink_entry,
};

mod legacy_root;
mod source_entry;

pub(super) fn write_tar_entries<W: Write>(
    tar: &mut ::tar::Builder<W>,
    source: &FilesystemSource,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    mut progress: Option<&mut super::super::FilesystemBuildProgressCtx<'_>>,
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

            source_entry::write_source_entry(
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
                reborrow_progress(&mut progress),
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
        legacy_root::write_legacy_root(
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
            reborrow_progress(&mut progress),
        )?;
    }

    Ok(())
}
