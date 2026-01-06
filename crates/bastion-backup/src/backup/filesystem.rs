use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::{Component, Path, PathBuf};

use bastion_core::manifest::{EntryIndexRef, HashAlgorithm, ManifestV1, PipelineSettings};
use serde::Serialize;
use tar::{EntryType, Header, HeaderMode};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tracing::info;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::backup::{
    COMPLETE_NAME, ENTRIES_INDEX_NAME, LocalArtifact, LocalRunArtifacts, MANIFEST_NAME, PartWriter,
    PayloadEncryption, stage_dir,
};
use bastion_core::job_spec::{FilesystemSource, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy};

#[derive(Debug, Serialize)]
struct EntryRecord {
    path: String,
    kind: String,
    size: u64,
    hash_alg: Option<HashAlgorithm>,
    hash: Option<String>,
}

const MAX_FS_ISSUE_SAMPLES: usize = 50;

#[derive(Debug, Default)]
pub struct FilesystemBuildIssues {
    pub warnings_total: u64,
    pub errors_total: u64,
    pub sample_warnings: Vec<String>,
    pub sample_errors: Vec<String>,
}

impl FilesystemBuildIssues {
    fn record_warning(&mut self, msg: impl Into<String>) {
        self.warnings_total = self.warnings_total.saturating_add(1);
        if self.sample_warnings.len() < MAX_FS_ISSUE_SAMPLES {
            self.sample_warnings.push(msg.into());
        }
    }

    fn record_error(&mut self, msg: impl Into<String>) {
        self.errors_total = self.errors_total.saturating_add(1);
        if self.sample_errors.len() < MAX_FS_ISSUE_SAMPLES {
            self.sample_errors.push(msg.into());
        }
    }
}

#[derive(Debug)]
pub struct FilesystemRunBuild {
    pub artifacts: LocalRunArtifacts,
    pub issues: FilesystemBuildIssues,
}

pub fn build_filesystem_run(
    data_dir: &Path,
    job_id: &str,
    run_id: &str,
    started_at: OffsetDateTime,
    source: &FilesystemSource,
    encryption: &PayloadEncryption,
    part_size_bytes: u64,
) -> Result<FilesystemRunBuild, anyhow::Error> {
    let using_paths = source.paths.iter().any(|p| !p.trim().is_empty());
    info!(
        job_id = %job_id,
        run_id = %run_id,
        using_paths,
        paths_count = source.paths.len(),
        root = %source.root,
        include_rules = source.include.len(),
        exclude_rules = source.exclude.len(),
        symlink_policy = ?source.symlink_policy,
        hardlink_policy = ?source.hardlink_policy,
        error_policy = ?source.error_policy,
        encryption = ?encryption,
        part_size_bytes,
        "building filesystem backup artifacts"
    );

    let stage = stage_dir(data_dir, run_id);
    std::fs::create_dir_all(&stage)?;

    let entries_path = stage.join(ENTRIES_INDEX_NAME);
    let entries_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(entries_path)?;
    let entries_writer = BufWriter::new(entries_file);
    let mut entries_writer = zstd::Encoder::new(entries_writer, 3)?;
    let mut entries_count = 0u64;
    let mut issues = FilesystemBuildIssues::default();

    let parts = write_tar_zstd_parts(
        &stage,
        source,
        encryption,
        &mut entries_writer,
        &mut entries_count,
        part_size_bytes,
        &mut issues,
    )?;
    entries_writer.finish()?;

    let ended_at = OffsetDateTime::now_utc();

    let job_uuid = Uuid::parse_str(job_id)?;
    let run_uuid = Uuid::parse_str(run_id)?;

    let manifest = ManifestV1 {
        format_version: ManifestV1::FORMAT_VERSION,
        job_id: job_uuid,
        run_id: run_uuid,
        started_at: started_at.format(&Rfc3339)?,
        ended_at: ended_at.format(&Rfc3339)?,
        pipeline: PipelineSettings {
            tar: "pax".to_string(),
            compression: "zstd".to_string(),
            encryption: match encryption {
                PayloadEncryption::None => "none".to_string(),
                PayloadEncryption::AgeX25519 { .. } => "age".to_string(),
            },
            encryption_key: match encryption {
                PayloadEncryption::None => None,
                PayloadEncryption::AgeX25519 { key_name, .. } => Some(key_name.clone()),
            },
            split_bytes: part_size_bytes,
        },
        artifacts: parts
            .iter()
            .map(|p| bastion_core::manifest::ArtifactPart {
                name: p.name.clone(),
                size: p.size,
                hash_alg: p.hash_alg.clone(),
                hash: p.hash.clone(),
            })
            .collect(),
        entry_index: EntryIndexRef {
            name: ENTRIES_INDEX_NAME.to_string(),
            count: entries_count,
        },
    };

    let manifest_path = stage.join(MANIFEST_NAME);
    let complete_path = stage.join(COMPLETE_NAME);

    write_json(&manifest_path, &manifest)?;
    write_json(&complete_path, &serde_json::json!({}))?;

    let parts_count = parts.len();
    let parts_bytes: u64 = parts.iter().map(|p| p.size).sum();
    info!(
        job_id = %job_id,
        run_id = %run_id,
        entries_count,
        parts_count,
        parts_bytes,
        warnings_total = issues.warnings_total,
        errors_total = issues.errors_total,
        "built filesystem backup artifacts"
    );

    Ok(FilesystemRunBuild {
        artifacts: LocalRunArtifacts {
            run_dir: stage.parent().unwrap_or(&stage).to_path_buf(),
            parts,
            entries_index_path: stage.join(ENTRIES_INDEX_NAME),
            entries_count,
            manifest_path,
            complete_path,
        },
        issues,
    })
}

fn write_tar_zstd_parts(
    stage_dir: &Path,
    source: &FilesystemSource,
    encryption: &PayloadEncryption,
    entries_writer: &mut zstd::Encoder<'_, BufWriter<File>>,
    entries_count: &mut u64,
    part_size_bytes: u64,
    issues: &mut FilesystemBuildIssues,
) -> Result<Vec<LocalArtifact>, anyhow::Error> {
    let payload_prefix: &'static str = "payload.part";
    let mut part_writer =
        PartWriter::new(stage_dir.to_path_buf(), part_size_bytes, payload_prefix)?;

    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    match encryption {
        PayloadEncryption::None => {
            let mut encoder = zstd::Encoder::new(&mut part_writer, 3)?;
            encoder.multithread(threads as u32)?;

            let mut tar = tar::Builder::new(encoder);
            write_tar_entries(&mut tar, source, entries_writer, entries_count, issues)?;

            tar.finish()?;
            let encoder = tar.into_inner()?;
            encoder.finish()?;
        }
        PayloadEncryption::AgeX25519 { recipient, .. } => {
            use std::str::FromStr as _;

            let recipient =
                age::x25519::Recipient::from_str(recipient).map_err(|e| anyhow::anyhow!(e))?;
            let encryptor = age::Encryptor::with_recipients(std::iter::once(
                &recipient as &dyn age::Recipient,
            ))?;
            let encrypted = encryptor.wrap_output(&mut part_writer)?;

            let mut encoder = zstd::Encoder::new(encrypted, 3)?;
            encoder.multithread(threads as u32)?;

            let mut tar = tar::Builder::new(encoder);
            write_tar_entries(&mut tar, source, entries_writer, entries_count, issues)?;

            tar.finish()?;
            let encoder = tar.into_inner()?;
            let encrypted = encoder.finish()?;
            encrypted.finish()?;
        }
    }
    entries_writer.flush()?;

    let parts = part_writer.finish()?;
    let local_parts = parts
        .into_iter()
        .map(|p| LocalArtifact {
            name: p.name.clone(),
            path: stage_dir.join(&p.name),
            size: p.size,
            hash_alg: p.hash_alg,
            hash: p.hash,
        })
        .collect();

    Ok(local_parts)
}

fn write_tar_entries<W: Write>(
    tar: &mut tar::Builder<W>,
    source: &FilesystemSource,
    entries_writer: &mut zstd::Encoder<'_, BufWriter<File>>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
) -> Result<(), anyhow::Error> {
    tar.follow_symlinks(source.symlink_policy == FsSymlinkPolicy::Follow);

    let exclude = compile_globset(&source.exclude)?;
    let include = compile_globset(&source.include)?;
    let has_includes = !source.include.is_empty();

    #[cfg(not(unix))]
    if source.hardlink_policy == FsHardlinkPolicy::Keep {
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
    tar: &mut tar::Builder<W>,
    path: &Path,
    source: &FilesystemSource,
    exclude: &globset::GlobSet,
    include: &globset::GlobSet,
    has_includes: bool,
    follow_links: bool,
    entries_writer: &mut zstd::Encoder<'_, BufWriter<File>>,
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
        let msg = format!("invalid source path: {} has no archive path", path.display());
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
    tar: &mut tar::Builder<W>,
    root: &Path,
    source: &FilesystemSource,
    exclude: &globset::GlobSet,
    include: &globset::GlobSet,
    has_includes: bool,
    follow_links: bool,
    entries_writer: &mut zstd::Encoder<'_, BufWriter<File>>,
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

#[allow(clippy::too_many_arguments)]
fn write_file_entry<W: Write>(
    tar: &mut tar::Builder<W>,
    fs_path: &Path,
    archive_path: &str,
    meta: &std::fs::Metadata,
    is_symlink_path: bool,
    source: &FilesystemSource,
    entries_writer: &mut zstd::Encoder<'_, BufWriter<File>>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    hardlink_index: &mut HashMap<FileId, HardlinkRecord>,
    seen_archive_paths: &mut HashSet<String>,
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
            let mut header = Header::new_gnu();
            header.set_metadata_in_mode(meta, HeaderMode::Complete);
            header.set_entry_type(EntryType::hard_link());
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
                },
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
            },
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
        },
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_dir_entry<W: Write>(
    tar: &mut tar::Builder<W>,
    fs_path: &Path,
    archive_path: &str,
    source: &FilesystemSource,
    entries_writer: &mut zstd::Encoder<'_, BufWriter<File>>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    seen_archive_paths: &mut HashSet<String>,
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
        },
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_symlink_entry<W: Write>(
    tar: &mut tar::Builder<W>,
    fs_path: &Path,
    archive_path: &str,
    source: &FilesystemSource,
    entries_writer: &mut zstd::Encoder<'_, BufWriter<File>>,
    entries_count: &mut u64,
    issues: &mut FilesystemBuildIssues,
    seen_archive_paths: &mut HashSet<String>,
) -> Result<(), anyhow::Error> {
    if seen_archive_paths.contains(archive_path) {
        issues.record_warning(format!(
            "duplicate archive path (symlink): {archive_path}"
        ));
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
        },
    )?;
    Ok(())
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

fn join_archive_path(prefix: &str, rel: &str) -> String {
    if prefix.is_empty() {
        rel.to_string()
    } else if rel.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}/{rel}")
    }
}

fn archive_prefix_for_path(path: &Path) -> Result<String, anyhow::Error> {
    let mut components = Vec::<String>::new();
    for comp in path.components() {
        match comp {
            Component::Prefix(prefix) => {
                #[cfg(windows)]
                {
                use std::path::Prefix as P;
                match prefix.kind() {
                    P::Disk(letter) | P::VerbatimDisk(letter) => {
                        components.push((letter as char).to_ascii_uppercase().to_string());
                    }
                    P::UNC(server, share) | P::VerbatimUNC(server, share) => {
                        components.push("UNC".to_string());
                        components.push(server.to_string_lossy().to_string());
                        components.push(share.to_string_lossy().to_string());
                    }
                    _ => {
                        components.push(prefix.as_os_str().to_string_lossy().to_string());
                    }
                }
                }
                #[cfg(not(windows))]
                {
                    components.push(prefix.as_os_str().to_string_lossy().to_string());
                }
            }
            Component::RootDir => {}
            Component::CurDir => {}
            Component::ParentDir => anyhow::bail!(
                "source path must not contain '..': {}",
                path.display()
            ),
            Component::Normal(p) => {
                let s = p.to_string_lossy();
                if s.is_empty() {
                    continue;
                }
                components.push(s.to_string());
            }
        }
    }

    Ok(components.join("/"))
}

fn write_entry_record(
    entries_writer: &mut zstd::Encoder<'_, BufWriter<File>>,
    entries_count: &mut u64,
    record: EntryRecord,
) -> Result<(), anyhow::Error> {
    let line = serde_json::to_vec(&record)?;
    entries_writer.write_all(&line)?;
    entries_writer.write_all(b"\n")?;
    *entries_count += 1;
    Ok(())
}

fn compile_globset(patterns: &[String]) -> Result<globset::GlobSet, anyhow::Error> {
    let mut builder = globset::GlobSetBuilder::new();
    for p in patterns {
        builder.add(globset::Glob::new(p)?);
    }
    Ok(builder.build()?)
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

#[derive(Debug, Clone)]
struct HardlinkRecord {
    first_path: String,
    size: u64,
    hash: String,
}

fn hash_file(path: &Path) -> Result<String, anyhow::Error> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), anyhow::Error> {
    let bytes = serde_json::to_vec_pretty(value)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::Path;

    use tempfile::tempdir;
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::backup::PayloadEncryption;
    use bastion_core::job_spec::{
        FilesystemSource, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy,
    };

    use super::{archive_prefix_for_path, build_filesystem_run};

    fn list_tar_paths(part_path: &Path) -> Vec<String> {
        let file = File::open(part_path).expect("open part");
        let decoder = zstd::Decoder::new(file).expect("zstd decoder");
        let mut archive = tar::Archive::new(decoder);
        archive
            .entries()
            .expect("entries")
            .map(|e| {
                e.expect("entry")
                    .path()
                    .expect("path")
                    .to_string_lossy()
                    .to_string()
            })
            .collect()
    }

    fn list_index_paths(entries_path: &Path) -> Vec<String> {
        let raw = std::fs::read(entries_path).expect("read entries index");
        let decoded = zstd::decode_all(std::io::Cursor::new(raw)).expect("decode entries index");
        decoded
            .split(|b| *b == b'\n')
            .filter(|line| !line.is_empty())
            .filter_map(|line| serde_json::from_slice::<serde_json::Value>(line).ok())
            .filter_map(|v| v.get("path").and_then(|p| p.as_str()).map(|s| s.to_string()))
            .collect()
    }

    #[test]
    fn filesystem_paths_can_backup_single_file() {
        let tmp = tempdir().expect("tempdir");
        let data_dir = tmp.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let src = tmp.path().join("hello.txt");
        std::fs::write(&src, b"hi").unwrap();

        let expected = archive_prefix_for_path(&src).unwrap();

        let source = FilesystemSource {
            paths: vec![src.to_string_lossy().to_string()],
            root: String::new(),
            include: Vec::new(),
            exclude: Vec::new(),
            symlink_policy: FsSymlinkPolicy::Keep,
            hardlink_policy: FsHardlinkPolicy::Copy,
            error_policy: FsErrorPolicy::FailFast,
        };

        let build = build_filesystem_run(
            &data_dir,
            &Uuid::new_v4().to_string(),
            &Uuid::new_v4().to_string(),
            OffsetDateTime::now_utc(),
            &source,
            &PayloadEncryption::None,
            4 * 1024 * 1024,
        )
        .unwrap();
        assert_eq!(build.issues.errors_total, 0);

        let part_paths = build
            .artifacts
            .parts
            .iter()
            .map(|p| p.path.as_path())
            .collect::<Vec<_>>();
        assert_eq!(part_paths.len(), 1);
        let tar_paths = list_tar_paths(part_paths[0]);
        assert!(tar_paths.contains(&expected));

        let index_paths = list_index_paths(&build.artifacts.entries_index_path);
        assert!(index_paths.contains(&expected));
    }

    #[test]
    fn filesystem_paths_deduplicates_overlapping_sources() {
        let tmp = tempdir().expect("tempdir");
        let data_dir = tmp.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let dir = tmp.path().join("dir");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.txt");
        std::fs::write(&file, b"a").unwrap();

        let expected = format!("{}/a.txt", archive_prefix_for_path(&dir).unwrap());

        let source = FilesystemSource {
            paths: vec![
                dir.to_string_lossy().to_string(),
                file.to_string_lossy().to_string(),
            ],
            root: String::new(),
            include: Vec::new(),
            exclude: Vec::new(),
            symlink_policy: FsSymlinkPolicy::Keep,
            hardlink_policy: FsHardlinkPolicy::Copy,
            error_policy: FsErrorPolicy::FailFast,
        };

        let build = build_filesystem_run(
            &data_dir,
            &Uuid::new_v4().to_string(),
            &Uuid::new_v4().to_string(),
            OffsetDateTime::now_utc(),
            &source,
            &PayloadEncryption::None,
            4 * 1024 * 1024,
        )
        .unwrap();
        assert_eq!(build.issues.errors_total, 0);
        assert_eq!(build.issues.warnings_total, 1);
        assert!(
            build
                .issues
                .sample_warnings
                .iter()
                .any(|w| w.contains("deduplicated") && w.contains("overlapping source")),
            "missing dedupe warning: {:?}",
            build.issues.sample_warnings
        );

        let part = build.artifacts.parts[0].path.as_path();
        let tar_paths = list_tar_paths(part);
        assert_eq!(tar_paths.iter().filter(|p| *p == &expected).count(), 1);
    }

    #[test]
    fn legacy_root_can_backup_single_file() {
        let tmp = tempdir().expect("tempdir");
        let data_dir = tmp.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let src = tmp.path().join("hello.txt");
        std::fs::write(&src, b"hi").unwrap();

        let source = FilesystemSource {
            paths: Vec::new(),
            root: src.to_string_lossy().to_string(),
            include: Vec::new(),
            exclude: Vec::new(),
            symlink_policy: FsSymlinkPolicy::Keep,
            hardlink_policy: FsHardlinkPolicy::Copy,
            error_policy: FsErrorPolicy::FailFast,
        };

        let build = build_filesystem_run(
            &data_dir,
            &Uuid::new_v4().to_string(),
            &Uuid::new_v4().to_string(),
            OffsetDateTime::now_utc(),
            &source,
            &PayloadEncryption::None,
            4 * 1024 * 1024,
        )
        .unwrap();
        assert_eq!(build.issues.errors_total, 0);

        let part = build.artifacts.parts[0].path.as_path();
        let tar_paths = list_tar_paths(part);
        assert!(tar_paths.contains(&"hello.txt".to_string()));
    }
}
