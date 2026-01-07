use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use serde::Deserialize;
use tower_cookies::Cookies;

use bastion_backup::restore;

use super::shared::require_session;
use super::{AppError, AppState};

#[derive(Debug, Deserialize)]
pub(super) struct ListRunEntriesQuery {
    #[serde(default)]
    prefix: Option<String>,
    #[serde(default)]
    cursor: Option<u64>,
    #[serde(default)]
    limit: Option<u64>,
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    hide_dotfiles: Option<bool>,
    #[serde(default)]
    min_size_bytes: Option<u64>,
    #[serde(default)]
    max_size_bytes: Option<u64>,
    #[serde(default)]
    type_sort: Option<String>,
}

pub(super) async fn list_run_entries(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(run_id): Path<String>,
    Query(query): Query<ListRunEntriesQuery>,
) -> Result<Json<restore::RunEntriesChildrenResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let prefix = query.prefix.as_deref();
    let cursor = query.cursor.unwrap_or(0);
    let limit = query.limit.unwrap_or(200);
    let q = query.q.as_deref().map(str::trim).filter(|v| !v.is_empty());
    let kind = query
        .kind
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let kind = match kind {
        None => None,
        Some(v) if matches!(v, "file" | "dir" | "symlink") => Some(v),
        Some(_) => {
            return Err(
                AppError::bad_request("invalid_kind", "invalid kind")
                    .with_details(serde_json::json!({ "field": "kind" })),
            );
        }
    };
    let hide_dotfiles = query.hide_dotfiles.unwrap_or(false);
    let (min_size_bytes, max_size_bytes) = match (query.min_size_bytes, query.max_size_bytes) {
        (Some(a), Some(b)) if a > b => (Some(b), Some(a)),
        other => other,
    };
    let type_sort = query.type_sort.as_deref().map(str::trim).filter(|v| !v.is_empty());
    let type_sort_file_first = match type_sort {
        None | Some("dir_first") => false,
        Some("file_first") => true,
        Some(_) => {
            return Err(
                AppError::bad_request("invalid_type_sort", "invalid type_sort")
                    .with_details(serde_json::json!({ "field": "type_sort" })),
            );
        }
    };

    let result = restore::list_run_entries_children(
        &state.db,
        state.secrets.as_ref(),
        &state.config.data_dir,
        &run_id,
        prefix,
        cursor,
        limit,
        q,
        kind,
        hide_dotfiles,
        min_size_bytes,
        max_size_bytes,
        type_sort_file_first,
    )
    .await;

    match result {
        Ok(v) => Ok(Json(v)),
        Err(error) => {
            let msg = format!("{error:#}");
            if msg.contains("run not found") {
                return Err(AppError::not_found("run_not_found", "Run not found"));
            }
            Err(AppError::bad_request(
                "run_entries_failed",
                format!("Run entries list failed: {error}"),
            ))
        }
    }
}
