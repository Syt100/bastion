use sqlx::{Row, SqlitePool};

use bastion_core::HUB_NODE_ID;

use super::super::AppError;

pub(super) async fn validate_node_id(db: &SqlitePool, node_id: &str) -> Result<(), AppError> {
    fn invalid_node_id_error(reason: &'static str, message: impl Into<String>) -> AppError {
        AppError::bad_request("invalid_node_id", message)
            .with_reason(reason)
            .with_field("node_id")
    }

    let node_id = node_id.trim();
    if node_id.is_empty() {
        return Err(invalid_node_id_error("required", "Node id is required"));
    }
    if node_id == HUB_NODE_ID {
        return Ok(());
    }

    let row = sqlx::query("SELECT revoked_at FROM agents WHERE id = ? LIMIT 1")
        .bind(node_id)
        .fetch_optional(db)
        .await?;

    let Some(row) = row else {
        return Err(invalid_node_id_error("not_found", "Node not found"));
    };
    if row.get::<Option<i64>, _>("revoked_at").is_some() {
        return Err(invalid_node_id_error("revoked", "Node is revoked"));
    }
    Ok(())
}
