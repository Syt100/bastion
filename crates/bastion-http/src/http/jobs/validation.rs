use sqlx::SqlitePool;

use bastion_core::HUB_NODE_ID;
use bastion_core::job_spec;
use bastion_storage::secrets_repo;

use super::super::AppError;

pub(super) fn validate_job_spec(spec: &serde_json::Value) -> Result<(), AppError> {
    job_spec::validate_value(spec).map_err(|error| {
        AppError::bad_request("invalid_spec", format!("Invalid job spec: {error}"))
    })
}

pub(super) async fn validate_job_target_scope(
    db: &SqlitePool,
    agent_id: Option<&str>,
    spec: &serde_json::Value,
) -> Result<(), AppError> {
    let node_id = agent_id.unwrap_or(HUB_NODE_ID);

    let parsed = job_spec::parse_value(spec).map_err(|error| {
        AppError::bad_request("invalid_spec", format!("Invalid job spec: {error}"))
    })?;

    let target = match &parsed {
        job_spec::JobSpecV1::Filesystem { target, .. } => target,
        job_spec::JobSpecV1::Sqlite { target, .. } => target,
        job_spec::JobSpecV1::Vaultwarden { target, .. } => target,
    };

    if let job_spec::TargetV1::Webdav { secret_name, .. } = target {
        let secret_name = secret_name.trim();
        if secret_name.is_empty() {
            return Err(AppError::bad_request(
                "invalid_webdav_secret",
                "WebDAV credential name is required",
            ));
        }

        let exists = secrets_repo::secret_exists(db, node_id, "webdav", secret_name).await?;
        if !exists {
            return Err(AppError::bad_request(
                "invalid_webdav_secret",
                "WebDAV credential not found",
            )
            .with_details(serde_json::json!({ "field": "spec.target.secret_name" })));
        }
    }

    Ok(())
}
