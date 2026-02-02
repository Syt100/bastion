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
    progress: Option<&mut super::FilesystemBuildProgressCtx<'_>>,
) -> Result<(), anyhow::Error> {
    let line = serde_json::to_vec(&record)?;
    entries_writer.write_all(&line)?;
    entries_writer.write_all(b"\n")?;
    *entries_count += 1;
    if let Some(p) = progress {
        p.record_entry(&record.kind, record.size);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::BufWriter;
    use std::sync::{Arc, Mutex};

    use tempfile::TempDir;

    use bastion_core::manifest::HashAlgorithm;

    use super::{EntryRecord, write_entry_record};

    #[test]
    fn write_entry_record_increments_count_and_writes_jsonl() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("entries.jsonl.zst");

        let file = std::fs::File::create(&path).unwrap();
        let writer = BufWriter::new(file);
        let mut enc = zstd::Encoder::new(writer, 3).unwrap();

        let mut count = 0u64;
        write_entry_record(
            &mut enc,
            &mut count,
            EntryRecord {
                path: "a.txt".to_string(),
                kind: "file".to_string(),
                size: 5,
                hash_alg: Some(HashAlgorithm::Blake3),
                hash: Some("h".to_string()),
                mtime: None,
                mode: None,
                uid: None,
                gid: None,
                xattrs: None,
                symlink_target: None,
                hardlink_group: None,
            },
            None,
        )
        .unwrap();
        assert_eq!(count, 1);

        let _ = enc.finish().unwrap();

        let raw = std::fs::read(&path).unwrap();
        let decoded = zstd::decode_all(std::io::Cursor::new(raw)).unwrap();
        let lines: Vec<&[u8]> = decoded
            .split(|b| *b == b'\n')
            .filter(|l| !l.is_empty())
            .collect();
        assert_eq!(lines.len(), 1);

        let v: serde_json::Value = serde_json::from_slice(lines[0]).unwrap();
        assert_eq!(v["path"], "a.txt");
        assert_eq!(v["kind"], "file");
        assert_eq!(v["size"], 5);
        assert_eq!(v["hash_alg"], "blake3");
        assert_eq!(v["hash"], "h");
    }

    #[test]
    fn write_entry_record_can_update_progress_counters() {
        use super::super::{FilesystemBuildProgressCtx, FilesystemBuildProgressUpdate};
        use bastion_core::progress::ProgressUnitsV1;

        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("entries.jsonl.zst");

        let file = std::fs::File::create(&path).unwrap();
        let writer = BufWriter::new(file);
        let mut enc = zstd::Encoder::new(writer, 3).unwrap();

        let last = Arc::new(Mutex::new(None::<FilesystemBuildProgressUpdate>));
        let last_for_cb = last.clone();
        let cb = move |u: FilesystemBuildProgressUpdate| {
            *last_for_cb.lock().unwrap() = Some(u);
        };

        let mut progress =
            FilesystemBuildProgressCtx::new("scan", Some(ProgressUnitsV1::default()), &cb);

        let mut count = 0u64;
        write_entry_record(
            &mut enc,
            &mut count,
            EntryRecord {
                path: "dir".to_string(),
                kind: "dir".to_string(),
                size: 0,
                hash_alg: None,
                hash: None,
                mtime: None,
                mode: None,
                uid: None,
                gid: None,
                xattrs: None,
                symlink_target: None,
                hardlink_group: None,
            },
            Some(&mut progress),
        )
        .unwrap();
        assert_eq!(count, 1);

        // Force emit to avoid time-based throttling in tests.
        progress.maybe_emit(true);

        let got = last.lock().unwrap().clone().expect("progress update");
        assert_eq!(got.stage, "scan");
        assert_eq!(got.done.dirs, 1);
        assert_eq!(got.done.files, 0);
        assert_eq!(got.done.bytes, 0);
    }
}
