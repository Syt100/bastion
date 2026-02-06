use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[cfg(unix)]
use base64::Engine as _;
use bastion_core::job_spec::{FilesystemSource, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy};
use bastion_core::manifest::HashAlgorithm;
use bastion_targets::{WebdavClient, WebdavCredentials};
use url::Url;
use walkdir::WalkDir;

use crate::backup::source_consistency::{
    FileFingerprintV2, SourceConsistencyTracker, detect_change_reason, fingerprint_for_meta,
};

use super::FilesystemBuildIssues;
use super::RawTreeBuildStats;
use super::entries_index::{EntriesIndexWriter, EntryRecord, write_entry_record};
use super::util::{archive_prefix_for_path, compile_globset, join_archive_path};

trait RawTreeDataSink {
    fn ensure_dir(&mut self, archive_path: &str) -> Result<(), anyhow::Error>;

    fn store_file_hashing_blake3(
        &mut self,
        fs_path: &Path,
        archive_path: &str,
        size: u64,
    ) -> Result<(String, Option<FileFingerprintV2>), anyhow::Error>;
}

struct LocalDataSink {
    data_dir: PathBuf,
}

impl LocalDataSink {
    fn new(stage_dir: &Path) -> Result<Self, anyhow::Error> {
        let data_dir = stage_dir.join("data");
        std::fs::create_dir_all(&data_dir)?;
        Ok(Self { data_dir })
    }
}

impl RawTreeDataSink for LocalDataSink {
    fn ensure_dir(&mut self, archive_path: &str) -> Result<(), anyhow::Error> {
        let dst_dir = data_path_for_archive_path(&self.data_dir, archive_path);
        std::fs::create_dir_all(&dst_dir)?;
        Ok(())
    }

    fn store_file_hashing_blake3(
        &mut self,
        fs_path: &Path,
        archive_path: &str,
        size: u64,
    ) -> Result<(String, Option<FileFingerprintV2>), anyhow::Error> {
        let dst_path = data_path_for_archive_path(&self.data_dir, archive_path);
        let parent = dst_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("invalid destination path: {}", dst_path.display()))?;
        std::fs::create_dir_all(parent)?;

        let file_name = dst_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow::anyhow!("invalid destination file name"))?;
        let tmp = dst_path.with_file_name(format!("{file_name}.partial"));
        let _ = std::fs::remove_file(&tmp);

        let CopyFileAndHashResult {
            written,
            hash,
            after_handle,
        } = match copy_file_and_hash(fs_path, &tmp) {
            Ok(v) => v,
            Err(error) => {
                let _ = std::fs::remove_file(&tmp);
                return Err(error);
            }
        };

        if written != size {
            let _ = std::fs::remove_file(&tmp);
            anyhow::bail!("copy size mismatch: expected {size}, got {written}");
        }

        let _ = std::fs::remove_file(&dst_path);
        if let Err(error) = std::fs::rename(&tmp, &dst_path) {
            let _ = std::fs::remove_file(&tmp);
            return Err(anyhow::Error::new(error));
        }
        Ok((hash, after_handle))
    }
}

struct WebdavDataSink {
    handle: tokio::runtime::Handle,
    client: WebdavClient,
    data_url: Url,
    ensured_collections: HashSet<String>,
    max_attempts: u32,
    resume_by_size: bool,
}

impl WebdavDataSink {
    fn new(
        handle: tokio::runtime::Handle,
        base_url: &str,
        credentials: WebdavCredentials,
        job_id: &str,
        run_id: &str,
        max_attempts: u32,
        resume_by_size: bool,
    ) -> Result<Self, anyhow::Error> {
        let mut base_url = Url::parse(base_url)?;
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }

        let client = WebdavClient::new(base_url.clone(), credentials)?;

        let job_url = base_url.join(&format!("{job_id}/"))?;
        let run_url = job_url.join(&format!("{run_id}/"))?;
        let data_url = run_url.join("data/")?;

        let mut out = Self {
            handle,
            client,
            data_url: data_url.clone(),
            ensured_collections: HashSet::new(),
            max_attempts,
            resume_by_size,
        };

        out.ensure_collection(&job_url)?;
        out.ensure_collection(&run_url)?;
        out.ensure_collection(&data_url)?;
        Ok(out)
    }

    fn ensure_collection(&mut self, url: &Url) -> Result<(), anyhow::Error> {
        let key = url.as_str().to_string();
        if self.ensured_collections.contains(&key) {
            return Ok(());
        }
        self.handle.block_on(self.client.ensure_collection(url))?;
        self.ensured_collections.insert(key);
        Ok(())
    }

    fn dir_url_for_archive_path(&self, archive_path: &str) -> Result<Url, anyhow::Error> {
        let mut dir_url = self.data_url.clone();
        {
            let mut segs = dir_url
                .path_segments_mut()
                .map_err(|_| anyhow::anyhow!("data_url cannot be a base"))?;
            segs.pop_if_empty();
            for seg in archive_path
                .split('/')
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                segs.push(seg);
            }
        }
        if !dir_url.path().ends_with('/') {
            dir_url.set_path(&format!("{}/", dir_url.path()));
        }
        Ok(dir_url)
    }

    fn file_urls_for_archive_path(&self, archive_path: &str) -> Result<(Url, Url), anyhow::Error> {
        let mut dir_url = self.data_url.clone();
        let mut file_url = self.data_url.clone();

        let segments = archive_path
            .split('/')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        if segments.is_empty() {
            anyhow::bail!("invalid raw-tree archive path: {archive_path}");
        }

        {
            let mut segs = dir_url
                .path_segments_mut()
                .map_err(|_| anyhow::anyhow!("data_url cannot be a base"))?;
            segs.pop_if_empty();
            for seg in &segments[..segments.len().saturating_sub(1)] {
                segs.push(seg);
            }
        }
        if !dir_url.path().ends_with('/') {
            dir_url.set_path(&format!("{}/", dir_url.path()));
        }

        {
            let mut segs = file_url
                .path_segments_mut()
                .map_err(|_| anyhow::anyhow!("data_url cannot be a base"))?;
            segs.pop_if_empty();
            for seg in &segments {
                segs.push(seg);
            }
        }

        Ok((dir_url, file_url))
    }
}

impl RawTreeDataSink for WebdavDataSink {
    fn ensure_dir(&mut self, archive_path: &str) -> Result<(), anyhow::Error> {
        let url = self.dir_url_for_archive_path(archive_path)?;
        self.ensure_collection(&url)?;
        Ok(())
    }

    fn store_file_hashing_blake3(
        &mut self,
        fs_path: &Path,
        archive_path: &str,
        size: u64,
    ) -> Result<(String, Option<FileFingerprintV2>), anyhow::Error> {
        let (dir_url, file_url) = self.file_urls_for_archive_path(archive_path)?;
        self.ensure_collection(&dir_url)?;

        if self.resume_by_size
            && let Some(existing) = self.handle.block_on(self.client.head_size(&file_url))?
            && existing == size
        {
            let (hash, after_handle) = hash_file_and_fingerprint(fs_path)?;
            return Ok((hash, after_handle));
        }

        let (hash, after_handle_meta) =
            self.handle
                .block_on(self.client.put_file_hash_blake3_with_retries(
                    &file_url,
                    fs_path,
                    size,
                    self.max_attempts,
                ))?;
        let after_handle = after_handle_meta.map(|m| fingerprint_for_meta(&m));
        Ok((hash, after_handle))
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_raw_tree(
    stage_dir: &Path,
    source: &FilesystemSource,
    read_mapping: Option<&super::FilesystemReadMapping>,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    consistency: &mut SourceConsistencyTracker,
    mut progress: Option<&mut super::FilesystemBuildProgressCtx<'_>>,
) -> Result<RawTreeBuildStats, anyhow::Error> {
    let mut sink = LocalDataSink::new(stage_dir)?;
    write_raw_tree_to_sink(
        &mut sink,
        source,
        read_mapping,
        entries_writer,
        entries_count,
        issues,
        consistency,
        super::reborrow_progress(&mut progress),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_raw_tree_webdav_direct(
    cfg: &super::RawTreeWebdavDirectUploadConfig,
    job_id: &str,
    run_id: &str,
    source: &FilesystemSource,
    read_mapping: Option<&super::FilesystemReadMapping>,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    consistency: &mut SourceConsistencyTracker,
    mut progress: Option<&mut super::FilesystemBuildProgressCtx<'_>>,
) -> Result<RawTreeBuildStats, anyhow::Error> {
    let mut sink = WebdavDataSink::new(
        cfg.handle.clone(),
        cfg.base_url.as_str(),
        cfg.credentials.clone(),
        job_id,
        run_id,
        cfg.max_attempts,
        cfg.resume_by_size,
    )?;
    write_raw_tree_to_sink(
        &mut sink,
        source,
        read_mapping,
        entries_writer,
        entries_count,
        issues,
        consistency,
        super::reborrow_progress(&mut progress),
    )
}

#[allow(clippy::too_many_arguments)]
fn write_raw_tree_to_sink(
    sink: &mut dyn RawTreeDataSink,
    source: &FilesystemSource,
    read_mapping: Option<&super::FilesystemReadMapping>,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    consistency: &mut SourceConsistencyTracker,
    mut progress: Option<&mut super::FilesystemBuildProgressCtx<'_>>,
) -> Result<RawTreeBuildStats, anyhow::Error> {
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

            let fs_path = match read_mapping {
                Some(mapping) => mapping.map_path(p.as_path())?,
                None => p.clone(),
            };
            write_source_entry(
                sink,
                fs_path.as_path(),
                p.as_path(),
                source,
                &exclude,
                &include,
                has_includes,
                follow_links,
                entries_writer,
                entries_count,
                issues,
                consistency,
                &mut stats,
                &mut hardlink_index,
                &mut seen_archive_paths,
                super::reborrow_progress(&mut progress),
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
        let fs_root = match read_mapping {
            Some(mapping) => mapping.map_path(root.as_path())?,
            None => root.clone(),
        };
        write_legacy_root(
            sink,
            fs_root.as_path(),
            source,
            &exclude,
            &include,
            has_includes,
            follow_links,
            entries_writer,
            entries_count,
            issues,
            consistency,
            &mut stats,
            &mut hardlink_index,
            &mut seen_archive_paths,
            super::reborrow_progress(&mut progress),
        )?;
    }

    Ok(stats)
}

#[allow(clippy::too_many_arguments)]
fn write_legacy_root(
    sink: &mut dyn RawTreeDataSink,
    root: &Path,
    source: &FilesystemSource,
    exclude: &globset::GlobSet,
    include: &globset::GlobSet,
    has_includes: bool,
    follow_links: bool,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    consistency: &mut SourceConsistencyTracker,
    stats: &mut RawTreeBuildStats,
    hardlink_index: &mut HashMap<FileId, String>,
    seen_archive_paths: &mut HashSet<String>,
    mut progress: Option<&mut super::FilesystemBuildProgressCtx<'_>>,
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
                sink,
                root,
                name,
                &meta,
                is_symlink_path,
                source,
                entries_writer,
                entries_count,
                issues,
                consistency,
                stats,
                hardlink_index,
                seen_archive_paths,
                super::reborrow_progress(&mut progress),
            )?;
        } else {
            write_symlink_entry(
                root,
                name,
                source,
                entries_writer,
                entries_count,
                issues,
                seen_archive_paths,
                super::reborrow_progress(&mut progress),
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
                sink,
                entry.path(),
                &archive_path,
                &meta,
                is_symlink_path,
                source,
                entries_writer,
                entries_count,
                issues,
                consistency,
                stats,
                hardlink_index,
                seen_archive_paths,
                super::reborrow_progress(&mut progress),
            )?;
            continue;
        }

        if entry.file_type().is_dir() {
            write_dir_entry(
                sink,
                entry.path(),
                &archive_path,
                source,
                entries_writer,
                entries_count,
                issues,
                seen_archive_paths,
                super::reborrow_progress(&mut progress),
            )?;
            continue;
        }

        if entry.file_type().is_symlink() {
            write_symlink_entry(
                entry.path(),
                &archive_path,
                source,
                entries_writer,
                entries_count,
                issues,
                seen_archive_paths,
                super::reborrow_progress(&mut progress),
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
    sink: &mut dyn RawTreeDataSink,
    fs_path: &Path,
    archive_path_basis: &Path,
    source: &FilesystemSource,
    exclude: &globset::GlobSet,
    include: &globset::GlobSet,
    has_includes: bool,
    follow_links: bool,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    consistency: &mut SourceConsistencyTracker,
    stats: &mut RawTreeBuildStats,
    hardlink_index: &mut HashMap<FileId, String>,
    seen_archive_paths: &mut HashSet<String>,
    mut progress: Option<&mut super::FilesystemBuildProgressCtx<'_>>,
) -> Result<(), anyhow::Error> {
    let prefix = match archive_prefix_for_path(archive_path_basis) {
        Ok(v) => v,
        Err(error) => {
            let msg = format!(
                "archive path error: {}: {error:#}",
                archive_path_basis.display()
            );
            if source.error_policy == FsErrorPolicy::FailFast {
                return Err(anyhow::anyhow!(msg));
            }
            issues.record_error(msg);
            return Ok(());
        }
    };
    let meta = match source_meta_for_policy(fs_path, source.symlink_policy) {
        Ok(m) => m,
        Err(error) => {
            let msg = format!("metadata error: {}: {error}", fs_path.display());
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
                sink,
                fs_path,
                &prefix,
                source,
                entries_writer,
                entries_count,
                issues,
                seen_archive_paths,
                super::reborrow_progress(&mut progress),
            )?;
        }

        let mut iter = WalkDir::new(fs_path).follow_links(follow_links).into_iter();
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
            if entry.path() == fs_path {
                continue;
            }

            let rel = match entry.path().strip_prefix(fs_path) {
                Ok(v) => v,
                Err(error) => {
                    let msg = format!(
                        "path error: {} is not under root {}: {error}",
                        entry.path().display(),
                        fs_path.display()
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
                    sink,
                    entry.path(),
                    &archive_path,
                    &meta,
                    is_symlink_path,
                    source,
                    entries_writer,
                    entries_count,
                    issues,
                    consistency,
                    stats,
                    hardlink_index,
                    seen_archive_paths,
                    super::reborrow_progress(&mut progress),
                )?;
                continue;
            }

            if entry.file_type().is_dir() {
                write_dir_entry(
                    sink,
                    entry.path(),
                    &archive_path,
                    source,
                    entries_writer,
                    entries_count,
                    issues,
                    seen_archive_paths,
                    super::reborrow_progress(&mut progress),
                )?;
                continue;
            }

            if entry.file_type().is_symlink() {
                write_symlink_entry(
                    entry.path(),
                    &archive_path,
                    source,
                    entries_writer,
                    entries_count,
                    issues,
                    seen_archive_paths,
                    super::reborrow_progress(&mut progress),
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
            archive_path_basis.display()
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
        let target = std::fs::read_link(fs_path)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "<unknown>".to_string());
        issues.record_warning(format!("skipped symlink: {archive_path} -> {target}"));
        return Ok(());
    }

    let is_symlink_path = std::fs::symlink_metadata(fs_path)
        .ok()
        .is_some_and(|m| m.file_type().is_symlink());

    if meta.is_file() {
        if has_includes && !include.is_match(&archive_path) {
            return Ok(());
        }
        write_file_entry(
            sink,
            fs_path,
            &archive_path,
            &meta,
            is_symlink_path,
            source,
            entries_writer,
            entries_count,
            issues,
            consistency,
            stats,
            hardlink_index,
            seen_archive_paths,
            super::reborrow_progress(&mut progress),
        )?;
        return Ok(());
    }

    if meta.file_type().is_symlink() {
        write_symlink_entry(
            fs_path,
            &archive_path,
            source,
            entries_writer,
            entries_count,
            issues,
            seen_archive_paths,
            super::reborrow_progress(&mut progress),
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
    sink: &mut dyn RawTreeDataSink,
    fs_path: &Path,
    archive_path: &str,
    meta: &std::fs::Metadata,
    is_symlink_path: bool,
    source: &FilesystemSource,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    consistency: &mut SourceConsistencyTracker,
    stats: &mut RawTreeBuildStats,
    hardlink_index: &mut HashMap<FileId, String>,
    seen_archive_paths: &mut HashSet<String>,
    progress: Option<&mut super::FilesystemBuildProgressCtx<'_>>,
) -> Result<(), anyhow::Error> {
    if seen_archive_paths.contains(archive_path) {
        issues.record_warning(format!("duplicate archive path (file): {archive_path}"));
        return Ok(());
    }

    let size = meta.len();
    let before_fp = fingerprint_for_meta(meta);

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

    let (hash, after_handle_fp) = match sink.store_file_hashing_blake3(fs_path, archive_path, size)
    {
        Ok(v) => v,
        Err(error) => {
            let msg = format!("store file error: {archive_path}: {error}");
            if source.error_policy == FsErrorPolicy::FailFast {
                return Err(anyhow::anyhow!(msg));
            }
            issues.record_error(msg);
            let after_path_fp = source_meta_for_policy(fs_path, source.symlink_policy)
                .ok()
                .map(|m| fingerprint_for_meta(&m));
            consistency.record_read_error(
                archive_path,
                error.to_string(),
                Some(before_fp),
                None,
                after_path_fp,
            );
            return Ok(());
        }
    };

    stats.data_files = stats.data_files.saturating_add(1);
    stats.data_bytes = stats.data_bytes.saturating_add(size);

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
        progress,
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_dir_entry(
    sink: &mut dyn RawTreeDataSink,
    fs_path: &Path,
    archive_path: &str,
    source: &FilesystemSource,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    seen_archive_paths: &mut HashSet<String>,
    progress: Option<&mut super::FilesystemBuildProgressCtx<'_>>,
) -> Result<(), anyhow::Error> {
    if seen_archive_paths.contains(archive_path) {
        issues.record_warning(format!("duplicate archive path (dir): {archive_path}"));
        return Ok(());
    }

    if let Err(error) = sink.ensure_dir(archive_path) {
        let msg = format!("create dir error: {archive_path}: {error}");
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
        progress,
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_symlink_entry(
    fs_path: &Path,
    archive_path: &str,
    source: &FilesystemSource,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    seen_archive_paths: &mut HashSet<String>,
    progress: Option<&mut super::FilesystemBuildProgressCtx<'_>>,
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
        progress,
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

struct CopyFileAndHashResult {
    written: u64,
    hash: String,
    after_handle: Option<FileFingerprintV2>,
}

fn copy_file_and_hash(src: &Path, dst: &Path) -> Result<CopyFileAndHashResult, anyhow::Error> {
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

    let after_handle = input.metadata().ok().map(|m| fingerprint_for_meta(&m));

    let hash = hasher.finalize().to_hex().to_string();
    Ok(CopyFileAndHashResult {
        written,
        hash,
        after_handle,
    })
}

fn hash_file_and_fingerprint(
    src: &Path,
) -> Result<(String, Option<FileFingerprintV2>), anyhow::Error> {
    let mut input = File::open(src)?;

    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; 1024 * 1024];

    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let after_handle = input.metadata().ok().map(|m| fingerprint_for_meta(&m));
    let hash = hasher.finalize().to_hex().to_string();
    Ok((hash, after_handle))
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
