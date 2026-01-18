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
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub size_min_bytes: Option<u64>,
    pub size_max_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
pub(super) struct FsListPage {
    pub entries: Vec<FsDirEntryV1>,
    pub next_cursor: Option<String>,
    pub total: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SortBy {
    Name,
    Mtime,
    Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SortDir {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SortKey {
    by: SortBy,
    dir: SortDir,
    rank: u8,
    name: String,
    mtime: i64,
    size: u64,
}

impl PartialOrd for SortKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SortKey {
    fn cmp(&self, other: &Self) -> Ordering {
        // Ensure a total order even if sort options differ (should not happen in practice).
        let order = (self.by as u8, self.dir as u8, self.rank).cmp(&(
            other.by as u8,
            other.dir as u8,
            other.rank,
        ));
        if order != Ordering::Equal {
            return order;
        }

        match self.by {
            SortBy::Name => match self.dir {
                SortDir::Asc => self.name.cmp(&other.name),
                SortDir::Desc => other.name.cmp(&self.name),
            },
            SortBy::Mtime => {
                let o = match self.dir {
                    SortDir::Asc => self.mtime.cmp(&other.mtime),
                    SortDir::Desc => other.mtime.cmp(&self.mtime),
                };
                if o != Ordering::Equal {
                    return o;
                }
                self.name.cmp(&other.name)
            }
            SortBy::Size => {
                let o = match self.dir {
                    SortDir::Asc => self.size.cmp(&other.size),
                    SortDir::Desc => other.size.cmp(&self.size),
                };
                if o != Ordering::Equal {
                    return o;
                }
                self.name.cmp(&other.name)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CursorKey {
    rank: u8,
    name: String,
    #[serde(default)]
    sort_by: Option<SortBy>,
    #[serde(default)]
    sort_dir: Option<SortDir>,
    #[serde(default)]
    mtime: Option<i64>,
    #[serde(default)]
    size: Option<u64>,
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

fn parse_sort_by(raw: Option<String>) -> Result<SortBy, String> {
    match raw.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(SortBy::Name),
        Some("name") => Ok(SortBy::Name),
        Some("mtime") => Ok(SortBy::Mtime),
        Some("size") => Ok(SortBy::Size),
        Some(_) => Err("invalid sort_by".to_string()),
    }
}

fn parse_sort_dir(raw: Option<String>) -> Result<SortDir, String> {
    match raw.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(SortDir::Asc),
        Some("asc") => Ok(SortDir::Asc),
        Some("desc") => Ok(SortDir::Desc),
        Some(_) => Err("invalid sort_dir".to_string()),
    }
}

#[derive(Debug, Clone)]
struct Candidate {
    key: SortKey,
    path: String,
    kind: String,
    size: Option<u64>,
    mtime: Option<i64>,
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
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
        self.key.cmp(&other.key)
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
    let mtime = meta
        .and_then(|m| m.modified().ok())
        .and_then(to_unix_seconds);
    (size, mtime)
}

pub(super) fn fs_list_dir_entries_paged(
    path: &str,
    opts: FsListOptions,
) -> Result<FsListPage, String> {
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
        && opts.sort_by.is_none()
        && opts.sort_dir.is_none()
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

            let ft = entry
                .file_type()
                .map_err(|e| format!("file_type failed: {e}"))?;
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
    let sort_by = parse_sort_by(opts.sort_by)?;
    let sort_dir = parse_sort_dir(opts.sort_dir)?;
    let min_bytes = opts.size_min_bytes;
    let max_bytes = opts.size_max_bytes;
    let size_filter_active = min_bytes.is_some() || max_bytes.is_some();

    let limit = opts.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT) as usize;

    let cursor_key = match opts.cursor.as_deref() {
        Some(v) if !v.trim().is_empty() => {
            let decoded = decode_cursor_key(v.trim())?;
            let cursor_sort_by = decoded.sort_by.unwrap_or(SortBy::Name);
            let cursor_sort_dir = decoded.sort_dir.unwrap_or(SortDir::Asc);
            if cursor_sort_by != sort_by || cursor_sort_dir != sort_dir {
                return Err("invalid cursor: sort options mismatch".to_string());
            }
            let cursor_mtime = match sort_by {
                SortBy::Mtime => decoded
                    .mtime
                    .ok_or_else(|| "invalid cursor: missing mtime key".to_string())?,
                _ => decoded.mtime.unwrap_or(0),
            };
            let cursor_size = match sort_by {
                SortBy::Size => decoded
                    .size
                    .ok_or_else(|| "invalid cursor: missing size key".to_string())?,
                _ => decoded.size.unwrap_or(0),
            };
            Some(SortKey {
                by: sort_by,
                dir: sort_dir,
                rank: decoded.rank,
                name: decoded.name,
                mtime: cursor_mtime,
                size: cursor_size,
            })
        }
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
        if let Some(needle) = needle.as_deref()
            && !name.to_lowercase().contains(needle)
        {
            continue;
        }

        let ft = entry
            .file_type()
            .map_err(|e| format!("file_type failed: {e}"))?;
        let kind = if ft.is_dir() {
            "dir"
        } else if ft.is_file() {
            "file"
        } else if ft.is_symlink() {
            "symlink"
        } else {
            "other"
        };

        if let Some(k) = kind_filter
            && kind != k
        {
            continue;
        }

        // Size filter applies to non-dir entries only (matches UI semantics).
        let mut size: Option<u64> = None;
        let mut mtime: Option<i64> = None;
        let needs_meta_for_sort = sort_by != SortBy::Name;
        let needs_meta_for_size_filter = size_filter_active && kind != "dir";
        if needs_meta_for_sort || needs_meta_for_size_filter {
            let (s, t) = read_meta(&entry.path());
            size = Some(s);
            mtime = t;
            if needs_meta_for_size_filter {
                if let Some(min) = min_bytes
                    && s < min
                {
                    continue;
                }
                if let Some(max) = max_bytes
                    && s > max
                {
                    continue;
                }
            }
        }

        total = total.saturating_add(1);

        let rank = rank_kind(kind, type_sort);
        let key = SortKey {
            by: sort_by,
            dir: sort_dir,
            rank,
            name: name.clone(),
            mtime: mtime.unwrap_or(0),
            size: size.unwrap_or(0),
        };
        if let Some(cursor_key) = cursor_key.as_ref()
            && key.cmp(cursor_key) != Ordering::Greater
        {
            continue;
        }

        after_cursor_total = after_cursor_total.saturating_add(1);

        let candidate = Candidate {
            key,
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
                name: c.key.name,
                path: c.path,
                kind: c.kind,
                size: c.size.unwrap_or(0),
                mtime: c.mtime,
            }
        })
        .collect::<Vec<_>>();

    let next_cursor = if after_cursor_total > limit as u64 && !entries.is_empty() {
        let last = entries.last().unwrap();
        let mut key = CursorKey {
            rank: rank_kind(&last.kind, type_sort),
            name: last.name.clone(),
            sort_by: Some(sort_by),
            sort_dir: Some(sort_dir),
            mtime: None,
            size: None,
        };
        match sort_by {
            SortBy::Mtime => key.mtime = Some(last.mtime.unwrap_or(0)),
            SortBy::Size => key.size = Some(last.size),
            SortBy::Name => {}
        }
        Some(encode_cursor_key(&key))
    } else {
        None
    };

    Ok(FsListPage {
        entries,
        next_cursor,
        total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_list_dir_entries_paged_sort_modes_are_stable() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join("a_dir")).unwrap();
        std::fs::write(dir.path().join("b_small.txt"), "b").unwrap();
        std::fs::write(dir.path().join("c_med.txt"), "cc").unwrap();
        std::fs::write(dir.path().join("d_big.txt"), "dddddd").unwrap();
        std::fs::write(dir.path().join("e.txt"), "e").unwrap();
        std::fs::write(dir.path().join("f.txt"), "ff").unwrap();
        std::fs::write(dir.path().join("g.txt"), "ggg").unwrap();

        let cases = [
            (None, None),
            (Some("name".to_string()), Some("asc".to_string())),
            (Some("name".to_string()), Some("desc".to_string())),
            (Some("mtime".to_string()), Some("asc".to_string())),
            (Some("mtime".to_string()), Some("desc".to_string())),
            (Some("size".to_string()), Some("asc".to_string())),
            (Some("size".to_string()), Some("desc".to_string())),
        ];

        for (sort_by, sort_dir) in cases {
            let full = fs_list_dir_entries_paged(
                dir.path().to_string_lossy().as_ref(),
                FsListOptions {
                    cursor: None,
                    limit: Some(2000),
                    q: None,
                    kind: None,
                    hide_dotfiles: false,
                    type_sort: Some("dir_first".to_string()),
                    sort_by: sort_by.clone(),
                    sort_dir: sort_dir.clone(),
                    size_min_bytes: None,
                    size_max_bytes: None,
                },
            )
            .unwrap();
            assert!(full.entries.len() >= 7);

            let page1 = fs_list_dir_entries_paged(
                dir.path().to_string_lossy().as_ref(),
                FsListOptions {
                    cursor: None,
                    limit: Some(3),
                    q: None,
                    kind: None,
                    hide_dotfiles: false,
                    type_sort: Some("dir_first".to_string()),
                    sort_by: sort_by.clone(),
                    sort_dir: sort_dir.clone(),
                    size_min_bytes: None,
                    size_max_bytes: None,
                },
            )
            .unwrap();
            assert_eq!(page1.entries.len(), 3);
            assert!(page1.next_cursor.is_some());

            let page2 = fs_list_dir_entries_paged(
                dir.path().to_string_lossy().as_ref(),
                FsListOptions {
                    cursor: page1.next_cursor.clone(),
                    limit: Some(3),
                    q: None,
                    kind: None,
                    hide_dotfiles: false,
                    type_sort: Some("dir_first".to_string()),
                    sort_by: sort_by.clone(),
                    sort_dir: sort_dir.clone(),
                    size_min_bytes: None,
                    size_max_bytes: None,
                },
            )
            .unwrap();
            assert_eq!(page2.entries.len(), 3);

            let combined = page1
                .entries
                .into_iter()
                .chain(page2.entries.into_iter())
                .collect::<Vec<_>>();
            assert_eq!(&combined[..], &full.entries[..combined.len()]);
        }
    }
}
