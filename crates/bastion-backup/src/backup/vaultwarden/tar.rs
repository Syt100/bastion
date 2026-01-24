use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use bastion_core::manifest::HashAlgorithm;
use serde::Serialize;
use walkdir::WalkDir;

use crate::backup::{LocalArtifact, PartWriter, PayloadEncryption};

#[derive(Debug, Serialize)]
struct EntryRecord {
    path: String,
    kind: String,
    size: u64,
    hash_alg: Option<HashAlgorithm>,
    hash: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_tar_zstd_parts(
    stage_dir: &Path,
    root: &Path,
    snapshot_path: &Path,
    encryption: &PayloadEncryption,
    entries_writer: &mut zstd::Encoder<'_, BufWriter<File>>,
    entries_count: &mut u64,
    part_size_bytes: u64,
    on_part_finished: Option<Box<dyn Fn(LocalArtifact) -> std::io::Result<()> + Send>>,
) -> Result<Vec<LocalArtifact>, anyhow::Error> {
    let payload_prefix: &'static str = "payload.part";
    let mut part_writer =
        PartWriter::new(stage_dir.to_path_buf(), part_size_bytes, payload_prefix)?;
    if let Some(cb) = on_part_finished {
        part_writer.set_on_part_finished(cb);
    }

    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    match encryption {
        PayloadEncryption::None => {
            let mut encoder = zstd::Encoder::new(&mut part_writer, 3)?;
            encoder.multithread(threads as u32)?;

            let mut tar = tar::Builder::new(encoder);
            write_vaultwarden_tar_entries(
                &mut tar,
                root,
                snapshot_path,
                entries_writer,
                entries_count,
            )?;

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
            write_vaultwarden_tar_entries(
                &mut tar,
                root,
                snapshot_path,
                entries_writer,
                entries_count,
            )?;

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

fn write_vaultwarden_tar_entries<W: Write>(
    tar: &mut tar::Builder<W>,
    root: &Path,
    snapshot_path: &Path,
    entries_writer: &mut zstd::Encoder<'_, BufWriter<File>>,
    entries_count: &mut u64,
) -> Result<(), anyhow::Error> {
    for next in WalkDir::new(root).follow_links(false).into_iter() {
        let entry = next?;
        if entry.path() == root {
            continue;
        }

        let rel = entry.path().strip_prefix(root)?;
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        if rel_str.is_empty() {
            continue;
        }

        if entry.file_type().is_file()
            && matches!(
                rel_str.as_str(),
                "db.sqlite3" | "db.sqlite3-wal" | "db.sqlite3-shm" | "db.sqlite3-journal"
            )
        {
            continue;
        }

        tar.append_path_with_name(entry.path(), Path::new(&rel_str))?;

        let record = if entry.file_type().is_file() {
            let size = entry.metadata()?.len();
            let hash = super::hash::hash_file(entry.path())?;
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

    // Add the SQLite snapshot as db.sqlite3 at the root of the archive.
    tar.append_path_with_name(snapshot_path, Path::new("db.sqlite3"))?;
    let snapshot_size = std::fs::metadata(snapshot_path)?.len();
    let snapshot_hash = super::hash::hash_file(snapshot_path)?;
    let record = EntryRecord {
        path: "db.sqlite3".to_string(),
        kind: "file".to_string(),
        size: snapshot_size,
        hash_alg: Some(HashAlgorithm::Blake3),
        hash: Some(snapshot_hash),
    };
    let line = serde_json::to_vec(&record)?;
    entries_writer.write_all(&line)?;
    entries_writer.write_all(b"\n")?;
    *entries_count += 1;

    Ok(())
}
