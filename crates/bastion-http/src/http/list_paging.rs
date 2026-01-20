use base64::Engine as _;
use serde::{Deserialize, Serialize};

use super::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum SortBy {
    Name,
    Mtime,
    Size,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum SortDir {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SortKey {
    pub(super) by: SortBy,
    pub(super) dir: SortDir,
    pub(super) rank: u8,
    pub(super) name: String,
    pub(super) mtime: i64,
    pub(super) size: u64,
}

impl PartialOrd for SortKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SortKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

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
            SortBy::Name => {
                let o = match self.dir {
                    SortDir::Asc => self.name.cmp(&other.name),
                    SortDir::Desc => other.name.cmp(&self.name),
                };
                if o != Ordering::Equal {
                    return o;
                }
                Ordering::Equal
            }
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
pub(super) struct CursorKey {
    pub(super) rank: u8,
    pub(super) name: String,
    #[serde(default)]
    pub(super) sort_by: Option<SortBy>,
    #[serde(default)]
    pub(super) sort_dir: Option<SortDir>,
    #[serde(default)]
    pub(super) mtime: Option<i64>,
    #[serde(default)]
    pub(super) size: Option<u64>,
}

pub(super) fn encode_cursor_key(key: &CursorKey) -> String {
    let json = serde_json::to_vec(key).unwrap_or_default();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
}

pub(super) fn decode_cursor_key(cursor: &str) -> Result<CursorKey, AppError> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| AppError::bad_request("invalid_cursor", "invalid cursor encoding"))?;
    serde_json::from_slice::<CursorKey>(&bytes)
        .map_err(|_| AppError::bad_request("invalid_cursor", "invalid cursor payload"))
}

pub(super) fn rank_kind(kind: &str, type_sort: Option<&str>) -> u8 {
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

pub(super) fn parse_sort_by(raw: Option<String>) -> Result<SortBy, AppError> {
    match raw.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(SortBy::Name),
        Some("name") => Ok(SortBy::Name),
        Some("mtime") => Ok(SortBy::Mtime),
        Some("size") => Ok(SortBy::Size),
        Some(_) => Err(AppError::bad_request("invalid_sort_by", "invalid sort_by")),
    }
}

pub(super) fn parse_sort_dir(raw: Option<String>) -> Result<SortDir, AppError> {
    match raw.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        None => Ok(SortDir::Asc),
        Some("asc") => Ok(SortDir::Asc),
        Some("desc") => Ok(SortDir::Desc),
        Some(_) => Err(AppError::bad_request(
            "invalid_sort_dir",
            "invalid sort_dir",
        )),
    }
}
