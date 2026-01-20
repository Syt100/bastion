use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use bastion_core::job_spec::VaultwardenSource;
use bastion_core::manifest::{ArtifactFormatV1, EntryIndexRef, ManifestV1, PipelineSettings};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tracing::info;
use uuid::Uuid;

use crate::backup::{
    BuildPipelineOptions, COMPLETE_NAME, ENTRIES_INDEX_NAME, LocalRunArtifacts, MANIFEST_NAME,
    PayloadEncryption, stage_dir,
};

pub fn build_vaultwarden_run(
    data_dir: &Path,
    job_id: &str,
    run_id: &str,
    started_at: OffsetDateTime,
    source: &VaultwardenSource,
    pipeline: BuildPipelineOptions<'_>,
) -> Result<LocalRunArtifacts, anyhow::Error> {
    let BuildPipelineOptions {
        artifact_format,
        encryption,
        part_size_bytes,
    } = pipeline;
    info!(
        job_id = %job_id,
        run_id = %run_id,
        vw_data_dir = %source.data_dir,
        artifact_format = ?artifact_format,
        encryption = ?encryption,
        part_size_bytes,
        "building vaultwarden backup artifacts"
    );

    if artifact_format != ArtifactFormatV1::ArchiveV1 {
        anyhow::bail!("vaultwarden backups currently support only archive_v1 artifact format");
    }

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

    let parts = super::tar::write_tar_zstd_parts(
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
            format: artifact_format,
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

    super::io::write_json(&manifest_path, &manifest)?;
    super::io::write_json(&complete_path, &serde_json::json!({}))?;

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
