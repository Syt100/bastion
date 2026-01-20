use std::io::Read;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

mod access;
mod engine;
mod entries_index;
mod operations;
mod parts;
mod path;
mod sinks;
pub mod sources;
mod unpack;
mod verify;
pub use entries_index::{
    ListRunEntriesChildrenOptions, RunEntriesChild, RunEntriesChildrenResponse,
    list_run_entries_children, list_run_entries_children_with_options,
};
pub use operations::{spawn_restore_operation, spawn_verify_operation};

#[derive(Debug, Clone, Copy)]
pub enum ConflictPolicy {
    Overwrite,
    Skip,
    Fail,
}

impl ConflictPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Overwrite => "overwrite",
            Self::Skip => "skip",
            Self::Fail => "fail",
        }
    }
}

impl std::str::FromStr for ConflictPolicy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "overwrite" => Ok(Self::Overwrite),
            "skip" => Ok(Self::Skip),
            "fail" => Ok(Self::Fail),
            _ => Err(anyhow::anyhow!("invalid conflict policy")),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct RestoreSelection {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub dirs: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PayloadDecryption {
    None,
    AgeX25519 { identity: String },
}

pub fn restore_to_local_fs(
    payload: Box<dyn Read + Send>,
    destination_dir: PathBuf,
    conflict: ConflictPolicy,
    decryption: PayloadDecryption,
    selection: Option<&RestoreSelection>,
) -> Result<(), anyhow::Error> {
    let mut sink = sinks::LocalFsSink::new(destination_dir, conflict);
    let mut engine = engine::RestoreEngine::new(&mut sink, decryption, selection)?;
    engine.restore(payload)?;
    Ok(())
}

#[cfg(test)]
mod tests;
