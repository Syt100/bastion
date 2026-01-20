use std::path::PathBuf;
use std::time::Duration;

use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::debug;

use bastion_core::HUB_NODE_ID;

use super::list_paging::{
    CursorKey, SortBy, SortDir, SortKey, decode_cursor_key, encode_cursor_key, parse_sort_by,
    parse_sort_dir, rank_kind,
};
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
    sort_by: Option<String>,
    #[serde(default)]
    sort_dir: Option<String>,
    #[serde(default)]
    size_min_bytes: Option<u64>,
    #[serde(default)]
    size_max_bytes: Option<u64>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
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
            sort_by: query.sort_by,
            sort_dir: query.sort_dir,
            size_min_bytes: query.size_min_bytes,
            size_max_bytes: query.size_max_bytes,
        };
        let page =
            tokio::task::spawn_blocking(move || list_dir_entries_paged(&path_for_worker, opts))
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
                sort_by: query.sort_by,
                sort_dir: query.sort_dir,
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
    sort_by: Option<String>,
    sort_dir: Option<String>,
    size_min_bytes: Option<u64>,
    size_max_bytes: Option<u64>,
}

#[derive(Debug)]
struct FsListPage {
    entries: Vec<FsListEntry>,
    next_cursor: Option<String>,
    total: u64,
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
        let mtime = meta
            .and_then(|m| m.modified().ok())
            .and_then(to_unix_seconds);
        (size, mtime)
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
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for Candidate {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.key.cmp(&other.key)
        }
    }

    const DEFAULT_LIMIT: u32 = 200;
    const MAX_LIMIT: u32 = 2000;

    let cursor = opts.cursor.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let q = opts.q.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let kind_filter = opts.kind.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let type_sort = opts.type_sort.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let sort_by = parse_sort_by(opts.sort_by)?;
    let sort_dir = parse_sort_dir(opts.sort_dir)?;

    let needle = q.as_deref().map(|v| v.to_lowercase());
    let kind_filter = kind_filter.as_deref();
    let type_sort = type_sort.as_deref();
    let min_bytes = opts.size_min_bytes;
    let max_bytes = opts.size_max_bytes;
    let size_filter_active = min_bytes.is_some() || max_bytes.is_some();

    let limit = opts.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT) as usize;
    let cursor_key = match cursor.as_deref() {
        Some(v) => {
            let decoded = decode_cursor_key(v)?;
            let cursor_sort_by = decoded.sort_by.unwrap_or(SortBy::Name);
            let cursor_sort_dir = decoded.sort_dir.unwrap_or(SortDir::Asc);
            if cursor_sort_by != sort_by || cursor_sort_dir != sort_dir {
                return Err(AppError::bad_request(
                    "invalid_cursor",
                    "cursor sort options mismatch",
                ));
            }
            let cursor_mtime = match sort_by {
                SortBy::Mtime => decoded.mtime.ok_or_else(|| {
                    AppError::bad_request("invalid_cursor", "cursor missing mtime key")
                })?,
                _ => decoded.mtime.unwrap_or(0),
            };
            let cursor_size = match sort_by {
                SortBy::Size => decoded.size.ok_or_else(|| {
                    AppError::bad_request("invalid_cursor", "cursor missing size key")
                })?,
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
        if let Some(needle) = needle.as_deref()
            && !name.to_lowercase().contains(needle)
        {
            continue;
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
            && key.cmp(cursor_key) != std::cmp::Ordering::Greater
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
                sort_by: None,
                sort_dir: None,
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
                sort_by: None,
                sort_dir: None,
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
                sort_by: None,
                sort_dir: None,
                size_min_bytes: None,
                size_max_bytes: None,
            },
        )
        .unwrap();
        assert!(!page2.entries.is_empty());

        let names1 = page1
            .entries
            .iter()
            .map(|e| e.name.clone())
            .collect::<Vec<_>>();
        let names2 = page2
            .entries
            .iter()
            .map(|e| e.name.clone())
            .collect::<Vec<_>>();
        for n in names2 {
            assert!(!names1.contains(&n));
        }
    }

    #[test]
    fn fs_list_paged_sort_modes_are_stable() {
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
            let full = list_dir_entries_paged(
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

            let page1 = list_dir_entries_paged(
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

            let page2 = list_dir_entries_paged(
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
