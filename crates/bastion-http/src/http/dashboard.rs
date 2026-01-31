use std::collections::HashMap;

use axum::Json;
use serde::Serialize;
use sqlx::Row;
use tower_cookies::Cookies;

use bastion_storage::runs_repo;

use super::shared::require_session;
use super::{AppError, AppState};

#[derive(Debug, Serialize)]
pub(in crate::http) struct DashboardOverviewResponse {
    stats: DashboardStats,
    trend_7d: Vec<DashboardTrendDay>,
    recent_runs: Vec<DashboardRecentRun>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct DashboardStats {
    agents: DashboardAgentsStats,
    jobs: DashboardJobsStats,
    runs: DashboardRunsStats,
    notifications: DashboardNotificationsStats,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct DashboardAgentsStats {
    total: i64,
    active: i64,
    online: i64,
    offline: i64,
    revoked: i64,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct DashboardJobsStats {
    active: i64,
    archived: i64,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct DashboardRunsStats {
    running: i64,
    queued: i64,
    success_24h: i64,
    failed_24h: i64,
    rejected_24h: i64,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct DashboardNotificationsStats {
    queued: i64,
    sending: i64,
    failed: i64,
    canceled: i64,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct DashboardTrendDay {
    // YYYY-MM-DD (UTC).
    day: String,
    success: i64,
    failed: i64,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct DashboardRecentRun {
    run_id: String,
    job_id: String,
    job_name: String,
    // "hub" or agent id.
    node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    node_name: Option<String>,
    status: runs_repo::RunStatus,
    started_at: i64,
    ended_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    executed_offline: bool,
}

pub(in crate::http) async fn get_overview(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<DashboardOverviewResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let now = time::OffsetDateTime::now_utc();
    let now_ts = now.unix_timestamp();
    let online_cutoff = now_ts.saturating_sub(60);
    let cutoff_24h = now_ts.saturating_sub(24 * 60 * 60);

    // Agents (online uses the same 60s cutoff as the Agents list page).
    let row = sqlx::query(
        r#"
        SELECT
          COUNT(1) AS total,
          COALESCE(SUM(CASE WHEN revoked_at IS NULL THEN 1 ELSE 0 END), 0) AS active,
          COALESCE(SUM(CASE WHEN revoked_at IS NOT NULL THEN 1 ELSE 0 END), 0) AS revoked,
          COALESCE(SUM(
            CASE
              WHEN revoked_at IS NULL AND last_seen_at IS NOT NULL AND last_seen_at >= ? THEN 1
              ELSE 0
            END
          ), 0) AS online
        FROM agents
        "#,
    )
    .bind(online_cutoff)
    .fetch_one(&state.db)
    .await?;

    let agents_total = row.get::<i64, _>("total");
    let agents_active = row.get::<i64, _>("active");
    let agents_revoked = row.get::<i64, _>("revoked");
    let agents_online = row.get::<i64, _>("online");
    let agents_offline = agents_active.saturating_sub(agents_online);

    // Jobs.
    let row = sqlx::query(
        r#"
        SELECT
          COALESCE(SUM(CASE WHEN archived_at IS NULL THEN 1 ELSE 0 END), 0) AS active,
          COALESCE(SUM(CASE WHEN archived_at IS NOT NULL THEN 1 ELSE 0 END), 0) AS archived
        FROM jobs
        "#,
    )
    .fetch_one(&state.db)
    .await?;

    let jobs_active = row.get::<i64, _>("active");
    let jobs_archived = row.get::<i64, _>("archived");

    // Runs (live + last 24h).
    let row = sqlx::query(
        r#"
        SELECT
          COALESCE(SUM(CASE WHEN status = 'running' THEN 1 ELSE 0 END), 0) AS running,
          COALESCE(SUM(CASE WHEN status = 'queued' THEN 1 ELSE 0 END), 0) AS queued,
          COALESCE(SUM(CASE WHEN ended_at IS NOT NULL AND ended_at >= ? AND status = 'success' THEN 1 ELSE 0 END), 0) AS success_24h,
          COALESCE(SUM(CASE WHEN ended_at IS NOT NULL AND ended_at >= ? AND status = 'failed' THEN 1 ELSE 0 END), 0) AS failed_24h,
          COALESCE(SUM(CASE WHEN ended_at IS NOT NULL AND ended_at >= ? AND status = 'rejected' THEN 1 ELSE 0 END), 0) AS rejected_24h
        FROM runs
        "#,
    )
    .bind(cutoff_24h)
    .bind(cutoff_24h)
    .bind(cutoff_24h)
    .fetch_one(&state.db)
    .await?;

    let runs_running = row.get::<i64, _>("running");
    let runs_queued = row.get::<i64, _>("queued");
    let runs_success_24h = row.get::<i64, _>("success_24h");
    let runs_failed_24h = row.get::<i64, _>("failed_24h");
    let runs_rejected_24h = row.get::<i64, _>("rejected_24h");

    // Notifications queue (exclude sent).
    let row = sqlx::query(
        r#"
        SELECT
          COALESCE(SUM(CASE WHEN status = 'queued' THEN 1 ELSE 0 END), 0) AS queued,
          COALESCE(SUM(CASE WHEN status = 'sending' THEN 1 ELSE 0 END), 0) AS sending,
          COALESCE(SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END), 0) AS failed,
          COALESCE(SUM(CASE WHEN status = 'canceled' THEN 1 ELSE 0 END), 0) AS canceled
        FROM notifications
        WHERE status IN ('queued', 'sending', 'failed', 'canceled')
        "#,
    )
    .fetch_one(&state.db)
    .await?;

    let notifications_queued = row.get::<i64, _>("queued");
    let notifications_sending = row.get::<i64, _>("sending");
    let notifications_failed = row.get::<i64, _>("failed");
    let notifications_canceled = row.get::<i64, _>("canceled");

    // Trend: last 7 days (UTC), inclusive.
    let today = now.date();
    let start = today - time::Duration::days(6);
    let start_ts = start.midnight().assume_utc().unix_timestamp();

    let rows = sqlx::query(
        r#"
        SELECT
          date(datetime(ended_at, 'unixepoch')) AS day,
          SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) AS success,
          SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) AS failed
        FROM runs
        WHERE ended_at IS NOT NULL
          AND ended_at >= ?
          AND status IN ('success', 'failed')
        GROUP BY day
        ORDER BY day ASC
        "#,
    )
    .bind(start_ts)
    .fetch_all(&state.db)
    .await?;

    let mut map: HashMap<String, (i64, i64)> = HashMap::new();
    for row in rows {
        let day = row.get::<String, _>("day");
        let success = row.get::<i64, _>("success");
        let failed = row.get::<i64, _>("failed");
        map.insert(day, (success, failed));
    }

    let mut trend_7d: Vec<DashboardTrendDay> = Vec::with_capacity(7);
    let mut d = start;
    for _ in 0..7 {
        let key = d.to_string(); // YYYY-MM-DD
        let (success, failed) = map.get(&key).copied().unwrap_or((0, 0));
        trend_7d.push(DashboardTrendDay {
            day: key,
            success,
            failed,
        });
        d = d.next_day().expect("valid date");
    }

    // Recent runs.
    let rows = sqlx::query(
        r#"
        SELECT
          r.id AS run_id,
          r.job_id AS job_id,
          r.status AS status,
          r.started_at AS started_at,
          r.ended_at AS ended_at,
          r.error AS error,
          r.summary_json AS summary_json,
          j.name AS job_name,
          j.agent_id AS agent_id,
          a.name AS agent_name
        FROM runs r
        JOIN jobs j ON j.id = r.job_id
        LEFT JOIN agents a ON a.id = j.agent_id
        ORDER BY r.started_at DESC
        LIMIT 20
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut recent_runs: Vec<DashboardRecentRun> = Vec::with_capacity(rows.len());
    for row in rows {
        let status = row
            .get::<String, _>("status")
            .parse::<runs_repo::RunStatus>()?;
        let summary_json = row.get::<Option<String>, _>("summary_json");
        let executed_offline = summary_json
            .as_deref()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .and_then(|v| v.get("executed_offline").and_then(|v| v.as_bool()))
            .unwrap_or(false);

        let agent_id = row.get::<Option<String>, _>("agent_id");
        let (node_id, node_name) = match agent_id {
            None => ("hub".to_string(), None),
            Some(id) => (id, row.get::<Option<String>, _>("agent_name")),
        };

        recent_runs.push(DashboardRecentRun {
            run_id: row.get::<String, _>("run_id"),
            job_id: row.get::<String, _>("job_id"),
            job_name: row.get::<String, _>("job_name"),
            node_id,
            node_name,
            status,
            started_at: row.get::<i64, _>("started_at"),
            ended_at: row.get::<Option<i64>, _>("ended_at"),
            error: row.get::<Option<String>, _>("error"),
            executed_offline,
        });
    }

    Ok(Json(DashboardOverviewResponse {
        stats: DashboardStats {
            agents: DashboardAgentsStats {
                total: agents_total,
                active: agents_active,
                online: agents_online,
                offline: agents_offline,
                revoked: agents_revoked,
            },
            jobs: DashboardJobsStats {
                active: jobs_active,
                archived: jobs_archived,
            },
            runs: DashboardRunsStats {
                running: runs_running,
                queued: runs_queued,
                success_24h: runs_success_24h,
                failed_24h: runs_failed_24h,
                rejected_24h: runs_rejected_24h,
            },
            notifications: DashboardNotificationsStats {
                queued: notifications_queued,
                sending: notifications_sending,
                failed: notifications_failed,
                canceled: notifications_canceled,
            },
        },
        trend_7d,
        recent_runs,
    }))
}
