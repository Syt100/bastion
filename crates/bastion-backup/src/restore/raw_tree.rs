use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[cfg(unix)]
use base64::Engine as _;
use bastion_core::progress::ProgressUnitsV1;

use super::ConflictPolicy;
use super::RestoreSelection;
use super::entries_index::EntryRecord;
use super::path;
use super::selection;
use super::sinks::{RestoreSink, WebdavSink, remove_existing_path};
use super::sources::ArtifactSource;

pub(super) fn restore_raw_tree_to_local_fs(
    source: &dyn ArtifactSource,
    entries_index_path: &Path,
    staging_dir: &Path,
    destination_dir: &Path,
    conflict: ConflictPolicy,
    selection: Option<&RestoreSelection>,
    on_progress: Option<&dyn Fn(ProgressUnitsV1)>,
) -> Result<(), anyhow::Error> {
    std::fs::create_dir_all(destination_dir)?;

    let selection = selection
        .map(selection::normalize_restore_selection)
        .transpose()?;

    // For conflict=skip, if a file exists where we need a directory, skip the whole subtree.
    let mut blocked_prefixes = Vec::<String>::new();

    // hardlink_group -> first restored destination path (absolute).
    let mut hardlink_first = HashMap::<String, PathBuf>::new();

    const RAW_TREE_PROGRESS_MIN_INTERVAL: Duration = Duration::from_secs(1);
    let mut progress_done = ProgressUnitsV1::default();
    let mut progress_last_emit = Instant::now();
    if let Some(cb) = on_progress {
        cb(progress_done);
    }

    for_each_entry(entries_index_path, |rec| {
        let archive_path = rec.path.clone();

        if let Some(sel) = selection.as_ref()
            && !sel.matches(&archive_path)
        {
            return Ok(());
        }
        if blocked_prefixes
            .iter()
            .any(|p| is_under_prefix(&archive_path, p))
        {
            return Ok(());
        }

        let rel_raw = PathBuf::from(&archive_path);
        let rel = path::safe_join(Path::new(""), &rel_raw)
            .ok_or_else(|| anyhow::anyhow!("invalid entry path: {}", archive_path))?;

        let progress = match rec.kind.as_str() {
            "dir" => {
                let dest_path = destination_dir.join(&rel);
                if dest_path.exists() {
                    let meta = std::fs::symlink_metadata(&dest_path)?;
                    if meta.is_dir() {
                        apply_fs_metadata_best_effort(&dest_path, &rec, FsEntryKind::Dir);
                        Some((true, 0))
                    } else {
                        match conflict {
                            ConflictPolicy::Overwrite => {
                                remove_existing_path(&dest_path)?;
                            }
                            ConflictPolicy::Skip => {
                                blocked_prefixes.push(archive_path);
                                return Ok(());
                            }
                            ConflictPolicy::Fail => {
                                anyhow::bail!("restore conflict: {} exists", dest_path.display());
                            }
                        }

                        std::fs::create_dir_all(&dest_path)?;
                        apply_fs_metadata_best_effort(&dest_path, &rec, FsEntryKind::Dir);
                        Some((true, 0))
                    }
                } else {
                    std::fs::create_dir_all(&dest_path)?;
                    apply_fs_metadata_best_effort(&dest_path, &rec, FsEntryKind::Dir);
                    Some((true, 0))
                }
            }
            "file" => {
                if ensure_parent_dirs(
                    destination_dir,
                    &archive_path,
                    conflict,
                    &mut blocked_prefixes,
                )? {
                    // blocked by a parent path conflict under conflict=skip
                    return Ok(());
                }

                let dest_path = destination_dir.join(&rel);
                if dest_path.exists() {
                    match conflict {
                        ConflictPolicy::Overwrite => {
                            remove_existing_path(&dest_path)?;
                        }
                        ConflictPolicy::Skip => return Ok(()),
                        ConflictPolicy::Fail => {
                            anyhow::bail!("restore conflict: {} exists", dest_path.display());
                        }
                    }
                }

                // Hardlink best-effort: if we already restored a file for this group, link to it.
                if let Some(group) = rec.hardlink_group.as_deref()
                    && let Some(first) = hardlink_first.get(group)
                    && try_hard_link(first, &dest_path).is_ok()
                {
                    apply_fs_metadata_best_effort(&dest_path, &rec, FsEntryKind::File);
                    Some((false, rec.size))
                } else {
                    let mut reader =
                        source.open_raw_tree_file_reader(&archive_path, rec.size, staging_dir)?;
                    write_file_atomic(&dest_path, &mut reader, rec.size)?;
                    apply_fs_metadata_best_effort(&dest_path, &rec, FsEntryKind::File);

                    if let Some(group) = rec.hardlink_group.as_deref() {
                        hardlink_first.entry(group.to_string()).or_insert(dest_path);
                    }
                    Some((false, rec.size))
                }
            }
            "symlink" => {
                let Some(target) = rec.symlink_target.as_deref() else {
                    anyhow::bail!("missing symlink_target for {}", archive_path);
                };

                if ensure_parent_dirs(
                    destination_dir,
                    &archive_path,
                    conflict,
                    &mut blocked_prefixes,
                )? {
                    return Ok(());
                }

                let dest_path = destination_dir.join(&rel);
                if dest_path.exists() {
                    match conflict {
                        ConflictPolicy::Overwrite => {
                            remove_existing_path(&dest_path)?;
                        }
                        ConflictPolicy::Skip => return Ok(()),
                        ConflictPolicy::Fail => {
                            anyhow::bail!("restore conflict: {} exists", dest_path.display());
                        }
                    }
                }

                create_symlink(target, &dest_path)?;
                apply_fs_metadata_best_effort(&dest_path, &rec, FsEntryKind::Symlink);
                Some((false, 0))
            }
            other => {
                // Skip unusual kinds for now (best-effort restore).
                tracing::debug!(kind = %other, path = %archive_path, "skipping raw-tree entry");
                None
            }
        };

        if let Some((is_dir, size)) = progress {
            if is_dir {
                progress_done.dirs = progress_done.dirs.saturating_add(1);
            } else {
                progress_done.files = progress_done.files.saturating_add(1);
                progress_done.bytes = progress_done.bytes.saturating_add(size);
            }

            if let Some(cb) = on_progress
                && progress_last_emit.elapsed() >= RAW_TREE_PROGRESS_MIN_INTERVAL
            {
                progress_last_emit = Instant::now();
                cb(progress_done);
            }
        }

        Ok(())
    })?;

    if let Some(cb) = on_progress {
        cb(progress_done);
    }

    Ok(())
}

pub(super) fn restore_raw_tree_to_webdav(
    source: &dyn ArtifactSource,
    entries_index_path: &Path,
    staging_dir: &Path,
    sink: &mut WebdavSink,
    selection: Option<&RestoreSelection>,
    on_progress: Option<&dyn Fn(ProgressUnitsV1)>,
) -> Result<(), anyhow::Error> {
    sink.prepare()?;

    let selection = selection
        .map(selection::normalize_restore_selection)
        .transpose()?;

    const RAW_TREE_PROGRESS_MIN_INTERVAL: Duration = Duration::from_secs(1);
    let mut progress_done = ProgressUnitsV1::default();
    let mut progress_last_emit = Instant::now();
    if let Some(cb) = on_progress {
        cb(progress_done);
    }

    for_each_entry(entries_index_path, |rec| {
        let archive_path = rec.path.clone();

        if let Some(sel) = selection.as_ref()
            && !sel.matches(&archive_path)
        {
            return Ok(());
        }

        let rel_raw = PathBuf::from(&archive_path);
        let rel = path::safe_join(Path::new(""), &rel_raw)
            .ok_or_else(|| anyhow::anyhow!("invalid entry path: {}", archive_path))?;

        let progress = match rec.kind.as_str() {
            "dir" => {
                sink.apply_raw_tree_dir(&rel, &rec)?;
                Some((true, 0))
            }
            "file" => {
                let reader =
                    source.open_raw_tree_file_reader(&archive_path, rec.size, staging_dir)?;
                sink.apply_raw_tree_file(&rel, &rec, reader)?;
                Some((false, rec.size))
            }
            "symlink" => {
                let target = rec.symlink_target.as_deref().unwrap_or("<unknown>");
                sink.apply_raw_tree_symlink(&rel, &rec, target)?;
                Some((false, 0))
            }
            _ => None,
        };

        if let Some((is_dir, size)) = progress {
            if is_dir {
                progress_done.dirs = progress_done.dirs.saturating_add(1);
            } else {
                progress_done.files = progress_done.files.saturating_add(1);
                progress_done.bytes = progress_done.bytes.saturating_add(size);
            }

            if let Some(cb) = on_progress
                && progress_last_emit.elapsed() >= RAW_TREE_PROGRESS_MIN_INTERVAL
            {
                progress_last_emit = Instant::now();
                cb(progress_done);
            }
        }

        Ok(())
    })?;

    if let Some(cb) = on_progress {
        cb(progress_done);
    }

    Ok(())
}

fn for_each_entry(
    entries_index_path: &Path,
    mut f: impl FnMut(EntryRecord) -> Result<(), anyhow::Error>,
) -> Result<(), anyhow::Error> {
    let file = std::fs::File::open(entries_index_path)?;
    let decoder = zstd::Decoder::new(file)?;
    let mut reader = BufReader::new(decoder);

    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let rec: EntryRecord = serde_json::from_str(trimmed)?;
        f(rec)?;
    }

    Ok(())
}

fn is_under_prefix(path: &str, prefix: &str) -> bool {
    path == prefix || (path.starts_with(prefix) && path.as_bytes().get(prefix.len()) == Some(&b'/'))
}

fn ensure_parent_dirs(
    destination_dir: &Path,
    archive_path: &str,
    conflict: ConflictPolicy,
    blocked_prefixes: &mut Vec<String>,
) -> Result<bool, anyhow::Error> {
    let mut prefix = String::new();
    let mut parts = archive_path.split('/').peekable();

    while let Some(seg) = parts.next() {
        let is_last = parts.peek().is_none();
        if is_last {
            break;
        }

        if seg.is_empty() {
            continue;
        }
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(seg);

        if blocked_prefixes.iter().any(|p| is_under_prefix(&prefix, p)) {
            return Ok(true);
        }

        let dir_path = destination_dir.join(&prefix);
        if dir_path.exists() {
            let meta = std::fs::symlink_metadata(&dir_path)?;
            if meta.is_dir() {
                continue;
            }
            match conflict {
                ConflictPolicy::Overwrite => {
                    remove_existing_path(&dir_path)?;
                    std::fs::create_dir_all(&dir_path)?;
                }
                ConflictPolicy::Skip => {
                    blocked_prefixes.push(prefix.clone());
                    return Ok(true);
                }
                ConflictPolicy::Fail => {
                    anyhow::bail!("restore conflict: {} exists", dir_path.display());
                }
            }
            continue;
        }

        std::fs::create_dir_all(&dir_path)?;
    }

    Ok(false)
}

fn write_file_atomic(
    dest_path: &Path,
    reader: &mut (impl Read + ?Sized),
    expected_size: u64,
) -> Result<(), anyhow::Error> {
    let parent = dest_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("invalid destination path"))?;
    std::fs::create_dir_all(parent)?;

    let file_name = dest_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid destination file name"))?;
    let tmp = dest_path.with_file_name(format!("{file_name}.partial"));
    let _ = std::fs::remove_file(&tmp);

    let mut out = std::fs::File::create(&tmp)?;
    let written = std::io::copy(reader, &mut out)?;
    out.flush()?;
    if written != expected_size {
        let _ = std::fs::remove_file(&tmp);
        anyhow::bail!(
            "restore entry size mismatch for {}: expected {}, got {}",
            dest_path.display(),
            expected_size,
            written
        );
    }

    let _ = std::fs::remove_file(dest_path);
    std::fs::rename(&tmp, dest_path)?;
    Ok(())
}

fn try_hard_link(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    std::fs::hard_link(src, dst)
}

fn create_symlink(target: &str, dst: &Path) -> Result<(), anyhow::Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs as unix_fs;
        unix_fs::symlink(target, dst)?;
        return Ok(());
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs as win_fs;
        // Best-effort: try file symlink first, then directory.
        if win_fs::symlink_file(target, dst).is_ok() {
            return Ok(());
        }
        let _ = win_fs::symlink_dir(target, dst);
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(anyhow::anyhow!("symlink is not supported on this platform"))
}

#[derive(Debug, Clone, Copy)]
enum FsEntryKind {
    File,
    Dir,
    Symlink,
}

fn apply_fs_metadata_best_effort(path: &Path, rec: &EntryRecord, kind: FsEntryKind) {
    let _ = apply_fs_metadata_best_effort_inner(path, rec, kind);
}

fn apply_fs_metadata_best_effort_inner(
    path: &Path,
    rec: &EntryRecord,
    kind: FsEntryKind,
) -> Result<(), anyhow::Error> {
    #[cfg(not(unix))]
    {
        let _ = kind;
    }

    // mode
    #[cfg(unix)]
    if let Some(mode) = rec.mode {
        use std::os::unix::fs::PermissionsExt as _;
        let perm = std::fs::Permissions::from_mode(mode & 0o7777);
        let _ = std::fs::set_permissions(path, perm);
    }

    // owner
    #[cfg(unix)]
    if let (Some(uid), Some(gid)) = (rec.uid, rec.gid) {
        use nix::unistd::{Gid, Uid};
        let uid = Uid::from_raw(uid as u32);
        let gid = Gid::from_raw(gid as u32);

        // Best-effort: only apply to non-symlinks (ownership for the link itself is not portable).
        if !matches!(kind, FsEntryKind::Symlink) {
            let _ = nix::unistd::chown(path, Some(uid), Some(gid));
        }
    }

    // xattrs (skip symlinks; many platforms follow links implicitly)
    #[cfg(unix)]
    if !matches!(kind, FsEntryKind::Symlink)
        && let Some(xattrs) = rec.xattrs.as_ref()
    {
        for (name, value_b64) in xattrs {
            let Ok(value) = base64::engine::general_purpose::STANDARD.decode(value_b64) else {
                continue;
            };
            let _ = xattr::set(path, name, &value);
        }
    }

    // mtime (best-effort; apply last)
    if let Some(mtime) = rec.mtime {
        let ft = filetime::FileTime::from_unix_time(mtime as i64, 0);
        let _ = filetime::set_file_mtime(path, ft);
    }

    Ok(())
}
