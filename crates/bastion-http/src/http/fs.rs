use std::path::PathBuf;
use std::time::Duration;

use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::debug;

use bastion_core::HUB_NODE_ID;

use super::shared::require_session;
use super::{AppError, AppState};

#[derive(Debug, Deserialize)]
pub(super) struct FsListQuery {
    path: String,
    #[serde(default)]
    cursor: Option<String>,
    #[serde(default)]
    limit: Option<u32>,
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    hide_dotfiles: Option<bool>,
    #[serde(default)]
    type_sort: Option<String>,
    #[serde(default)]
    size_min_bytes: Option<u64>,
    #[serde(default)]
    size_max_bytes: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(super) struct FsListEntry {
    name: String,
    path: String,
    kind: String,
    size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    mtime: Option<i64>,
}

#[derive(Debug, Serialize)]
pub(super) struct FsListResponse {
    path: String,
    entries: Vec<FsListEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total: Option<u64>,
}

pub(super) async fn fs_list(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(node_id): Path<String>,
    Query(query): Query<FsListQuery>,
) -> Result<Json<FsListResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let path = query.path.trim();
    if path.is_empty() {
        return Err(AppError::bad_request("invalid_path", "path is required")
            .with_details(serde_json::json!({ "field": "path" })));
    }

    if node_id == HUB_NODE_ID {
        let path = path.to_string();
        let path_for_worker = path.clone();
        let opts = FsListOptions {
            cursor: query.cursor,
            limit: query.limit,
            q: query.q,
            kind: query.kind,
            hide_dotfiles: query.hide_dotfiles.unwrap_or(false),
            type_sort: query.type_sort,
            size_min_bytes: query.size_min_bytes,
            size_max_bytes: query.size_max_bytes,
        };
        let page = tokio::task::spawn_blocking(move || list_dir_entries_paged(&path_for_worker, opts))
            .await
            .map_err(|e| anyhow::anyhow!(e))??;
        return Ok(Json(FsListResponse {
            path,
            entries: page.entries,
            next_cursor: page.next_cursor,
            total: Some(page.total),
        }));
    }

    if !state.agent_manager.is_connected(&node_id).await {
        return Err(AppError::conflict("agent_offline", "Agent is offline"));
    }

    let page = state
        .agent_manager
        .fs_list_page(
            &node_id,
            path.to_string(),
            bastion_engine::agent_manager::FsListOptions {
                cursor: query.cursor,
                limit: query.limit,
                q: query.q,
                kind: query.kind,
                hide_dotfiles: query.hide_dotfiles.unwrap_or(false),
                type_sort: query.type_sort,
                size_min_bytes: query.size_min_bytes,
                size_max_bytes: query.size_max_bytes,
            },
            Duration::from_secs(5),
        )
        .await
        .map_err(|error| {
            AppError::bad_request(
                "agent_fs_list_failed",
                format!("Agent filesystem list failed: {error}"),
            )
        })?;

    Ok(Json(FsListResponse {
        path: path.to_string(),
        entries: page
            .entries
            .into_iter()
            .map(|e| FsListEntry {
                name: e.name,
                path: e.path,
                kind: e.kind,
                size: e.size,
                mtime: e.mtime,
            })
            .collect(),
        next_cursor: page.next_cursor,
        total: page.total,
    }))
}

#[derive(Debug, Clone)]
struct FsListOptions {
    cursor: Option<String>,
    limit: Option<u32>,
    q: Option<String>,
    kind: Option<String>,
    hide_dotfiles: bool,
    type_sort: Option<String>,
    size_min_bytes: Option<u64>,
    size_max_bytes: Option<u64>,
}

#[derive(Debug)]
struct FsListPage {
    entries: Vec<FsListEntry>,
    next_cursor: Option<String>,
    total: u64,
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

fn decode_cursor_key(cursor: &str) -> Result<CursorKey, AppError> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| AppError::bad_request("invalid_cursor", "invalid cursor encoding"))?;
    serde_json::from_slice::<CursorKey>(&bytes)
        .map_err(|_| AppError::bad_request("invalid_cursor", "invalid cursor payload"))
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

fn list_dir_entries_paged(path: &str, opts: FsListOptions) -> Result<FsListPage, AppError> {
    use std::io::ErrorKind;
    use std::time::UNIX_EPOCH;

    fn map_io(path: &str, error: std::io::Error) -> AppError {
        match error.kind() {
            ErrorKind::NotFound => AppError::not_found("path_not_found", "Path not found")
                .with_details(serde_json::json!({ "path": path })),
            ErrorKind::PermissionDenied => {
                AppError::forbidden("permission_denied", "Permission denied")
                    .with_details(serde_json::json!({ "path": path }))
            }
            _ => {
                AppError::bad_request("fs_list_failed", format!("Filesystem list failed: {error}"))
                    .with_details(serde_json::json!({ "path": path }))
            }
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
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for Candidate {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            (self.rank, &self.name).cmp(&(other.rank, &other.name))
        }
    }

    const DEFAULT_LIMIT: u32 = 200;
    const MAX_LIMIT: u32 = 2000;

    let cursor = opts.cursor.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    });
    let q = opts.q.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    });
    let kind_filter = opts.kind.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    });
    let type_sort = opts.type_sort.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    });

    let needle = q.as_deref().map(|v| v.to_lowercase());
    let kind_filter = kind_filter.as_deref();
    let type_sort = type_sort.as_deref();
    let min_bytes = opts.size_min_bytes;
    let max_bytes = opts.size_max_bytes;
    let size_filter_active = min_bytes.is_some() || max_bytes.is_some();

    let limit = opts.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT) as usize;
    let cursor_key = match cursor.as_deref() {
        Some(v) => Some(decode_cursor_key(v)?),
        None => None,
    };

    let dir_path = path;
    let dir = PathBuf::from(dir_path);
    let meta = std::fs::metadata(&dir).map_err(|e| map_io(dir_path, e))?;
    if !meta.is_dir() {
        return Err(
            AppError::bad_request("not_directory", "path is not a directory")
                .with_details(serde_json::json!({ "path": dir_path })),
        );
    }

    let mut total: u64 = 0;
    let mut after_cursor_total: u64 = 0;
    let mut heap = std::collections::BinaryHeap::<Candidate>::new();

    let iter = std::fs::read_dir(&dir).map_err(|e| map_io(dir_path, e))?;
    for entry in iter {
        let entry = match entry {
            Ok(v) => v,
            Err(error) => {
                debug!(dir = %dir_path, error = %error, "fs list entry failed");
                continue;
            }
        };
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

        let ft = match entry.file_type() {
            Ok(v) => v,
            Err(error) => {
                debug!(dir = %dir_path, error = %error, "fs list file_type failed");
                continue;
            }
        };
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
            let _ = heap.pop();
        }
    }

    let mut selected: Vec<Candidate> = heap.into_vec();
    selected.sort();

    let entries = selected
        .into_iter()
        .map(|mut c| {
            if c.size.is_none() || c.mtime.is_none() {
                let (s, t) = read_meta(&PathBuf::from(&c.path));
                c.size = Some(s);
                c.mtime = t;
            }
            FsListEntry {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_list_paged_not_found_maps_to_path_not_found() {
        let err = list_dir_entries_paged(
            "/__bastion__definitely__does_not_exist__",
            FsListOptions {
                cursor: None,
                limit: Some(10),
                q: None,
                kind: None,
                hide_dotfiles: false,
                type_sort: None,
                size_min_bytes: None,
                size_max_bytes: None,
            },
        )
        .unwrap_err();
        assert_eq!(err.code(), "path_not_found");
    }

    #[test]
    fn fs_list_paged_basic_pagination_is_stable() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join("a_dir")).unwrap();
        std::fs::write(dir.path().join("b.txt"), "b").unwrap();
        std::fs::write(dir.path().join("c.txt"), "c").unwrap();

        let page1 = list_dir_entries_paged(
            dir.path().to_string_lossy().as_ref(),
            FsListOptions {
                cursor: None,
                limit: Some(2),
                q: None,
                kind: None,
                hide_dotfiles: false,
                type_sort: Some("dir_first".to_string()),
                size_min_bytes: None,
                size_max_bytes: None,
            },
        )
        .unwrap();
        assert_eq!(page1.entries.len(), 2);
        assert!(page1.next_cursor.is_some());

        let page2 = list_dir_entries_paged(
            dir.path().to_string_lossy().as_ref(),
            FsListOptions {
                cursor: page1.next_cursor.clone(),
                limit: Some(2),
                q: None,
                kind: None,
                hide_dotfiles: false,
                type_sort: Some("dir_first".to_string()),
                size_min_bytes: None,
                size_max_bytes: None,
            },
        )
        .unwrap();
        assert!(!page2.entries.is_empty());

        let names1 = page1.entries.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
        let names2 = page2.entries.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
        for n in names2 {
            assert!(!names1.contains(&n));
        }
    }
}
