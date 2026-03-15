use axum::Json;
use axum::extract::State;
use serde::Serialize;
use tower_cookies::Cookies;

use super::shared::require_session;
use super::{AppError, AppState, ConfigValueSource};

#[derive(Debug, Serialize)]
pub(in crate::http) struct PublicMetadataResponse {
    public_base_url: Option<String>,
    source: ConfigValueSource,
    command_generation_ready: bool,
}

pub(in crate::http) async fn get_public_metadata(
    state: State<AppState>,
    cookies: Cookies,
) -> Result<Json<PublicMetadataResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    Ok(Json(PublicMetadataResponse {
        public_base_url: state.hub_runtime_config.public_base_url.clone(),
        source: state.hub_runtime_config.sources.public_base_url,
        command_generation_ready: state.hub_runtime_config.public_base_url.is_some(),
    }))
}
