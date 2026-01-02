use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use bastion_core::manifest::{EntryIndexRef, HashAlgorithm, ManifestV1, PipelineSettings};
use serde::Serialize;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tracing::info;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::backup::{
    COMPLETE_NAME, ENTRIES_INDEX_NAME, LocalArtifact, LocalRunArtifacts, MANIFEST_NAME, PartWriter,
    PayloadEncryption, stage_dir,
};
use bastion_core::job_spec::VaultwardenSource;

#[derive(Debug, Serialize)]
struct EntryRecord {
    path: String,
    kind: String,
    size: u64,
    hash_alg: Option<HashAlgorithm>,
    hash: Option<String>,
}

pub fn build_vaultwarden_run(
    data_dir: &Path,
    job_id: &str,
    run_id: &str,
    started_at: OffsetDateTime,
    source: &VaultwardenSource,
    encryption: &PayloadEncryption,
    part_size_bytes: u64,
) -> Result<LocalRunArtifacts, anyhow::Error> {
    info!(
        job_id = %job_id,
        run_id = %run_id,
        vw_data_dir = %source.data_dir,
        encryption = ?encryption,
        part_size_bytes,
        "building vaultwarden backup artifacts"
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

    let root = PathBuf::from(source.data_dir.trim());
    if root.as_os_str().is_empty() {
        anyhow::bail!("vaultwarden.source.data_dir is required");
    }

    let run_dir = crate::backup::run_dir(data_dir, run_id);
    let source_dir = run_dir.join("source");
    std::fs::create_dir_all(&source_dir)?;

    let source_db_path = root.join("db.sqlite3");
    let snapshot_path = source_dir.join("db.sqlite3");
    crate::backup::sqlite::create_snapshot(&source_db_path.to_string_lossy(), &snapshot_path)?;
    let snapshot_size = std::fs::metadata(&snapshot_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let parts = write_tar_zstd_parts(
        &stage,
        &root,
        &snapshot_path,
        encryption,
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
        snapshot_size,
        "built vaultwarden backup artifacts"
    );

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
    root: &Path,
    snapshot_path: &Path,
    encryption: &PayloadEncryption,
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

    // Add the SQLite snapshot as db.sqlite3 at the root of the archive.
    tar.append_path_with_name(snapshot_path, Path::new("db.sqlite3"))?;
    let snapshot_size = std::fs::metadata(snapshot_path)?.len();
    let snapshot_hash = hash_file(snapshot_path)?;
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
    use super::build_vaultwarden_run;
    use crate::backup::PayloadEncryption;
    use bastion_core::job_spec::VaultwardenSource;
    use rusqlite::Connection;
    use std::fs;
    use tempfile::tempdir;
    use time::OffsetDateTime;
    use uuid::Uuid;

    fn read_entries(paths: &std::path::Path) -> Vec<serde_json::Value> {
        let raw = fs::read(paths).unwrap();
        let decoded = zstd::decode_all(std::io::Cursor::new(raw)).unwrap();
        decoded
            .split(|b| *b == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice::<serde_json::Value>(line).unwrap())
            .collect()
    }

    #[test]
    fn vaultwarden_run_includes_snapshot_and_files() {
        let tmp = tempdir().unwrap();
        let data_dir = tmp.path().join("data");
        fs::create_dir_all(&data_dir).unwrap();

        let vw_dir = tmp.path().join("vw");
        fs::create_dir_all(vw_dir.join("attachments")).unwrap();
        fs::write(vw_dir.join("attachments").join("hello.txt"), b"hi").unwrap();

        let db_path = vw_dir.join("db.sqlite3");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch("CREATE TABLE foo(x INTEGER); INSERT INTO foo VALUES(42);")
            .unwrap();
        drop(conn);

        fs::write(vw_dir.join("db.sqlite3-wal"), b"ignored").unwrap();

        let job_id = Uuid::new_v4().to_string();
        let run_id = Uuid::new_v4().to_string();

        let source = VaultwardenSource {
            data_dir: vw_dir.to_string_lossy().to_string(),
        };
        let encryption = PayloadEncryption::None;
        let artifacts = build_vaultwarden_run(
            &data_dir,
            &job_id,
            &run_id,
            OffsetDateTime::now_utc(),
            &source,
            &encryption,
            4 * 1024 * 1024,
        )
        .unwrap();

        let entries = read_entries(&artifacts.entries_index_path);
        let paths: Vec<String> = entries
            .iter()
            .filter_map(|v| {
                v.get("path")
                    .and_then(|p| p.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        assert!(paths.contains(&"attachments/hello.txt".to_string()));
        assert!(paths.contains(&"db.sqlite3".to_string()));
        assert!(!paths.contains(&"db.sqlite3-wal".to_string()));

        let snapshot_path = artifacts.run_dir.join("source").join("db.sqlite3");
        let snapshot_conn = Connection::open(snapshot_path).unwrap();
        let n: i64 = snapshot_conn
            .query_row("SELECT x FROM foo", [], |row| row.get(0))
            .unwrap();
        assert_eq!(n, 42);
    }
}
