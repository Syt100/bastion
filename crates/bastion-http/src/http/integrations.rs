use axum::Json;
use axum::extract::State;
use serde::Serialize;
use sqlx::Row;
use tower_cookies::Cookies;

use super::shared::require_session;
use super::{AppError, AppState};

#[derive(Debug, Serialize)]
struct IntegrationsDomainSummary<T> {
    state: String,
    summary: T,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
struct DistributionSummary {
    coverage_total: i64,
    drifted_total: i64,
    failed_total: i64,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct IntegrationsSummaryResponse {
    storage: IntegrationsDomainSummary<StorageSummary>,
    notifications: IntegrationsDomainSummary<NotificationsSummary>,
    distribution: IntegrationsDomainSummary<DistributionSummary>,
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

async fn load_storage_summary(db: &sqlx::SqlitePool) -> Result<StorageSummary, anyhow::Error> {
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

async fn load_distribution_summary(
    db: &sqlx::SqlitePool,
) -> Result<DistributionSummary, anyhow::Error> {
    let row = sqlx::query(
        r#"
        SELECT
          SUM(CASE WHEN revoked_at IS NULL THEN 1 ELSE 0 END) AS coverage_total,
          SUM(
            CASE
              WHEN revoked_at IS NULL
               AND last_config_sync_error_kind IS NULL
               AND desired_config_snapshot_id IS NOT NULL
               AND applied_config_snapshot_id != desired_config_snapshot_id
              THEN 1 ELSE 0
            END
          ) AS drifted_total,
          SUM(
            CASE
              WHEN revoked_at IS NULL AND last_config_sync_error_kind IS NOT NULL
              THEN 1 ELSE 0
            END
          ) AS failed_total
        FROM agents
        "#,
    )
    .fetch_one(db)
    .await?;

    Ok(DistributionSummary {
        coverage_total: row.get::<Option<i64>, _>("coverage_total").unwrap_or(0),
        drifted_total: row.get::<Option<i64>, _>("drifted_total").unwrap_or(0),
        failed_total: row.get::<Option<i64>, _>("failed_total").unwrap_or(0),
    })
}

pub(in crate::http) async fn get_integrations_summary(
    state: State<AppState>,
    cookies: Cookies,
) -> Result<Json<IntegrationsSummaryResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let storage_summary = match load_storage_summary(&state.db).await {
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
