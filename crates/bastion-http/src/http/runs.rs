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

    let result = restore::list_run_entries_children(
        &state.db,
        state.secrets.as_ref(),
        &state.config.data_dir,
        &run_id,
        prefix,
        cursor,
        limit,
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
