use std::time::Duration;

use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use tracing::debug;
use url::Url;

use bastion_core::HUB_NODE_ID;
use bastion_engine::agent_manager::WebdavListRemoteError;
use bastion_storage::secrets_repo;
use bastion_targets::{
    WebdavClient, WebdavCredentials, WebdavHttpError, WebdavNotDirectoryError, WebdavPropfindEntry,
};

use super::list_paging::{
    CursorKey, SortBy, SortDir, SortKey, decode_cursor_key, encode_cursor_key, parse_sort_by,
    parse_sort_dir, rank_kind,
};
use super::shared::require_session;
use super::{AppError, AppState};

#[derive(Debug, Deserialize)]
pub(super) struct WebdavListRequest {
    base_url: String,
    secret_name: String,
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
pub(super) struct WebdavListEntry {
    name: String,
    path: String,
    kind: String,
    size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    mtime: Option<i64>,
}

#[derive(Debug, Serialize)]
pub(super) struct WebdavListResponse {
    path: String,
    entries: Vec<WebdavListEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total: Option<u64>,
}

pub(super) async fn webdav_list(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(node_id): Path<String>,
    Query(req): Query<WebdavListRequest>,
) -> Result<Json<WebdavListResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    webdav_list_impl(&state, node_id, req).await
}

pub(super) async fn webdav_list_post(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(node_id): Path<String>,
    Json(req): Json<WebdavListRequest>,
) -> Result<Json<WebdavListResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    webdav_list_impl(&state, node_id, req).await
}

async fn webdav_list_impl(
    state: &AppState,
    node_id: String,
    req: WebdavListRequest,
) -> Result<Json<WebdavListResponse>, AppError> {
    let base_url = req.base_url.trim().to_string();
    if base_url.is_empty() {
        return Err(
            AppError::bad_request("invalid_base_url", "base_url is required")
                .with_reason("required")
                .with_field("base_url"),
        );
    }
    let secret_name = req.secret_name.trim().to_string();
    if secret_name.is_empty() {
        return Err(
            AppError::bad_request("invalid_webdav_secret", "secret_name is required")
                .with_details(serde_json::json!({ "field": "secret_name" })),
        );
    }

    let path = normalize_picker_path(&req.path)?;
    if node_id == HUB_NODE_ID {
        let creds_bytes = secrets_repo::get_secret(
            &state.db,
            &state.secrets,
            HUB_NODE_ID,
            "webdav",
            &secret_name,
        )
        .await?
        .ok_or_else(|| {
            AppError::bad_request("missing_webdav_secret", "WebDAV credential not found")
                .with_details(serde_json::json!({ "field": "secret_name" }))
        })?;
        let credentials = WebdavCredentials::from_json(&creds_bytes).map_err(|e| {
            AppError::bad_request(
                "invalid_webdav_secret",
                format!("Invalid WebDAV secret payload: {e}"),
            )
            .with_details(serde_json::json!({ "field": "secret_name" }))
        })?;

        let (entries, next_cursor, total) =
            list_webdav_on_hub(&base_url, credentials, &path, req).await?;

        return Ok(Json(WebdavListResponse {
            path,
            entries,
            next_cursor,
            total: Some(total),
        }));
    }

    if !state.agent_manager.is_connected(&node_id).await {
        return Err(AppError::conflict("agent_offline", "Agent is offline"));
    }

    let opts = bastion_engine::agent_manager::WebdavListOptions {
        cursor: req.cursor,
        limit: req.limit,
        q: req.q,
        kind: req.kind,
        hide_dotfiles: req.hide_dotfiles.unwrap_or(false),
        type_sort: req.type_sort,
        sort_by: req.sort_by,
        sort_dir: req.sort_dir,
        size_min_bytes: req.size_min_bytes,
        size_max_bytes: req.size_max_bytes,
    };

    let page = state
        .agent_manager
        .webdav_list_page(
            &node_id,
            base_url.clone(),
            secret_name.clone(),
            path.clone(),
            opts,
            Duration::from_secs(5),
        )
        .await
        .map_err(|error| map_agent_webdav_list_error(&path, error))?;

    Ok(Json(WebdavListResponse {
        path,
        entries: page
            .entries
            .into_iter()
            .map(|e| WebdavListEntry {
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

fn classify_legacy_remote_webdav_error(message: &str) -> Option<&'static str> {
    let msg = message.to_ascii_lowercase();
    if msg.contains("not a directory") {
        return Some("not_directory");
    }
    if msg.contains("no such file") || msg.contains("not found") || msg.contains("cannot find the")
    {
        return Some("path_not_found");
    }
    if msg.contains("permission denied") || msg.contains("access is denied") {
        return Some("permission_denied");
    }
    None
}

fn map_agent_webdav_list_error(path: &str, error: anyhow::Error) -> AppError {
    if let Some(e) = error.downcast_ref::<WebdavListRemoteError>() {
        let remote_code = e.code.trim().to_string();
        let message = e.message.trim().to_string();

        let (mapped_code, mapped_from_legacy_message) = match remote_code.as_str() {
            "permission_denied" | "path_not_found" | "not_directory" | "invalid_cursor" => {
                (remote_code.as_str(), false)
            }
            _ => match classify_legacy_remote_webdav_error(&message) {
                Some(mapped) => (mapped, true),
                None => ("agent_webdav_list_failed", false),
            },
        };

        let mut err = match mapped_code {
            "permission_denied" => AppError::forbidden("permission_denied", "Permission denied"),
            "path_not_found" => AppError::not_found("path_not_found", "Path not found"),
            "not_directory" => AppError::bad_request("not_directory", "path is not a directory"),
            "invalid_cursor" => AppError::bad_request("invalid_cursor", "invalid cursor"),
            _ => AppError::bad_request(
                "agent_webdav_list_failed",
                format!("Agent WebDAV list failed: {message}"),
            ),
        };

        let mut details = serde_json::Map::new();
        details.insert(
            "path".to_string(),
            serde_json::Value::String(path.to_string()),
        );
        details.insert(
            "agent_error_code".to_string(),
            serde_json::Value::String(remote_code.clone()),
        );
        if mapped_from_legacy_message {
            details.insert(
                "agent_error_code_mapped".to_string(),
                serde_json::Value::String(mapped_code.to_string()),
            );
        }

        err = err.with_details(serde_json::Value::Object(details));
        return err;
    }

    AppError::bad_request(
        "agent_webdav_list_failed",
        format!("Agent WebDAV list failed: {error}"),
    )
    .with_details(serde_json::json!({ "path": path }))
}

async fn list_webdav_on_hub(
    base_url: &str,
    credentials: WebdavCredentials,
    path: &str,
    req: WebdavListRequest,
) -> Result<(Vec<WebdavListEntry>, Option<String>, u64), AppError> {
    let mut base_url = Url::parse(base_url).map_err(|_| {
        AppError::bad_request("invalid_base_url", "invalid base_url")
            .with_reason("invalid_format")
            .with_field("base_url")
    })?;
    if !base_url.path().ends_with('/') {
        base_url.set_path(&format!("{}/", base_url.path()));
    }

    let mut list_url = base_url.clone();
    {
        let mut segs = list_url.path_segments_mut().map_err(|_| {
            AppError::bad_request("invalid_base_url", "base_url cannot be a base")
                .with_reason("cannot_be_base")
                .with_field("base_url")
        })?;
        for part in path
            .trim_matches('/')
            .split('/')
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            if part == "." || part == ".." {
                return Err(AppError::bad_request(
                    "invalid_path",
                    "invalid path segment",
                ));
            }
            segs.push(part);
        }
    }
    if !list_url.path().ends_with('/') {
        list_url.set_path(&format!("{}/", list_url.path()));
    }

    let client = WebdavClient::new(base_url, credentials)
        .map_err(|e| AppError::bad_request("webdav_list_failed", e.to_string()))?;

    let propfind = client
        .propfind_depth1(&list_url)
        .await
        .map_err(|error| map_webdav_list_error(path, error))?;

    let page = page_webdav_entries(path, propfind, req)?;
    Ok((page.entries, page.next_cursor, page.total))
}

fn map_webdav_list_error(path: &str, error: anyhow::Error) -> AppError {
    if let Some(_e) = error.downcast_ref::<WebdavNotDirectoryError>() {
        return AppError::bad_request("not_directory", "path is not a directory")
            .with_details(serde_json::json!({ "path": path }));
    }

    if let Some(e) = error.downcast_ref::<WebdavHttpError>() {
        let mut out = match e.status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                AppError::forbidden("permission_denied", "Permission denied")
            }
            StatusCode::NOT_FOUND => AppError::not_found("path_not_found", "Path not found"),
            _ => AppError::bad_request("webdav_list_failed", format!("WebDAV list failed: {e}")),
        };
        out = out.with_details(serde_json::json!({ "path": path, "status": e.status.as_u16() }));
        return out;
    }

    AppError::bad_request("webdav_list_failed", format!("WebDAV list failed: {error}"))
        .with_details(serde_json::json!({ "path": path }))
}

#[derive(Debug)]
struct WebdavListPage {
    entries: Vec<WebdavListEntry>,
    next_cursor: Option<String>,
    total: u64,
}

fn page_webdav_entries(
    path: &str,
    entries: Vec<WebdavPropfindEntry>,
    req: WebdavListRequest,
) -> Result<WebdavListPage, AppError> {
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

    let cursor = req.cursor.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let q = req.q.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let kind_filter = req.kind.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let type_sort = req.type_sort.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    let sort_by = parse_sort_by(req.sort_by)?;
    let sort_dir = parse_sort_dir(req.sort_dir)?;

    let needle = q.as_deref().map(|v| v.to_lowercase());
    let kind_filter = kind_filter.as_deref();
    let type_sort = type_sort.as_deref();
    let min_bytes = req.size_min_bytes;
    let max_bytes = req.size_max_bytes;
    let size_filter_active = min_bytes.is_some() || max_bytes.is_some();

    let limit = req.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT) as usize;
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

    let mut total: u64 = 0;
    let mut after_cursor_total: u64 = 0;
    let mut heap = std::collections::BinaryHeap::<Candidate>::new();

    for entry in entries {
        let name = entry.name.trim();
        if name.is_empty() || name == "/" {
            continue;
        }

        if req.hide_dotfiles.unwrap_or(false) && name.starts_with('.') {
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
            && key.cmp(cursor_key) != std::cmp::Ordering::Greater
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

    let mut selected: Vec<Candidate> = heap.into_vec();
    selected.sort();

    let entries = selected
        .into_iter()
        .map(|c| WebdavListEntry {
            name: c.key.name.clone(),
            path: join_picker_path(path, &c.key.name),
            kind: c.kind,
            size: c.size,
            mtime: c.mtime,
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

    Ok(WebdavListPage {
        entries,
        next_cursor,
        total,
    })
}

fn join_picker_path(base: &str, name: &str) -> String {
    if base == "/" {
        format!("/{name}")
    } else {
        format!("{}/{}", base.trim_end_matches('/'), name)
    }
}

fn normalize_picker_path(path: &str) -> Result<String, AppError> {
    let path = path.trim();
    if path.is_empty() {
        return Err(AppError::bad_request("invalid_path", "path is required")
            .with_details(serde_json::json!({ "field": "path" })));
    }

    let mut out = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    if out != "/" {
        out = out.trim_end_matches('/').to_string();
    }
    debug!(path = %out, "webdav picker path normalized");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_agent_webdav_list_error_uses_structured_remote_code() {
        let err = WebdavListRemoteError {
            code: "not_directory".to_string(),
            message: "legacy text should not matter".to_string(),
        };
        let app = map_agent_webdav_list_error("/tmp/file", anyhow::Error::new(err));
        assert_eq!(app.code(), "not_directory");
        assert_eq!(app.status(), axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(
            app.details().and_then(|v| v.get("agent_error_code")),
            Some(&serde_json::Value::String("not_directory".to_string()))
        );
    }

    #[test]
    fn map_agent_webdav_list_error_legacy_message_falls_back_by_message() {
        let err = WebdavListRemoteError {
            code: "error".to_string(),
            message: "Permission denied for /data".to_string(),
        };
        let app = map_agent_webdav_list_error("/data", anyhow::Error::new(err));
        assert_eq!(app.code(), "permission_denied");
        assert_eq!(
            app.details().and_then(|v| v.get("agent_error_code_mapped")),
            Some(&serde_json::Value::String("permission_denied".to_string()))
        );
    }

    #[tokio::test]
    async fn list_webdav_on_hub_invalid_base_url_has_reason_invalid_format() {
        let err = list_webdav_on_hub(
            "not-a-url",
            WebdavCredentials {
                username: "u".to_string(),
                password: "p".to_string(),
            },
            "/",
            WebdavListRequest {
                base_url: "not-a-url".to_string(),
                secret_name: "s".to_string(),
                path: "/".to_string(),
                cursor: None,
                limit: None,
                q: None,
                kind: None,
                hide_dotfiles: None,
                type_sort: None,
                sort_by: None,
                sort_dir: None,
                size_min_bytes: None,
                size_max_bytes: None,
            },
        )
        .await
        .expect_err("invalid base_url should fail");

        assert_eq!(err.code(), "invalid_base_url");
        assert_eq!(
            err.details().and_then(|v| v.get("reason")),
            Some(&serde_json::Value::String("invalid_format".to_string()))
        );
        assert_eq!(
            err.details().and_then(|v| v.get("field")),
            Some(&serde_json::Value::String("base_url".to_string()))
        );
    }

    #[tokio::test]
    async fn list_webdav_on_hub_base_url_cannot_be_base_has_structured_reason() {
        let err = list_webdav_on_hub(
            "mailto:user@example.com",
            WebdavCredentials {
                username: "u".to_string(),
                password: "p".to_string(),
            },
            "/",
            WebdavListRequest {
                base_url: "mailto:user@example.com".to_string(),
                secret_name: "s".to_string(),
                path: "/".to_string(),
                cursor: None,
                limit: None,
                q: None,
                kind: None,
                hide_dotfiles: None,
                type_sort: None,
                sort_by: None,
                sort_dir: None,
                size_min_bytes: None,
                size_max_bytes: None,
            },
        )
        .await
        .expect_err("non-hierarchical base_url should fail");

        assert_eq!(err.code(), "invalid_base_url");
        assert_eq!(
            err.details().and_then(|v| v.get("reason")),
            Some(&serde_json::Value::String("cannot_be_base".to_string()))
        );
        assert_eq!(
            err.details().and_then(|v| v.get("field")),
            Some(&serde_json::Value::String("base_url".to_string()))
        );
    }
}
