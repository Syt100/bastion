use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use bastion_core::manifest::{EntryIndexRef, HashAlgorithm, ManifestV1, PipelineSettings};
use serde::Serialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::backup::{
    COMPLETE_NAME, ENTRIES_INDEX_NAME, LocalArtifact, LocalRunArtifacts, MANIFEST_NAME, PartWriter,
    stage_dir,
};
use crate::job_spec::FilesystemSource;

#[derive(Debug, Serialize)]
struct EntryRecord {
    path: String,
    kind: String,
    size: u64,
    hash_alg: Option<HashAlgorithm>,
    hash: Option<String>,
}

pub fn build_filesystem_run(
    data_dir: &Path,
    job_id: &str,
    run_id: &str,
    started_at: OffsetDateTime,
    source: &FilesystemSource,
    part_size_bytes: u64,
) -> Result<LocalRunArtifacts, anyhow::Error> {
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

    let parts = write_tar_zstd_parts(
        &stage,
        source,
        &mut entries_writer,
        &mut entries_count,
        part_size_bytes,
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
            encryption: "none".to_string(),
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

    Ok(LocalRunArtifacts {
        run_dir: stage.parent().unwrap_or(&stage).to_path_buf(),
        parts,
        entries_index_path: stage.join(ENTRIES_INDEX_NAME),
        entries_count,
        manifest_path,
        complete_path,
    })
}

fn write_tar_zstd_parts(
    stage_dir: &Path,
    source: &FilesystemSource,
    entries_writer: &mut zstd::Encoder<'_, BufWriter<File>>,
    entries_count: &mut u64,
    part_size_bytes: u64,
) -> Result<Vec<LocalArtifact>, anyhow::Error> {
    let payload_prefix: &'static str = "payload.part";
    let mut part_writer =
        PartWriter::new(stage_dir.to_path_buf(), part_size_bytes, payload_prefix)?;

    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let mut encoder = zstd::Encoder::new(&mut part_writer, 3)?;
    encoder.multithread(threads as u32)?;

    let mut tar = tar::Builder::new(encoder);

    let exclude = compile_globset(&source.exclude)?;
    let include = compile_globset(&source.include)?;
    let has_includes = !source.include.is_empty();

    let root = PathBuf::from(source.root.trim());

    let mut iter = WalkDir::new(&root).follow_links(false).into_iter();
    while let Some(next) = iter.next() {
        let entry = next?;
        if entry.path() == root {
            continue;
        }

        let rel = entry.path().strip_prefix(&root)?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if rel_str.is_empty() {
            continue;
        }

        let is_dir = entry.file_type().is_dir();
        if exclude.is_match(&rel_str) || (is_dir && exclude.is_match(&format!("{rel_str}/"))) {
            if is_dir {
                iter.skip_current_dir();
            }
            continue;
        }

        if has_includes && entry.file_type().is_file() && !include.is_match(&rel_str) {
            continue;
        }

        tar.append_path_with_name(entry.path(), Path::new(&rel_str))?;

        let record = if entry.file_type().is_file() {
            let size = entry.metadata()?.len();
            let hash = hash_file(entry.path())?;
            EntryRecord {
                path: rel_str,
                kind: "file".to_string(),
                size,
                hash_alg: Some(HashAlgorithm::Blake3),
                hash: Some(hash),
            }
        } else if entry.file_type().is_dir() {
            EntryRecord {
                path: rel_str,
                kind: "dir".to_string(),
                size: 0,
                hash_alg: None,
                hash: None,
            }
        } else if entry.file_type().is_symlink() {
            EntryRecord {
                path: rel_str,
                kind: "symlink".to_string(),
                size: 0,
                hash_alg: None,
                hash: None,
            }
        } else {
            continue;
        };

        let line = serde_json::to_vec(&record)?;
        entries_writer.write_all(&line)?;
        entries_writer.write_all(b"\n")?;
        *entries_count += 1;
    }

    tar.finish()?;
    let encoder = tar.into_inner()?;
    encoder.finish()?;
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

fn compile_globset(patterns: &[String]) -> Result<globset::GlobSet, anyhow::Error> {
    let mut builder = globset::GlobSetBuilder::new();
    for p in patterns {
        builder.add(globset::Glob::new(p)?);
    }
    Ok(builder.build()?)
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
