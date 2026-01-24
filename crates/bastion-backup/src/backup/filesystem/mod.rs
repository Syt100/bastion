use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::Path;
use std::time::{Duration, Instant};

use bastion_core::manifest::{ArtifactFormatV1, EntryIndexRef, ManifestV1, PipelineSettings};
use bastion_core::progress::ProgressUnitsV1;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tracing::info;
use uuid::Uuid;

use crate::backup::{
    BuildPipelineOptions, COMPLETE_NAME, ENTRIES_INDEX_NAME, LocalArtifact, LocalRunArtifacts,
    MANIFEST_NAME, PayloadEncryption, stage_dir,
};
use bastion_core::job_spec::FilesystemSource;

mod entries_index;
mod raw_tree;
mod tar;
mod util;

const MAX_FS_ISSUE_SAMPLES: usize = 50;
const FS_PROGRESS_MIN_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone)]
pub struct FilesystemBuildProgressUpdate {
    pub stage: &'static str,
    pub done: ProgressUnitsV1,
    pub total: Option<ProgressUnitsV1>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RawTreeBuildStats {
    pub data_files: u64,
    pub data_bytes: u64,
}

struct FilesystemBuildProgressCtx<'a> {
    stage: &'static str,
    done: ProgressUnitsV1,
    total: Option<ProgressUnitsV1>,
    last_emit: Instant,
    on_progress: &'a dyn Fn(FilesystemBuildProgressUpdate),
}

impl<'a> FilesystemBuildProgressCtx<'a> {
    fn new(
        stage: &'static str,
        total: Option<ProgressUnitsV1>,
        on_progress: &'a dyn Fn(FilesystemBuildProgressUpdate),
    ) -> Self {
        Self {
            stage,
            done: ProgressUnitsV1::default(),
            total,
            last_emit: Instant::now(),
            on_progress,
        }
    }

    fn maybe_emit(&mut self, force: bool) {
        if !force && self.last_emit.elapsed() < FS_PROGRESS_MIN_INTERVAL {
            return;
        }
        self.last_emit = Instant::now();
        (self.on_progress)(FilesystemBuildProgressUpdate {
            stage: self.stage,
            done: self.done,
            total: self.total,
        });
    }

    fn record_entry(&mut self, kind: &str, size: u64) {
        if kind == "dir" {
            self.done.dirs = self.done.dirs.saturating_add(1);
            self.maybe_emit(false);
            return;
        }

        self.done.files = self.done.files.saturating_add(1);
        self.done.bytes = self.done.bytes.saturating_add(size);
        self.maybe_emit(false);
    }
}

fn reborrow_progress<'p, 'a>(
    progress: &'p mut Option<&mut FilesystemBuildProgressCtx<'a>>,
) -> Option<&'p mut FilesystemBuildProgressCtx<'a>> {
    progress.as_mut().map(|p| &mut **p)
}

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
    pub source_total: Option<ProgressUnitsV1>,
    pub raw_tree_stats: Option<RawTreeBuildStats>,
}

pub fn build_filesystem_run(
    data_dir: &Path,
    job_id: &str,
    run_id: &str,
    started_at: OffsetDateTime,
    source: &FilesystemSource,
    pipeline: BuildPipelineOptions<'_>,
    on_progress: Option<&dyn Fn(FilesystemBuildProgressUpdate)>,
    on_part_finished: Option<Box<dyn Fn(LocalArtifact) -> std::io::Result<()> + Send>>,
) -> Result<FilesystemRunBuild, anyhow::Error> {
    let BuildPipelineOptions {
        artifact_format,
        encryption,
        part_size_bytes,
    } = pipeline;
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
        artifact_format = ?artifact_format,
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

    let pre_scan_totals = if source.pre_scan {
        // Pre-scan only affects user-facing totals/ETA; packaging still enforces correctness.
        match on_progress {
            Some(cb) => {
                let mut ctx = FilesystemBuildProgressCtx::new("scan", None, cb);
                ctx.maybe_emit(true);
                let totals = scan::scan_filesystem_source(source, &mut issues, Some(&mut ctx))?;
                ctx.done = totals;
                ctx.total = Some(totals);
                ctx.maybe_emit(true);
                Some(totals)
            }
            None => Some(scan::scan_filesystem_source(source, &mut issues, None)?),
        }
    } else {
        None
    };

    let mut packaging_progress =
        on_progress.map(|cb| FilesystemBuildProgressCtx::new("packaging", pre_scan_totals, cb));
    if let Some(ctx) = packaging_progress.as_mut() {
        ctx.maybe_emit(true);
    }

    let (artifact_format, parts, raw_tree_stats, tar_kind, compression_kind, split_bytes) =
        match artifact_format {
            ArtifactFormatV1::ArchiveV1 => (
                ArtifactFormatV1::ArchiveV1,
                tar::write_tar_zstd_parts(
                    &stage,
                    source,
                    encryption,
                    &mut entries_writer,
                    &mut entries_count,
                    part_size_bytes,
                    &mut issues,
                    packaging_progress.as_mut(),
                    on_part_finished,
                )?,
                None,
                "pax",
                "zstd",
                part_size_bytes,
            ),
            ArtifactFormatV1::RawTreeV1 => {
                let _ = on_part_finished;
                if !matches!(encryption, PayloadEncryption::None) {
                    anyhow::bail!("raw_tree_v1 does not support payload encryption");
                }
                let stats = raw_tree::write_raw_tree(
                    &stage,
                    source,
                    &mut entries_writer,
                    &mut entries_count,
                    &mut issues,
                    packaging_progress.as_mut(),
                )?;
                (
                    ArtifactFormatV1::RawTreeV1,
                    Vec::new(),
                    Some(stats),
                    "none",
                    "none",
                    0,
                )
            }
        };
    entries_writer.finish()?;
    if let Some(ctx) = packaging_progress.as_mut() {
        ctx.maybe_emit(true);
    }

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
            tar: tar_kind.to_string(),
            compression: compression_kind.to_string(),
            encryption: match encryption {
                PayloadEncryption::None => "none".to_string(),
                PayloadEncryption::AgeX25519 { .. } => "age".to_string(),
            },
            encryption_key: match encryption {
                PayloadEncryption::None => None,
                PayloadEncryption::AgeX25519 { key_name, .. } => Some(key_name.clone()),
            },
            split_bytes,
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
        raw_tree_files = raw_tree_stats.as_ref().map(|s| s.data_files).unwrap_or(0),
        raw_tree_bytes = raw_tree_stats.as_ref().map(|s| s.data_bytes).unwrap_or(0),
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
        source_total: pre_scan_totals,
        raw_tree_stats,
    })
}

#[cfg(test)]
mod tests;

mod scan;
