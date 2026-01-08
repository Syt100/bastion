use serde_json::json;
use sqlx::SqlitePool;

use bastion_core::HUB_NODE_ID;
use bastion_storage::notification_destinations_repo;
use bastion_storage::notifications_repo;

use super::super::AppError;

pub(super) fn require_supported_channel(channel: &str) -> Result<(), AppError> {
    if channel == notifications_repo::CHANNEL_WECOM_BOT
        || channel == notifications_repo::CHANNEL_EMAIL
    {
        return Ok(());
    }
    Err(
        AppError::bad_request("invalid_channel", "Unsupported notification channel")
            .with_details(json!({ "field": "channel" })),
    )
}

pub(super) async fn destination_exists(
    db: &SqlitePool,
    channel: &str,
    name: &str,
) -> Result<bool, anyhow::Error> {
    let Some(kind) = notification_destinations_repo::secret_kind_for_channel(channel) else {
        return Ok(false);
    };
    let row =
        sqlx::query("SELECT 1 FROM secrets WHERE node_id = ? AND kind = ? AND name = ? LIMIT 1")
            .bind(HUB_NODE_ID)
            .bind(kind)
            .bind(name)
            .fetch_optional(db)
            .await?;
    Ok(row.is_some())
}
