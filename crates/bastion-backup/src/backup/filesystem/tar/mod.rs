use std::io::Write as _;
use std::path::Path;

use bastion_core::job_spec::FilesystemSource;

use crate::backup::source_consistency::SourceConsistencyTracker;
use crate::backup::{LocalArtifact, PartWriter, PayloadEncryption};

use super::FilesystemBuildIssues;
use super::entries_index::EntriesIndexWriter;

mod entry;
mod walk;

#[allow(clippy::too_many_arguments)]
pub(super) fn write_tar_zstd_parts(
    stage_dir: &Path,
    source: &FilesystemSource,
    encryption: &PayloadEncryption,
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    part_size_bytes: u64,
    issues: &mut FilesystemBuildIssues,
    consistency: &mut SourceConsistencyTracker,
    progress: Option<&mut super::FilesystemBuildProgressCtx<'_>>,
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

            let mut tar = ::tar::Builder::new(encoder);
            walk::write_tar_entries(
                &mut tar,
                source,
                entries_writer,
                entries_count,
                issues,
                consistency,
                progress,
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

            let mut tar = ::tar::Builder::new(encoder);
            walk::write_tar_entries(
                &mut tar,
                source,
                entries_writer,
                entries_count,
                issues,
                consistency,
                progress,
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
