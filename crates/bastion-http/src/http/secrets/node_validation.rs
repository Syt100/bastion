use serde_json::json;
use sqlx::{Row, SqlitePool};

use bastion_core::HUB_NODE_ID;

use super::super::AppError;

pub(super) async fn validate_node_id(db: &SqlitePool, node_id: &str) -> Result<(), AppError> {
    let node_id = node_id.trim();
    if node_id.is_empty() {
        return Err(
            AppError::bad_request("invalid_node_id", "Node id is required")
                .with_details(json!({ "field": "node_id" })),
        );
    }
    if node_id == HUB_NODE_ID {
        return Ok(());
    }

    let row = sqlx::query("SELECT revoked_at FROM agents WHERE id = ? LIMIT 1")
        .bind(node_id)
        .fetch_optional(db)
        .await?;

    let Some(row) = row else {
        return Err(AppError::bad_request("invalid_node_id", "Node not found"));
    };
    if row.get::<Option<i64>, _>("revoked_at").is_some() {
        return Err(AppError::bad_request("invalid_node_id", "Node is revoked"));
    }
    Ok(())
}
