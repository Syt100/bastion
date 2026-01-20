pub mod filesystem;
pub mod sqlite;
pub mod vaultwarden;

use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub use bastion_core::backup_format::{
    COMPLETE_NAME, ENTRIES_INDEX_NAME, LocalArtifact, LocalRunArtifacts, MANIFEST_NAME,
};
use bastion_core::manifest::{ArtifactFormatV1, ArtifactPart, HashAlgorithm};

#[derive(Debug, Clone, Default)]
pub enum PayloadEncryption {
    #[default]
    None,
    AgeX25519 {
        recipient: String,
        key_name: String,
    },
}

#[derive(Debug, Clone)]
pub struct BuildPipelineOptions<'a> {
    pub artifact_format: ArtifactFormatV1,
    pub encryption: &'a PayloadEncryption,
    pub part_size_bytes: u64,
}

pub fn run_dir(data_dir: &Path, run_id: &str) -> PathBuf {
    data_dir.join("runs").join(run_id)
}

pub fn stage_dir(data_dir: &Path, run_id: &str) -> PathBuf {
    run_dir(data_dir, run_id).join("staging")
}

#[derive(Debug)]
pub struct PartWriter {
    dir: PathBuf,
    part_size: u64,
    prefix: &'static str,
    next_index: u32,
    current: Option<PartState>,
    parts: Vec<ArtifactPart>,
}

#[derive(Debug)]
struct PartState {
    name: String,
    file: File,
    hasher: blake3::Hasher,
    size: u64,
}

impl PartWriter {
    pub fn new(dir: PathBuf, part_size: u64, prefix: &'static str) -> Result<Self, io::Error> {
        Ok(Self {
            dir,
            part_size,
            prefix,
            next_index: 1,
            current: None,
            parts: Vec::new(),
        })
    }

    pub fn finish(mut self) -> Result<Vec<ArtifactPart>, io::Error> {
        self.finish_part()?;
        Ok(self.parts)
    }

    fn ensure_part(&mut self) -> Result<(), io::Error> {
        if self.current.is_some() {
            return Ok(());
        }

        std::fs::create_dir_all(&self.dir)?;
        let name = format!("{}{idx:06}", self.prefix, idx = self.next_index);
        self.next_index += 1;
        let path = self.dir.join(&name);
        let file = File::create(path)?;
        self.current = Some(PartState {
            name,
            file,
            hasher: blake3::Hasher::new(),
            size: 0,
        });
        Ok(())
    }

    fn finish_part(&mut self) -> Result<(), io::Error> {
        let Some(state) = self.current.take() else {
            return Ok(());
        };

        if state.size == 0 {
            return Ok(());
        }

        let hash = state.hasher.finalize().to_hex().to_string();
        self.parts.push(ArtifactPart {
            name: state.name,
            size: state.size,
            hash_alg: HashAlgorithm::Blake3,
            hash,
        });
        Ok(())
    }
}

impl Write for PartWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        self.ensure_part()?;

        let mut total_written = 0usize;
        while total_written < buf.len() {
            let state = self
                .current
                .as_mut()
                .expect("part exists after ensure_part");

            let remaining = (self.part_size.saturating_sub(state.size)) as usize;
            if remaining == 0 {
                self.finish_part()?;
                self.ensure_part()?;
                continue;
            }

            let n = remaining.min(buf.len() - total_written);
            state
                .file
                .write_all(&buf[total_written..total_written + n])?;
            state.hasher.update(&buf[total_written..total_written + n]);
            state.size = state.size.saturating_add(n as u64);
            total_written += n;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(state) = self.current.as_mut() {
            state.file.flush()?;
        }
        Ok(())
    }
}
