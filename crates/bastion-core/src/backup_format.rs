use std::path::PathBuf;

use crate::manifest::HashAlgorithm;

pub const ENTRIES_INDEX_NAME: &str = "entries.jsonl.zst";
pub const MANIFEST_NAME: &str = "manifest.json";
pub const COMPLETE_NAME: &str = "complete.json";

#[derive(Debug, Clone)]
pub struct LocalArtifact {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub hash_alg: HashAlgorithm,
    pub hash: String,
}

#[derive(Debug, Clone)]
pub struct LocalRunArtifacts {
    pub run_dir: PathBuf,
    pub parts: Vec<LocalArtifact>,
    pub entries_index_path: PathBuf,
    pub entries_count: u64,
    pub manifest_path: PathBuf,
    pub complete_path: PathBuf,
}
