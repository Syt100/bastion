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
