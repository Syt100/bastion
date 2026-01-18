use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use base64::Engine as _;
use bastion_core::agent_protocol::FsDirEntryV1;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub(super) struct FsListOptions {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub q: Option<String>,
    pub kind: Option<String>,
    pub hide_dotfiles: bool,
    pub type_sort: Option<String>,
    pub size_min_bytes: Option<u64>,
    pub size_max_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
pub(super) struct FsListPage {
    pub entries: Vec<FsDirEntryV1>,
    pub next_cursor: Option<String>,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CursorKey {
    rank: u8,
    name: String,
}

fn encode_cursor_key(key: &CursorKey) -> String {
    let json = serde_json::to_vec(key).unwrap_or_default();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
}

fn decode_cursor_key(cursor: &str) -> Result<CursorKey, String> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| "invalid cursor encoding".to_string())?;
    serde_json::from_slice::<CursorKey>(&bytes).map_err(|_| "invalid cursor payload".to_string())
}

fn rank_kind(kind: &str, type_sort: Option<&str>) -> u8 {
    let file_like = kind == "file" || kind == "symlink";
    match type_sort {
        Some("file_first") => {
            if file_like {
                0
            } else if kind == "dir" {
                1
            } else {
                2
            }
        }
        _ => {
            if kind == "dir" {
                0
            } else if file_like {
                1
            } else {
                2
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Candidate {
    rank: u8,
    name: String,
    path: String,
    kind: String,
    size: Option<u64>,
    mtime: Option<i64>,
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.rank == other.rank && self.name == other.name
    }
}

impl Eq for Candidate {}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.rank, &self.name).cmp(&(other.rank, &other.name))
    }
}

fn to_unix_seconds(t: std::time::SystemTime) -> Option<i64> {
    t.duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs() as i64)
}

fn read_meta(path: &PathBuf) -> (u64, Option<i64>) {
    let meta = std::fs::metadata(path).ok();
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let mtime = meta.and_then(|m| m.modified().ok()).and_then(to_unix_seconds);
    (size, mtime)
}

pub(super) fn fs_list_dir_entries_paged(path: &str, opts: FsListOptions) -> Result<FsListPage, String> {
    const DEFAULT_LIMIT: u32 = 200;
    const MAX_LIMIT: u32 = 2000;

    let path = path.trim();
    if path.is_empty() {
        return Err("path is required".to_string());
    }

    let dir = PathBuf::from(path);
    let meta = std::fs::metadata(&dir).map_err(|e| format!("stat failed: {e}"))?;
    if !meta.is_dir() {
        return Err("path is not a directory".to_string());
    }

    let is_legacy_full_list = opts.cursor.is_none()
        && opts.limit.is_none()
        && opts.q.is_none()
        && opts.kind.is_none()
        && !opts.hide_dotfiles
        && opts.type_sort.is_none()
        && opts.size_min_bytes.is_none()
        && opts.size_max_bytes.is_none();

    if is_legacy_full_list {
        let mut out = Vec::<FsDirEntryV1>::new();
        let entries = std::fs::read_dir(&dir).map_err(|e| format!("read_dir failed: {e}"))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("read_dir entry failed: {e}"))?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.trim().is_empty() {
                continue;
            }

            let ft = entry.file_type().map_err(|e| format!("file_type failed: {e}"))?;
            let kind = if ft.is_dir() {
                "dir"
            } else if ft.is_file() {
                "file"
            } else if ft.is_symlink() {
                "symlink"
            } else {
                "other"
            };

            let (size, mtime) = read_meta(&entry.path());
            out.push(FsDirEntryV1 {
                name,
                path: entry.path().to_string_lossy().to_string(),
                kind: kind.to_string(),
                size,
                mtime,
            });
        }

        out.sort_by(|a, b| a.name.cmp(&b.name));
        return Ok(FsListPage {
            total: out.len() as u64,
            entries: out,
            next_cursor: None,
        });
    }

    let needle = opts.q.as_deref().map(|v| v.to_lowercase());
    let kind_filter = opts.kind.as_deref();
    let type_sort = opts.type_sort.as_deref();
    let min_bytes = opts.size_min_bytes;
    let max_bytes = opts.size_max_bytes;
    let size_filter_active = min_bytes.is_some() || max_bytes.is_some();

    let limit = opts.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT) as usize;

    let cursor_key = match opts.cursor.as_deref() {
        Some(v) if !v.trim().is_empty() => Some(decode_cursor_key(v.trim())?),
        _ => None,
    };

    let mut total: u64 = 0;
    let mut after_cursor_total: u64 = 0;
    let mut heap = BinaryHeap::<Candidate>::new();

    let entries = std::fs::read_dir(&dir).map_err(|e| format!("read_dir failed: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read_dir entry failed: {e}"))?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.trim().is_empty() {
            continue;
        }

        if opts.hide_dotfiles && name.starts_with('.') {
            continue;
        }
        if let Some(needle) = needle.as_deref() {
            if !name.to_lowercase().contains(needle) {
                continue;
            }
        }

        let ft = entry.file_type().map_err(|e| format!("file_type failed: {e}"))?;
        let kind = if ft.is_dir() {
            "dir"
        } else if ft.is_file() {
            "file"
        } else if ft.is_symlink() {
            "symlink"
        } else {
            "other"
        };

        if let Some(k) = kind_filter {
            if kind != k {
                continue;
            }
        }

        // Size filter applies to non-dir entries only (matches UI semantics).
        let mut size: Option<u64> = None;
        let mut mtime: Option<i64> = None;
        if size_filter_active && kind != "dir" {
            let (s, t) = read_meta(&entry.path());
            size = Some(s);
            mtime = t;
            if let Some(min) = min_bytes {
                if s < min {
                    continue;
                }
            }
            if let Some(max) = max_bytes {
                if s > max {
                    continue;
                }
            }
        }

        total = total.saturating_add(1);

        let rank = rank_kind(kind, type_sort);
        let key = CursorKey {
            rank,
            name: name.clone(),
        };
        if let Some(cursor_key) = cursor_key.as_ref() {
            if (key.rank, &key.name) <= (cursor_key.rank, &cursor_key.name) {
                continue;
            }
        }

        after_cursor_total = after_cursor_total.saturating_add(1);

        let candidate = Candidate {
            rank,
            name,
            path: entry.path().to_string_lossy().to_string(),
            kind: kind.to_string(),
            size,
            mtime,
        };

        heap.push(candidate);
        if heap.len() > limit {
            // Remove the largest element so the heap keeps the smallest `limit`.
            let _ = heap.pop();
        }
    }

    let mut selected: Vec<Candidate> = heap.into_vec();
    selected.sort();

    // Fill metadata for selected entries when it wasn't computed during scanning.
    let entries = selected
        .into_iter()
        .map(|mut c| {
            if c.size.is_none() || c.mtime.is_none() {
                let (s, t) = read_meta(&PathBuf::from(&c.path));
                c.size = Some(s);
                c.mtime = t;
            }
            FsDirEntryV1 {
                name: c.name,
                path: c.path,
                kind: c.kind,
                size: c.size.unwrap_or(0),
                mtime: c.mtime,
            }
        })
        .collect::<Vec<_>>();

    let next_cursor = if after_cursor_total > limit as u64 && !entries.is_empty() {
        let last = entries.last().unwrap();
        Some(encode_cursor_key(&CursorKey {
            rank: rank_kind(&last.kind, type_sort),
            name: last.name.clone(),
        }))
    } else {
        None
    };

    Ok(FsListPage {
        entries,
        next_cursor,
        total,
    })
}
