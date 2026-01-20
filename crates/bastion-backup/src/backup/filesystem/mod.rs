use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::Path;

use bastion_core::manifest::{ArtifactFormatV1, EntryIndexRef, ManifestV1, PipelineSettings};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tracing::info;
use uuid::Uuid;

use crate::backup::{
    COMPLETE_NAME, ENTRIES_INDEX_NAME, LocalRunArtifacts, MANIFEST_NAME, PayloadEncryption,
    stage_dir,
};
use bastion_core::job_spec::FilesystemSource;

mod entries_index;
mod tar;
mod util;

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

    let parts = tar::write_tar_zstd_parts(
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
            format: ArtifactFormatV1::ArchiveV1,
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

    util::write_json(&manifest_path, &manifest)?;
    util::write_json(&complete_path, &serde_json::json!({}))?;

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

#[cfg(test)]
mod tests;
