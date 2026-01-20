use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};

use bastion_core::manifest::HashAlgorithm;
use serde::Serialize;

pub(super) type EntriesIndexWriter<'a> = zstd::Encoder<'a, BufWriter<File>>;

#[derive(Debug, Serialize)]
pub(super) struct EntryRecord {
    pub(super) path: String,
    pub(super) kind: String,
    pub(super) size: u64,
    pub(super) hash_alg: Option<HashAlgorithm>,
    pub(super) hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) mtime: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) mode: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) uid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) gid: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) xattrs: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) symlink_target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) hardlink_group: Option<String>,
}

pub(super) fn write_entry_record(
    entries_writer: &mut EntriesIndexWriter<'_>,
    entries_count: &mut u64,
    record: EntryRecord,
) -> Result<(), anyhow::Error> {
    let line = serde_json::to_vec(&record)?;
    entries_writer.write_all(&line)?;
    entries_writer.write_all(b"\n")?;
    *entries_count += 1;
    Ok(())
}
