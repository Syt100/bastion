use std::cmp::Ordering;
use std::collections::BinaryHeap;

use base64::Engine as _;
use bastion_core::agent_protocol::FsDirEntryV1;
use bastion_targets::{
    WebdavClient, WebdavCredentials, WebdavHttpError, WebdavNotDirectoryError, WebdavPropfindEntry,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone)]
pub(super) struct WebdavListOptions {
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
pub(super) struct WebdavListPage {
    pub entries: Vec<FsDirEntryV1>,
    pub next_cursor: Option<String>,
    pub total: u64,
}

#[derive(Debug, Clone)]
pub(super) struct WebdavListError {
    pub code: String,
    pub message: String,
}

impl WebdavListError {
    fn invalid_cursor(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_cursor".to_string(),
            message: message.into(),
        }
    }
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

fn decode_cursor_key(cursor: &str) -> Result<CursorKey, WebdavListError> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| WebdavListError::invalid_cursor("invalid cursor encoding"))?;
    serde_json::from_slice::<CursorKey>(&bytes)
        .map_err(|_| WebdavListError::invalid_cursor("invalid cursor payload"))
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

fn parse_sort_by(raw: Option<String>) -> Result<SortBy, WebdavListError> {
    match raw.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(SortBy::Name),
        Some("name") => Ok(SortBy::Name),
        Some("mtime") => Ok(SortBy::Mtime),
        Some("size") => Ok(SortBy::Size),
        Some(_) => Err(WebdavListError {
            code: "error".to_string(),
            message: "invalid sort_by".to_string(),
        }),
    }
}

fn parse_sort_dir(raw: Option<String>) -> Result<SortDir, WebdavListError> {
    match raw.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(SortDir::Asc),
        Some("asc") => Ok(SortDir::Asc),
        Some("desc") => Ok(SortDir::Desc),
        Some(_) => Err(WebdavListError {
            code: "error".to_string(),
            message: "invalid sort_dir".to_string(),
        }),
    }
}

fn normalize_picker_path(path: &str) -> Result<String, WebdavListError> {
    let path = path.trim();
    if path.is_empty() {
        return Err(WebdavListError {
            code: "error".to_string(),
            message: "path is required".to_string(),
        });
    }

    let mut out = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    if out != "/" {
        out = out.trim_end_matches('/').to_string();
    }
    Ok(out)
}

fn join_picker_path(base: &str, name: &str) -> String {
    if base == "/" {
        format!("/{name}")
    } else {
        format!("{}/{}", base.trim_end_matches('/'), name)
    }
}

pub(super) async fn webdav_list_dir_entries_paged(
    base_url: &str,
    credentials: WebdavCredentials,
    path: &str,
    opts: WebdavListOptions,
) -> Result<WebdavListPage, WebdavListError> {
    let path = normalize_picker_path(path)?;

    let mut base_url = Url::parse(base_url.trim()).map_err(|e| WebdavListError {
        code: "error".to_string(),
        message: format!("invalid base_url: {e}"),
    })?;
    if !base_url.path().ends_with('/') {
        base_url.set_path(&format!("{}/", base_url.path()));
    }

    let mut list_url = base_url.clone();
    {
        let mut segs = list_url.path_segments_mut().map_err(|_| WebdavListError {
            code: "error".to_string(),
            message: "webdav base_url cannot be a base".to_string(),
        })?;
        for part in path
            .trim_matches('/')
            .split('/')
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            if part == "." || part == ".." {
                return Err(WebdavListError {
                    code: "error".to_string(),
                    message: "invalid path segment".to_string(),
                });
            }
            segs.push(part);
        }
    }
    if !list_url.path().ends_with('/') {
        list_url.set_path(&format!("{}/", list_url.path()));
    }

    let client = WebdavClient::new(base_url, credentials).map_err(|e| WebdavListError {
        code: "error".to_string(),
        message: format!("failed to init webdav client: {e}"),
    })?;

    let entries = client.propfind_depth1(&list_url).await.map_err(|error| {
        if let Some(e) = error.downcast_ref::<WebdavNotDirectoryError>() {
            return WebdavListError {
                code: "not_directory".to_string(),
                message: e.to_string(),
            };
        }
        if let Some(e) = error.downcast_ref::<WebdavHttpError>() {
            let code = match e.status {
                reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
                    "permission_denied"
                }
                reqwest::StatusCode::NOT_FOUND => "path_not_found",
                _ => "error",
            };
            return WebdavListError {
                code: code.to_string(),
                message: e.to_string(),
            };
        }
        WebdavListError {
            code: "error".to_string(),
            message: error.to_string(),
        }
    })?;

    webdav_page_entries(path.as_str(), entries, opts)
}

fn webdav_page_entries(
    path: &str,
    entries: Vec<WebdavPropfindEntry>,
    opts: WebdavListOptions,
) -> Result<WebdavListPage, WebdavListError> {
    #[derive(Debug, Clone)]
    struct Candidate {
        key: SortKey,
        kind: String,
        size: u64,
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
                return Err(WebdavListError::invalid_cursor(
                    "cursor sort options mismatch",
                ));
            }
            let cursor_mtime = match sort_by {
                SortBy::Mtime => decoded
                    .mtime
                    .ok_or_else(|| WebdavListError::invalid_cursor("cursor missing mtime key"))?,
                _ => decoded.mtime.unwrap_or(0),
            };
            let cursor_size = match sort_by {
                SortBy::Size => decoded
                    .size
                    .ok_or_else(|| WebdavListError::invalid_cursor("cursor missing size key"))?,
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

    let mut total: u64 = 0;
    let mut after_cursor_total: u64 = 0;
    let mut heap = BinaryHeap::<Candidate>::new();

    for entry in entries {
        let name = entry.name.trim();
        if name.is_empty() || name == "/" {
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

        let kind = entry.kind.trim();
        if let Some(k) = kind_filter
            && kind != k
        {
            continue;
        }

        let size = entry.size.unwrap_or(0);
        if size_filter_active && kind != "dir" {
            if let Some(min) = min_bytes
                && size < min
            {
                continue;
            }
            if let Some(max) = max_bytes
                && size > max
            {
                continue;
            }
        }

        total = total.saturating_add(1);

        let rank = rank_kind(kind, type_sort);
        let key = SortKey {
            by: sort_by,
            dir: sort_dir,
            rank,
            name: name.to_string(),
            mtime: entry.mtime.unwrap_or(0),
            size,
        };

        if let Some(cursor_key) = cursor_key.as_ref()
            && key.cmp(cursor_key) != Ordering::Greater
        {
            continue;
        }

        after_cursor_total = after_cursor_total.saturating_add(1);
        heap.push(Candidate {
            key,
            kind: kind.to_string(),
            size,
            mtime: entry.mtime,
        });
        if heap.len() > limit {
            let _ = heap.pop();
        }
    }

    let mut selected = heap.into_vec();
    selected.sort();

    let out_entries = selected
        .into_iter()
        .map(|c| FsDirEntryV1 {
            name: c.key.name.clone(),
            path: join_picker_path(path, &c.key.name),
            kind: c.kind,
            size: c.size,
            mtime: c.mtime,
        })
        .collect::<Vec<_>>();

    let next_cursor = if after_cursor_total > limit as u64 && !out_entries.is_empty() {
        let last = out_entries.last().unwrap();
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

    Ok(WebdavListPage {
        entries: out_entries,
        next_cursor,
        total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webdav_list_paged_basic_pagination_is_stable() {
        let entries = vec![
            WebdavPropfindEntry {
                href: "/root/a_dir/".to_string(),
                name: "a_dir".to_string(),
                kind: "dir".to_string(),
                size: None,
                mtime: None,
            },
            WebdavPropfindEntry {
                href: "/root/b.txt".to_string(),
                name: "b.txt".to_string(),
                kind: "file".to_string(),
                size: Some(1),
                mtime: Some(3),
            },
            WebdavPropfindEntry {
                href: "/root/c.txt".to_string(),
                name: "c.txt".to_string(),
                kind: "file".to_string(),
                size: Some(1),
                mtime: Some(2),
            },
        ];

        let page1 = webdav_page_entries(
            "/",
            entries.clone(),
            WebdavListOptions {
                cursor: None,
                limit: Some(2),
                q: None,
                kind: None,
                hide_dotfiles: false,
                type_sort: Some("dir_first".to_string()),
                sort_by: Some("name".to_string()),
                sort_dir: Some("asc".to_string()),
                size_min_bytes: None,
                size_max_bytes: None,
            },
        )
        .unwrap();
        assert_eq!(page1.entries.len(), 2);
        assert!(page1.next_cursor.is_some());

        let page2 = webdav_page_entries(
            "/",
            entries,
            WebdavListOptions {
                cursor: page1.next_cursor.clone(),
                limit: Some(2),
                q: None,
                kind: None,
                hide_dotfiles: false,
                type_sort: Some("dir_first".to_string()),
                sort_by: Some("name".to_string()),
                sort_dir: Some("asc".to_string()),
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
}
