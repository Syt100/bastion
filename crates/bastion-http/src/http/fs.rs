use std::path::PathBuf;
use std::time::Duration;

use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;

use bastion_core::HUB_NODE_ID;

use super::shared::require_session;
use super::{AppError, AppState};

#[derive(Debug, Deserialize)]
pub(super) struct FsListQuery {
    path: String,
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
        let entries = tokio::task::spawn_blocking(move || list_dir_entries(&path_for_worker))
            .await
            .map_err(|e| anyhow::anyhow!(e))??;
        return Ok(Json(FsListResponse { path, entries }));
    }

    if !state.agent_manager.is_connected(&node_id).await {
        return Err(AppError::conflict("agent_offline", "Agent is offline"));
    }

    let entries = state
        .agent_manager
        .fs_list(&node_id, path.to_string(), Duration::from_secs(5))
        .await
        .map_err(|error| {
            AppError::bad_request("agent_fs_list_failed", format!("Agent filesystem list failed: {error}"))
        })?;

    Ok(Json(FsListResponse {
        path: path.to_string(),
        entries: entries
            .into_iter()
            .map(|e| FsListEntry {
                name: e.name,
                path: e.path,
                kind: e.kind,
                size: e.size,
                mtime: e.mtime,
            })
            .collect(),
    }))
}

fn list_dir_entries(path: &str) -> Result<Vec<FsListEntry>, anyhow::Error> {
    use std::time::UNIX_EPOCH;

    let dir = PathBuf::from(path);
    let meta = std::fs::metadata(&dir)?;
    if !meta.is_dir() {
        anyhow::bail!("path is not a directory");
    }

    let mut out = Vec::<FsListEntry>::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.trim().is_empty() {
            continue;
        }

        let path = entry.path().to_string_lossy().to_string();
        let ft = entry.file_type()?;
        let kind = if ft.is_dir() {
            "dir"
        } else if ft.is_file() {
            "file"
        } else if ft.is_symlink() {
            "symlink"
        } else {
            "other"
        };

        let meta = entry.metadata().ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let mtime = meta.and_then(|m| m.modified().ok()).and_then(|t| {
            t.duration_since(UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs() as i64)
        });

        out.push(FsListEntry {
            name,
            path,
            kind: kind.to_string(),
            size,
            mtime,
        });
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}
