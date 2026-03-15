use std::collections::HashMap;

use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tower_cookies::Cookies;

use bastion_core::HUB_NODE_ID;

use super::shared::require_session;
use super::{AppError, AppState};

#[derive(Debug, Serialize)]
struct IntegrationsDomainSummary<T> {
    state: String,
    summary: T,
}

#[derive(Debug, Clone, Serialize)]
struct StorageSummary {
    items_total: i64,
    in_use_total: i64,
    invalid_total: i64,
}

#[derive(Debug, Serialize)]
struct NotificationsSummary {
    destinations_total: i64,
    recent_failures_total: i64,
    queue_backlog_total: i64,
}

#[derive(Debug, Clone, Serialize)]
struct DistributionSummary {
    coverage_total: i64,
    drifted_total: i64,
    failed_total: i64,
    offline_total: i64,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct IntegrationsSummaryResponse {
    storage: IntegrationsDomainSummary<StorageSummary>,
    notifications: IntegrationsDomainSummary<NotificationsSummary>,
    distribution: IntegrationsDomainSummary<DistributionSummary>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct StorageUsageRef {
    job_id: String,
    job_name: String,
    latest_run_id: Option<String>,
    latest_run_status: Option<String>,
    latest_run_at: Option<i64>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct StorageHealthSummary {
    state: String,
    latest_run_id: Option<String>,
    latest_run_status: Option<String>,
    latest_run_at: Option<i64>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct StorageIntegrationItem {
    name: String,
    updated_at: i64,
    usage_total: i64,
    usage: Vec<StorageUsageRef>,
    health: StorageHealthSummary,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct StorageDetailsResponse {
    node_id: String,
    summary: StorageSummary,
    items: Vec<StorageIntegrationItem>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct DistributionScopeItem {
    agent_id: String,
    agent_name: Option<String>,
    connection_status: String,
    distribution_state: String,
    pending_tasks_total: i64,
    last_attempt_at: Option<i64>,
    last_error_kind: Option<String>,
    last_error: Option<String>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct DistributionDetailsResponse {
    summary: DistributionSummary,
    items: Vec<DistributionScopeItem>,
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct StorageDetailsQuery {
    node_id: Option<String>,
}

fn classify_storage_state(summary: &StorageSummary) -> &'static str {
    if summary.items_total == 0 && summary.in_use_total == 0 {
        "empty"
    } else if summary.invalid_total > 0 {
        "degraded"
    } else {
        "ready"
    }
}

fn classify_notifications_state(summary: &NotificationsSummary) -> &'static str {
    if summary.destinations_total == 0
        && summary.queue_backlog_total == 0
        && summary.recent_failures_total == 0
    {
        "empty"
    } else if summary.recent_failures_total > 0 {
        "degraded"
    } else {
        "ready"
    }
}

fn classify_distribution_state(summary: &DistributionSummary) -> &'static str {
    if summary.coverage_total == 0 {
        "empty"
    } else if summary.drifted_total > 0 || summary.failed_total > 0 {
        "degraded"
    } else {
        "ready"
    }
}

fn invalid_node_id_error(reason: &'static str, message: impl Into<String>) -> AppError {
    AppError::bad_request("invalid_node_id", message)
        .with_reason(reason)
        .with_field("node_id")
}

async fn resolve_node_id(
    db: &sqlx::SqlitePool,
    requested_node_id: Option<&str>,
) -> Result<String, AppError> {
    let Some(node_id) = requested_node_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(HUB_NODE_ID.to_string());
    };

    if node_id == HUB_NODE_ID {
        return Ok(HUB_NODE_ID.to_string());
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

    Ok(node_id.to_string())
}

fn storage_health_state(latest_run_status: Option<&str>, usage_total: usize) -> &'static str {
    if usage_total == 0 {
        return "unused";
    }

    match latest_run_status {
        Some("success") => "healthy",
        Some("failed" | "rejected" | "canceled") => "attention",
        Some("queued" | "running") => "progressing",
        Some(_) => "configured",
        None => "configured",
    }
}

fn agent_online(last_seen_at: Option<i64>, now: i64) -> bool {
    let cutoff = now.saturating_sub(60);
    last_seen_at.is_some_and(|ts| ts >= cutoff)
}

fn distribution_state(
    desired_snapshot_id: Option<&str>,
    applied_snapshot_id: Option<&str>,
    last_error_kind: Option<&str>,
) -> &'static str {
    if last_error_kind.is_some() {
        return "failed";
    }
    if desired_snapshot_id.is_none() || applied_snapshot_id != desired_snapshot_id {
        return "drifted";
    }
    "covered"
}

async fn load_global_storage_summary(
    db: &sqlx::SqlitePool,
) -> Result<StorageSummary, anyhow::Error> {
    let items_total =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM secrets WHERE kind = 'webdav'")
            .fetch_one(db)
            .await?;

    let in_use_total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(DISTINCT TRIM(COALESCE(json_extract(spec_json, '$.target.secret_name'), '')))
        FROM jobs
        WHERE archived_at IS NULL
          AND json_extract(spec_json, '$.target.type') = 'webdav'
          AND TRIM(COALESCE(json_extract(spec_json, '$.target.secret_name'), '')) <> ''
        "#,
    )
    .fetch_one(db)
    .await?;

    let invalid_total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM jobs j
        WHERE j.archived_at IS NULL
          AND json_extract(j.spec_json, '$.target.type') = 'webdav'
          AND (
            TRIM(COALESCE(json_extract(j.spec_json, '$.target.secret_name'), '')) = ''
            OR NOT EXISTS (
              SELECT 1
              FROM secrets s
              WHERE s.kind = 'webdav'
                AND s.name = TRIM(COALESCE(json_extract(j.spec_json, '$.target.secret_name'), ''))
                AND s.node_id = COALESCE(j.agent_id, 'hub')
            )
          )
        "#,
    )
    .fetch_one(db)
    .await?;

    Ok(StorageSummary {
        items_total,
        in_use_total,
        invalid_total,
    })
}

async fn load_storage_summary_for_node(
    db: &sqlx::SqlitePool,
    node_id: &str,
) -> Result<StorageSummary, anyhow::Error> {
    let items_total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM secrets WHERE kind = 'webdav' AND node_id = ?",
    )
    .bind(node_id)
    .fetch_one(db)
    .await?;

    let in_use_total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(DISTINCT TRIM(COALESCE(json_extract(spec_json, '$.target.secret_name'), '')))
        FROM jobs
        WHERE archived_at IS NULL
          AND json_extract(spec_json, '$.target.type') = 'webdav'
          AND COALESCE(agent_id, 'hub') = ?
          AND TRIM(COALESCE(json_extract(spec_json, '$.target.secret_name'), '')) <> ''
        "#,
    )
    .bind(node_id)
    .fetch_one(db)
    .await?;

    let invalid_total = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM jobs j
        WHERE j.archived_at IS NULL
          AND json_extract(j.spec_json, '$.target.type') = 'webdav'
          AND COALESCE(j.agent_id, 'hub') = ?
          AND (
            TRIM(COALESCE(json_extract(j.spec_json, '$.target.secret_name'), '')) = ''
            OR NOT EXISTS (
              SELECT 1
              FROM secrets s
              WHERE s.kind = 'webdav'
                AND s.name = TRIM(COALESCE(json_extract(j.spec_json, '$.target.secret_name'), ''))
                AND s.node_id = ?
            )
          )
        "#,
    )
    .bind(node_id)
    .bind(node_id)
    .fetch_one(db)
    .await?;

    Ok(StorageSummary {
        items_total,
        in_use_total,
        invalid_total,
    })
}

async fn load_notifications_summary(
    db: &sqlx::SqlitePool,
) -> Result<NotificationsSummary, anyhow::Error> {
    let destinations_total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM secrets WHERE node_id = 'hub' AND kind IN ('wecom_bot', 'smtp')",
    )
    .fetch_one(db)
    .await?;
    let recent_failures_total =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notifications WHERE status = 'failed'")
            .fetch_one(db)
            .await?;
    let queue_backlog_total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM notifications WHERE status IN ('queued', 'sending')",
    )
    .fetch_one(db)
    .await?;

    Ok(NotificationsSummary {
        destinations_total,
        recent_failures_total,
        queue_backlog_total,
    })
}

async fn load_storage_details(
    db: &sqlx::SqlitePool,
    node_id: &str,
) -> Result<StorageDetailsResponse, anyhow::Error> {
    let summary = load_storage_summary_for_node(db, node_id).await?;

    let secret_rows = sqlx::query(
        "SELECT name, updated_at FROM secrets WHERE node_id = ? AND kind = 'webdav' ORDER BY name ASC",
    )
    .bind(node_id)
    .fetch_all(db)
    .await?;

    let usage_rows = sqlx::query(
        r#"
        SELECT
          TRIM(COALESCE(json_extract(j.spec_json, '$.target.secret_name'), '')) AS secret_name,
          j.id AS job_id,
          j.name AS job_name,
          latest.id AS latest_run_id,
          latest.status AS latest_run_status,
          COALESCE(latest.ended_at, latest.started_at) AS latest_run_at
        FROM jobs j
        LEFT JOIN runs latest ON latest.id = (
          SELECT r2.id
          FROM runs r2
          WHERE r2.job_id = j.id
          ORDER BY COALESCE(r2.ended_at, r2.started_at, 0) DESC, r2.id DESC
          LIMIT 1
        )
        WHERE j.archived_at IS NULL
          AND json_extract(j.spec_json, '$.target.type') = 'webdav'
          AND COALESCE(j.agent_id, 'hub') = ?
          AND TRIM(COALESCE(json_extract(j.spec_json, '$.target.secret_name'), '')) <> ''
        ORDER BY COALESCE(latest.ended_at, latest.started_at, 0) DESC, j.updated_at DESC, j.id ASC
        "#,
    )
    .bind(node_id)
    .fetch_all(db)
    .await?;

    let mut usage_by_secret: HashMap<String, Vec<StorageUsageRef>> = HashMap::new();
    for row in usage_rows {
        let secret_name = row.get::<String, _>("secret_name");
        usage_by_secret
            .entry(secret_name)
            .or_default()
            .push(StorageUsageRef {
                job_id: row.get::<String, _>("job_id"),
                job_name: row.get::<String, _>("job_name"),
                latest_run_id: row.get::<Option<String>, _>("latest_run_id"),
                latest_run_status: row.get::<Option<String>, _>("latest_run_status"),
                latest_run_at: row.get::<Option<i64>, _>("latest_run_at"),
            });
    }

    let items = secret_rows
        .into_iter()
        .map(|row| {
            let name = row.get::<String, _>("name");
            let usage = usage_by_secret.remove(&name).unwrap_or_default();
            let usage_total = usage.len();
            let latest_usage = usage.first();
            let latest_run_id = latest_usage.and_then(|usage| usage.latest_run_id.clone());
            let latest_run_status = latest_usage.and_then(|usage| usage.latest_run_status.clone());
            let latest_run_at = latest_usage.and_then(|usage| usage.latest_run_at);
            let health_state =
                storage_health_state(latest_run_status.as_deref(), usage_total).to_string();

            StorageIntegrationItem {
                name,
                updated_at: row.get::<i64, _>("updated_at"),
                usage_total: usage_total as i64,
                usage,
                health: StorageHealthSummary {
                    state: health_state,
                    latest_run_id,
                    latest_run_status,
                    latest_run_at,
                },
            }
        })
        .collect();

    Ok(StorageDetailsResponse {
        node_id: node_id.to_string(),
        summary,
        items,
    })
}

fn summarize_distribution_items(items: &[DistributionScopeItem]) -> DistributionSummary {
    DistributionSummary {
        coverage_total: items.len() as i64,
        drifted_total: items
            .iter()
            .filter(|item| item.distribution_state == "drifted")
            .count() as i64,
        failed_total: items
            .iter()
            .filter(|item| item.distribution_state == "failed")
            .count() as i64,
        offline_total: items
            .iter()
            .filter(|item| item.connection_status == "offline")
            .count() as i64,
    }
}

async fn load_distribution_items(
    db: &sqlx::SqlitePool,
) -> Result<Vec<DistributionScopeItem>, anyhow::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
          a.id,
          a.name,
          a.last_seen_at,
          a.desired_config_snapshot_id,
          a.applied_config_snapshot_id,
          a.last_config_sync_attempt_at,
          a.last_config_sync_error_kind,
          a.last_config_sync_error,
          COALESCE(task_counts.total, 0) AS pending_tasks_total
        FROM agents a
        LEFT JOIN (
          SELECT agent_id, COUNT(*) AS total
          FROM agent_tasks
          WHERE completed_at IS NULL
          GROUP BY agent_id
        ) AS task_counts ON task_counts.agent_id = a.id
        WHERE a.revoked_at IS NULL
        ORDER BY a.created_at DESC, a.id ASC
        "#,
    )
    .fetch_all(db)
    .await?;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    Ok(rows
        .into_iter()
        .map(|row| DistributionScopeItem {
            agent_id: row.get::<String, _>("id"),
            agent_name: row.get::<Option<String>, _>("name"),
            connection_status: if agent_online(row.get::<Option<i64>, _>("last_seen_at"), now) {
                "online".to_string()
            } else {
                "offline".to_string()
            },
            distribution_state: distribution_state(
                row.get::<Option<String>, _>("desired_config_snapshot_id")
                    .as_deref(),
                row.get::<Option<String>, _>("applied_config_snapshot_id")
                    .as_deref(),
                row.get::<Option<String>, _>("last_config_sync_error_kind")
                    .as_deref(),
            )
            .to_string(),
            pending_tasks_total: row.get::<i64, _>("pending_tasks_total"),
            last_attempt_at: row.get::<Option<i64>, _>("last_config_sync_attempt_at"),
            last_error_kind: row.get::<Option<String>, _>("last_config_sync_error_kind"),
            last_error: row.get::<Option<String>, _>("last_config_sync_error"),
        })
        .collect())
}

async fn load_distribution_summary(
    db: &sqlx::SqlitePool,
) -> Result<DistributionSummary, anyhow::Error> {
    let items = load_distribution_items(db).await?;
    Ok(summarize_distribution_items(&items))
}

pub(in crate::http) async fn get_integrations_summary(
    state: State<AppState>,
    cookies: Cookies,
) -> Result<Json<IntegrationsSummaryResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let storage_summary = match load_global_storage_summary(&state.db).await {
        Ok(summary) => summary,
        Err(error) => {
            tracing::warn!(error = %error, "failed to build integrations storage summary");
            StorageSummary {
                items_total: 0,
                in_use_total: 0,
                invalid_total: 0,
            }
        }
    };
    let notifications_summary = match load_notifications_summary(&state.db).await {
        Ok(summary) => summary,
        Err(error) => {
            tracing::warn!(error = %error, "failed to build integrations notifications summary");
            NotificationsSummary {
                destinations_total: 0,
                recent_failures_total: 0,
                queue_backlog_total: 0,
            }
        }
    };
    let distribution_summary = match load_distribution_summary(&state.db).await {
        Ok(summary) => summary,
        Err(error) => {
            tracing::warn!(error = %error, "failed to build integrations distribution summary");
            DistributionSummary {
                coverage_total: 0,
                drifted_total: 0,
                failed_total: 0,
                offline_total: 0,
            }
        }
    };

    Ok(Json(IntegrationsSummaryResponse {
        storage: IntegrationsDomainSummary {
            state: classify_storage_state(&storage_summary).to_string(),
            summary: storage_summary,
        },
        notifications: IntegrationsDomainSummary {
            state: classify_notifications_state(&notifications_summary).to_string(),
            summary: notifications_summary,
        },
        distribution: IntegrationsDomainSummary {
            state: classify_distribution_state(&distribution_summary).to_string(),
            summary: distribution_summary,
        },
    }))
}

pub(in crate::http) async fn get_storage_details(
    state: State<AppState>,
    cookies: Cookies,
    Query(query): Query<StorageDetailsQuery>,
) -> Result<Json<StorageDetailsResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let node_id = resolve_node_id(&state.db, query.node_id.as_deref()).await?;
    let response = load_storage_details(&state.db, &node_id)
        .await
        .map_err(AppError::from)?;
    Ok(Json(response))
}

pub(in crate::http) async fn get_distribution_details(
    state: State<AppState>,
    cookies: Cookies,
) -> Result<Json<DistributionDetailsResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let items = load_distribution_items(&state.db)
        .await
        .map_err(AppError::from)?;
    let summary = summarize_distribution_items(&items);
    Ok(Json(DistributionDetailsResponse { summary, items }))
}
