use bastion_core::manifest::HashAlgorithm;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(in crate::restore) struct EntryRecord {
    pub(in crate::restore) path: String,
    pub(in crate::restore) kind: String,
    pub(in crate::restore) size: u64,
    pub(in crate::restore) hash_alg: Option<HashAlgorithm>,
    pub(in crate::restore) hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RunEntriesChild {
    pub path: String,
    pub kind: String,
    pub size: u64,
}

#[derive(Debug, Serialize)]
pub struct RunEntriesChildrenResponse {
    pub prefix: String,
    pub cursor: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<u64>,
    pub entries: Vec<RunEntriesChild>,
}

#[derive(Debug, Clone, Default)]
pub struct ListRunEntriesChildrenOptions {
    pub prefix: Option<String>,
    pub cursor: u64,
    pub limit: u64,
    pub q: Option<String>,
    pub kind: Option<String>,
    pub hide_dotfiles: bool,
    pub min_size_bytes: Option<u64>,
    pub max_size_bytes: Option<u64>,
    pub type_sort_file_first: bool,
}

#[derive(Debug)]
pub(in crate::restore) struct ListChildrenFromEntriesIndexOptions {
    pub(in crate::restore) prefix: String,
    pub(in crate::restore) cursor: usize,
    pub(in crate::restore) limit: usize,
    pub(in crate::restore) q: Option<String>,
    pub(in crate::restore) kind: Option<String>,
    pub(in crate::restore) hide_dotfiles: bool,
    pub(in crate::restore) min_size_bytes: Option<u64>,
    pub(in crate::restore) max_size_bytes: Option<u64>,
    pub(in crate::restore) type_sort_file_first: bool,
}
