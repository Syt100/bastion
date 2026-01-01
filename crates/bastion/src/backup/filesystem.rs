use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

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
use crate::job_spec::{FilesystemSource, FsErrorPolicy, FsHardlinkPolicy, FsSymlinkPolicy};

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
    info!(
        job_id = %job_id,
        run_id = %run_id,
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

    let root = PathBuf::from(source.root.trim());

    #[cfg(not(unix))]
    if source.hardlink_policy == FsHardlinkPolicy::Keep {
        issues.record_warning(
            "hardlink_policy=keep is not supported on this platform; storing as copies",
        );
    }

    let mut hardlink_index = HashMap::<FileId, HardlinkRecord>::new();

    let follow_links = source.symlink_policy == FsSymlinkPolicy::Follow;
    let mut iter = WalkDir::new(&root).follow_links(follow_links).into_iter();
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

        let rel = match entry.path().strip_prefix(&root) {
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
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if rel_str.is_empty() {
            continue;
        }

        let is_dir = entry.file_type().is_dir();
        if exclude.is_match(&rel_str) || (is_dir && exclude.is_match(format!("{rel_str}/"))) {
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
            issues.record_warning(format!("skipped symlink: {rel_str} -> {target}"));
            continue;
        }

        if entry.file_type().is_file() {
            if has_includes && !include.is_match(&rel_str) {
                continue;
            }

            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(error) => {
                    let msg = format!("metadata error: {rel_str}: {error}");
                    if source.error_policy == FsErrorPolicy::FailFast {
                        return Err(anyhow::anyhow!(msg));
                    }
                    issues.record_error(msg);
                    continue;
                }
            };

            let size = meta.len();

            if source.hardlink_policy == FsHardlinkPolicy::Keep
                && !is_symlink_path
                && hardlink_candidate(&meta)
                && let Some(id) = file_id_for_meta(&meta)
            {
                if let Some(existing) = hardlink_index.get(&id) {
                    let mut header = Header::new_gnu();
                    header.set_metadata_in_mode(&meta, HeaderMode::Complete);
                    header.set_entry_type(EntryType::hard_link());
                    header.set_size(0);

                    if let Err(error) = tar.append_link(
                        &mut header,
                        Path::new(&rel_str),
                        Path::new(&existing.first_path),
                    ) {
                        let msg = format!("archive error (hardlink): {rel_str}: {error}");
                        if source.error_policy == FsErrorPolicy::FailFast {
                            return Err(anyhow::anyhow!(msg));
                        }
                        issues.record_error(msg);
                        continue;
                    }

                    write_entry_record(
                        entries_writer,
                        entries_count,
                        EntryRecord {
                            path: rel_str,
                            kind: "file".to_string(),
                            size: existing.size,
                            hash_alg: Some(HashAlgorithm::Blake3),
                            hash: Some(existing.hash.clone()),
                        },
                    )?;
                    continue;
                }

                let hash = match hash_file(entry.path()) {
                    Ok(h) => h,
                    Err(error) => {
                        let msg = format!("hash error: {rel_str}: {error}");
                        if source.error_policy == FsErrorPolicy::FailFast {
                            return Err(anyhow::anyhow!(msg));
                        }
                        issues.record_error(msg);
                        continue;
                    }
                };
                hardlink_index.insert(
                    id,
                    HardlinkRecord {
                        first_path: rel_str.clone(),
                        size,
                        hash: hash.clone(),
                    },
                );

                if let Err(error) = tar.append_path_with_name(entry.path(), Path::new(&rel_str)) {
                    let msg = format!("archive error: {rel_str}: {error}");
                    if source.error_policy == FsErrorPolicy::FailFast {
                        return Err(anyhow::anyhow!(msg));
                    }
                    issues.record_error(msg);
                    continue;
                }

                write_entry_record(
                    entries_writer,
                    entries_count,
                    EntryRecord {
                        path: rel_str,
                        kind: "file".to_string(),
                        size,
                        hash_alg: Some(HashAlgorithm::Blake3),
                        hash: Some(hash),
                    },
                )?;
                continue;
            }

            let hash = match hash_file(entry.path()) {
                Ok(h) => h,
                Err(error) => {
                    let msg = format!("hash error: {rel_str}: {error}");
                    if source.error_policy == FsErrorPolicy::FailFast {
                        return Err(anyhow::anyhow!(msg));
                    }
                    issues.record_error(msg);
                    continue;
                }
            };

            if let Err(error) = tar.append_path_with_name(entry.path(), Path::new(&rel_str)) {
                let msg = format!("archive error: {rel_str}: {error}");
                if source.error_policy == FsErrorPolicy::FailFast {
                    return Err(anyhow::anyhow!(msg));
                }
                issues.record_error(msg);
                continue;
            }

            write_entry_record(
                entries_writer,
                entries_count,
                EntryRecord {
                    path: rel_str,
                    kind: "file".to_string(),
                    size,
                    hash_alg: Some(HashAlgorithm::Blake3),
                    hash: Some(hash),
                },
            )?;
            continue;
        }

        if entry.file_type().is_dir() {
            if let Err(error) = tar.append_path_with_name(entry.path(), Path::new(&rel_str)) {
                let msg = format!("archive error: {rel_str}: {error}");
                if source.error_policy == FsErrorPolicy::FailFast {
                    return Err(anyhow::anyhow!(msg));
                }
                issues.record_error(msg);
                iter.skip_current_dir();
                continue;
            }

            write_entry_record(
                entries_writer,
                entries_count,
                EntryRecord {
                    path: rel_str,
                    kind: "dir".to_string(),
                    size: 0,
                    hash_alg: None,
                    hash: None,
                },
            )?;
            continue;
        }

        if entry.file_type().is_symlink() {
            if let Err(error) = tar.append_path_with_name(entry.path(), Path::new(&rel_str)) {
                let msg = format!("archive error: {rel_str}: {error}");
                if source.error_policy == FsErrorPolicy::FailFast {
                    return Err(anyhow::anyhow!(msg));
                }
                issues.record_error(msg);
                continue;
            }

            write_entry_record(
                entries_writer,
                entries_count,
                EntryRecord {
                    path: rel_str,
                    kind: "symlink".to_string(),
                    size: 0,
                    hash_alg: None,
                    hash: None,
                },
            )?;
            continue;
        }

        let msg = format!("unsupported file type: {rel_str}");
        if source.error_policy == FsErrorPolicy::FailFast {
            return Err(anyhow::anyhow!(msg));
        }
        issues.record_error(msg);
    }

    Ok(())
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
