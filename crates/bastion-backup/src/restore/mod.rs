use std::io::Read;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use bastion_targets::{WebdavClient, WebdavCredentials};
use url::Url;

mod access;
mod engine;
mod entries_index;
mod operations;
mod parts;
mod path;
mod raw_tree;
mod selection;
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

#[derive(Debug, Clone)]
pub enum RestoreDestination {
    LocalFs { directory: PathBuf },
    Webdav {
        base_url: String,
        secret_name: String,
        prefix: String,
    },
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

pub fn restore_to_webdav(
    payload: Box<dyn Read + Send>,
    base_url: &str,
    credentials: WebdavCredentials,
    prefix: &str,
    op_id: &str,
    conflict: ConflictPolicy,
    decryption: PayloadDecryption,
    selection: Option<&RestoreSelection>,
    staging_dir: PathBuf,
) -> Result<(), anyhow::Error> {
    let mut base_url = Url::parse(base_url.trim())?;
    if !base_url.path().ends_with('/') {
        base_url.set_path(&format!("{}/", base_url.path()));
    }

    let mut prefix_url = base_url;
    {
        let mut segs = prefix_url
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("webdav base_url cannot be a base"))?;
        for part in prefix
            .trim()
            .trim_matches('/')
            .split('/')
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            segs.push(part);
        }
    }
    if !prefix_url.path().ends_with('/') {
        prefix_url.set_path(&format!("{}/", prefix_url.path()));
    }

    let client = WebdavClient::new(prefix_url.clone(), credentials)?;
    let handle = tokio::runtime::Handle::current();
    let mut sink = sinks::WebdavSink::new(
        handle,
        client,
        prefix_url,
        conflict,
        op_id.trim().to_string(),
        staging_dir,
    )?;
    let mut engine = engine::RestoreEngine::new(&mut sink, decryption, selection)?;
    engine.restore(payload)?;
    Ok(())
}

#[cfg(test)]
mod tests;
