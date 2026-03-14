use axum::Json;
use axum::extract::Query;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tower_cookies::Cookies;
use url::form_urlencoded::Serializer;

use super::shared::require_session;
use super::{AppError, AppState};

#[derive(Debug, Deserialize)]
pub(super) struct CommandCenterQuery {
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    range: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SectionState {
    Ready,
    Empty,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Severity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReadinessOverall {
    Healthy,
    Degraded,
    Empty,
}

#[derive(Debug, Serialize)]
pub(super) struct CommandCenterResponse {
    generated_at: i64,
    scope: CommandCenterScopeEcho,
    range: CommandCenterRangeEcho,
    attention: CommandCenterSection<CommandCenterItem>,
    critical_activity: CommandCenterSection<CommandCenterItem>,
    recovery_readiness: RecoveryReadiness,
    watchlist: CommandCenterSection<CommandCenterItem>,
}

#[derive(Debug, Serialize)]
struct CommandCenterScopeEcho {
    requested: String,
    effective: String,
}

#[derive(Debug, Serialize)]
struct CommandCenterRangeEcho {
    preset: String,
    from: i64,
    to: i64,
}

#[derive(Debug, Serialize)]
struct CommandCenterSection<T> {
    state: SectionState,
    items: Vec<T>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

#[derive(Debug, Serialize)]
struct CommandCenterItem {
    id: String,
    kind: String,
    severity: Severity,
    title: String,
    summary: String,
    occurred_at: i64,
    scope: String,
    context: CommandCenterItemContext,
    primary_action: CommandCenterAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    secondary_action: Option<CommandCenterAction>,
}

#[derive(Debug, Default, Serialize)]
struct CommandCenterItemContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    job_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    job_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    node_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    operation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    notification_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    channel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct CommandCenterAction {
    label: String,
    href: String,
}

#[derive(Debug, Serialize)]
struct RecoveryReadiness {
    state: SectionState,
    overall: ReadinessOverall,
    backup: RecoveryReadinessSignal,
    verify: RecoveryReadinessSignal,
    blockers: Vec<RecoveryReadinessBlocker>,
}

#[derive(Debug, Serialize)]
struct RecoveryReadinessSignal {
    recent_success_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recent_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recent_job_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recent_job_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recent_operation_id: Option<String>,
    active_jobs: i64,
    covered_jobs: i64,
}

#[derive(Debug, Serialize)]
struct RecoveryReadinessBlocker {
    kind: String,
    title: String,
    summary: String,
    href: String,
}

#[derive(Debug, Clone)]
enum RequestedScope {
    All,
    Hub,
    Agent(String),
}

impl RequestedScope {
    fn parse(raw: Option<&str>) -> Result<Self, AppError> {
        let Some(raw) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
            return Ok(Self::All);
        };

        if raw == "all" {
            return Ok(Self::All);
        }
        if raw == "hub" {
            return Ok(Self::Hub);
        }
        if let Some(agent_id) = raw.strip_prefix("agent:").map(str::trim)
            && !agent_id.is_empty()
        {
            return Ok(Self::Agent(agent_id.to_string()));
        }

        Err(
            AppError::bad_request("invalid_scope", "invalid scope")
                .with_reason("unsupported_value")
                .with_field("scope"),
        )
    }

    fn as_str(&self) -> String {
        match self {
            Self::All => "all".to_string(),
            Self::Hub => "hub".to_string(),
            Self::Agent(agent_id) => format!("agent:{agent_id}"),
        }
    }

    fn includes_agent(&self, agent_id: Option<&str>) -> bool {
        match self {
            Self::All => true,
            Self::Hub => agent_id.is_none(),
            Self::Agent(expected) => agent_id == Some(expected.as_str()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ResolvedRange {
    preset: &'static str,
    from: i64,
    to: i64,
}

impl ResolvedRange {
    fn parse(raw: Option<&str>, now_ts: i64) -> Result<Self, AppError> {
        let raw = raw.map(str::trim).filter(|value| !value.is_empty()).unwrap_or("24h");
        let (preset, seconds) = match raw {
            "24h" => ("24h", 24 * 60 * 60),
            "7d" => ("7d", 7 * 24 * 60 * 60),
            "30d" => ("30d", 30 * 24 * 60 * 60),
            _ => {
                return Err(
                    AppError::bad_request("invalid_range", "invalid range")
                        .with_reason("unsupported_value")
                        .with_field("range"),
                );
            }
        };

        Ok(Self {
            preset,
            from: now_ts.saturating_sub(seconds),
            to: now_ts,
        })
    }
}

#[derive(Debug)]
struct AttentionRunRow {
    run_id: String,
    job_id: String,
    job_name: String,
    agent_id: Option<String>,
    agent_name: Option<String>,
    status: String,
    occurred_at: i64,
    error: Option<String>,
}

#[derive(Debug)]
struct NotificationFailureRow {
    notification_id: String,
    run_id: String,
    job_id: String,
    job_name: String,
    agent_id: Option<String>,
    agent_name: Option<String>,
    channel: String,
    occurred_at: i64,
    error: Option<String>,
}

#[derive(Debug)]
struct AgentIssueRow {
    agent_id: String,
    agent_name: Option<String>,
    revoked_at: Option<i64>,
    last_seen_at: Option<i64>,
    created_at: i64,
}

#[derive(Debug)]
struct ActivityRunRow {
    run_id: String,
    job_id: String,
    job_name: String,
    agent_id: Option<String>,
    agent_name: Option<String>,
    status: String,
    started_at: i64,
    ended_at: Option<i64>,
    error: Option<String>,
}

#[derive(Debug)]
struct ActivityOperationRow {
    operation_id: String,
    kind: String,
    status: String,
    run_id: String,
    job_id: String,
    job_name: String,
    agent_id: Option<String>,
    agent_name: Option<String>,
    started_at: i64,
    ended_at: Option<i64>,
    error: Option<String>,
}

#[derive(Debug)]
struct ActiveJobRow {
    job_id: String,
    agent_id: Option<String>,
}

#[derive(Debug)]
struct JobSignalRow {
    job_id: String,
    latest_success_at: Option<i64>,
}

#[derive(Debug)]
struct LatestBackupRow {
    run_id: String,
    job_id: String,
    job_name: String,
    agent_id: Option<String>,
    ended_at: i64,
}

#[derive(Debug)]
struct LatestVerifyRow {
    operation_id: String,
    run_id: String,
    job_id: String,
    job_name: String,
    agent_id: Option<String>,
    ended_at: i64,
}

pub(super) async fn get_command_center(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Query(query): Query<CommandCenterQuery>,
) -> Result<Json<CommandCenterResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let now_ts = time::OffsetDateTime::now_utc().unix_timestamp();
    let requested_scope = RequestedScope::parse(query.scope.as_deref())?;
    let resolved_range = ResolvedRange::parse(query.range.as_deref(), now_ts)?;

    let attention = section_or_degraded(build_attention(&state, &requested_scope, resolved_range).await);
    let critical_activity =
        section_or_degraded(build_critical_activity(&state, &requested_scope, resolved_range).await);
    let recovery_readiness =
        recovery_readiness_or_degraded(build_recovery_readiness(&state, &requested_scope).await);
    let watchlist = section_or_degraded(build_watchlist(&state, &requested_scope, resolved_range).await);

    Ok(Json(CommandCenterResponse {
        generated_at: now_ts,
        scope: CommandCenterScopeEcho {
            requested: requested_scope.as_str(),
            effective: requested_scope.as_str(),
        },
        range: CommandCenterRangeEcho {
            preset: resolved_range.preset.to_string(),
            from: resolved_range.from,
            to: resolved_range.to,
        },
        attention,
        critical_activity,
        recovery_readiness,
        watchlist,
    }))
}

fn section_or_degraded<T>(
    result: anyhow::Result<CommandCenterSection<T>>,
) -> CommandCenterSection<T> {
    match result {
        Ok(section) => section,
        Err(error) => {
            tracing::warn!(error = ?error, "command center section degraded");
            CommandCenterSection {
                state: SectionState::Degraded,
                items: Vec::new(),
                note: Some("section_unavailable".to_string()),
            }
        }
    }
}

fn recovery_readiness_or_degraded(result: anyhow::Result<RecoveryReadiness>) -> RecoveryReadiness {
    match result {
        Ok(readiness) => readiness,
        Err(error) => {
            tracing::warn!(error = ?error, "command center readiness degraded");
            RecoveryReadiness {
                state: SectionState::Degraded,
                overall: ReadinessOverall::Degraded,
                backup: RecoveryReadinessSignal {
                    recent_success_at: None,
                    recent_run_id: None,
                    recent_job_id: None,
                    recent_job_name: None,
                    recent_operation_id: None,
                    active_jobs: 0,
                    covered_jobs: 0,
                },
                verify: RecoveryReadinessSignal {
                    recent_success_at: None,
                    recent_run_id: None,
                    recent_job_id: None,
                    recent_job_name: None,
                    recent_operation_id: None,
                    active_jobs: 0,
                    covered_jobs: 0,
                },
                blockers: vec![RecoveryReadinessBlocker {
                    kind: "section_unavailable".to_string(),
                    title: "Recovery readiness is unavailable".to_string(),
                    summary: "The server could not assemble readiness signals for this scope.".to_string(),
                    href: "/system/runtime".to_string(),
                }],
            }
        }
    }
}

async fn build_attention(
    state: &AppState,
    scope: &RequestedScope,
    range: ResolvedRange,
) -> anyhow::Result<CommandCenterSection<CommandCenterItem>> {
    let failed_runs = load_failed_runs(state, range).await?;
    let failed_notifications = load_failed_notifications(state, range).await?;
    let agent_issues = load_agent_issues(state).await?;

    let mut items = Vec::new();

    for row in failed_runs {
        if !scope.includes_agent(row.agent_id.as_deref()) {
            continue;
        }

        let severity = if row.status == "failed" {
            Severity::Critical
        } else {
            Severity::Warning
        };
        let scope_value = job_scope_string(row.agent_id.as_deref());
        let primary_href = run_detail_href(&row.run_id, &scope_value);
        let secondary_href = jobs_scope_href(&scope_value);
        items.push(CommandCenterItem {
            id: format!("run:{}", row.run_id),
            kind: format!("run_{}", row.status),
            severity,
            title: format!("{} needs review", row.job_name),
            summary: row
                .error
                .clone()
                .unwrap_or_else(|| format!("Latest run finished as {}.", row.status)),
            occurred_at: row.occurred_at,
            scope: scope_value.clone(),
            context: CommandCenterItemContext {
                run_id: Some(row.run_id.clone()),
                job_id: Some(row.job_id.clone()),
                job_name: Some(row.job_name.clone()),
                node_id: Some(node_id_for_agent(row.agent_id.as_deref())),
                node_name: node_name_for_agent(row.agent_id.as_deref(), row.agent_name.as_deref()),
                status: Some(row.status.clone()),
                error: row.error.clone(),
                ..Default::default()
            },
            primary_action: CommandCenterAction {
                label: "Open run".to_string(),
                href: primary_href,
            },
            secondary_action: Some(CommandCenterAction {
                label: "Open jobs".to_string(),
                href: secondary_href,
            }),
        });
    }

    for row in failed_notifications {
        if !scope.includes_agent(row.agent_id.as_deref()) {
            continue;
        }

        let scope_value = job_scope_string(row.agent_id.as_deref());
        items.push(CommandCenterItem {
            id: format!("notification:{}", row.notification_id),
            kind: "notification_failed".to_string(),
            severity: Severity::Warning,
            title: format!("Notification delivery failed for {}", row.job_name),
            summary: row
                .error
                .clone()
                .unwrap_or_else(|| format!("{} destination needs retry.", row.channel)),
            occurred_at: row.occurred_at,
            scope: scope_value.clone(),
            context: CommandCenterItemContext {
                run_id: Some(row.run_id.clone()),
                job_id: Some(row.job_id.clone()),
                job_name: Some(row.job_name.clone()),
                node_id: Some(node_id_for_agent(row.agent_id.as_deref())),
                node_name: node_name_for_agent(row.agent_id.as_deref(), row.agent_name.as_deref()),
                notification_id: Some(row.notification_id.clone()),
                channel: Some(row.channel.clone()),
                error: row.error.clone(),
                ..Default::default()
            },
            primary_action: CommandCenterAction {
                label: "Open queue".to_string(),
                href: notifications_queue_href("failed"),
            },
            secondary_action: Some(CommandCenterAction {
                label: "Open run".to_string(),
                href: run_detail_href(&row.run_id, &scope_value),
            }),
        });
    }

    for row in agent_issues {
        if !scope.includes_agent(Some(row.agent_id.as_str())) {
            continue;
        }

        let (kind, severity, title, summary, occurred_at) = if let Some(revoked_at) = row.revoked_at
        {
            (
                "agent_revoked",
                Severity::Critical,
                format!("{} is revoked", row.agent_name.as_deref().unwrap_or(&row.agent_id)),
                "This agent must be re-enrolled before it can receive work again.".to_string(),
                revoked_at,
            )
        } else {
            (
                "agent_offline",
                Severity::Warning,
                format!("{} is offline", row.agent_name.as_deref().unwrap_or(&row.agent_id)),
                "The Hub has not heard from this agent in the normal heartbeat window.".to_string(),
                row.last_seen_at.unwrap_or(row.created_at),
            )
        };

        items.push(CommandCenterItem {
            id: format!("agent:{}", row.agent_id),
            kind: kind.to_string(),
            severity,
            title,
            summary,
            occurred_at,
            scope: format!("agent:{}", row.agent_id),
            context: CommandCenterItemContext {
                node_id: Some(row.agent_id.clone()),
                node_name: Some(row.agent_name.clone().unwrap_or_else(|| row.agent_id.clone())),
                ..Default::default()
            },
            primary_action: CommandCenterAction {
                label: "Open fleet".to_string(),
                href: fleet_href(if kind == "agent_revoked" { "revoked" } else { "offline" }),
            },
            secondary_action: None,
        });
    }

    items.sort_by(|left, right| {
        severity_rank(left.severity)
            .cmp(&severity_rank(right.severity))
            .then_with(|| right.occurred_at.cmp(&left.occurred_at))
    });
    items.truncate(8);

    Ok(section_from_items(items))
}

async fn build_critical_activity(
    state: &AppState,
    scope: &RequestedScope,
    range: ResolvedRange,
) -> anyhow::Result<CommandCenterSection<CommandCenterItem>> {
    let recent_runs = load_recent_runs(state, range).await?;
    let recent_operations = load_recent_operations(state, range).await?;

    let mut items = Vec::new();

    for row in recent_runs {
        if !scope.includes_agent(row.agent_id.as_deref()) {
            continue;
        }

        let occurred_at = row.ended_at.unwrap_or(row.started_at);
        let scope_value = job_scope_string(row.agent_id.as_deref());
        let severity = match row.status.as_str() {
            "failed" => Severity::Critical,
            "rejected" => Severity::Warning,
            "running" => Severity::Info,
            _ => Severity::Info,
        };
        items.push(CommandCenterItem {
            id: format!("activity-run:{}", row.run_id),
            kind: format!("run_{}", row.status),
            severity,
            title: format!("{} {}", row.job_name, activity_suffix_for_run(&row.status)),
            summary: row
                .error
                .clone()
                .unwrap_or_else(|| format!("{} scope activity.", scope_label_from_agent(row.agent_id.as_deref()))),
            occurred_at,
            scope: scope_value.clone(),
            context: CommandCenterItemContext {
                run_id: Some(row.run_id.clone()),
                job_id: Some(row.job_id.clone()),
                job_name: Some(row.job_name.clone()),
                node_id: Some(node_id_for_agent(row.agent_id.as_deref())),
                node_name: node_name_for_agent(row.agent_id.as_deref(), row.agent_name.as_deref()),
                status: Some(row.status.clone()),
                error: row.error.clone(),
                ..Default::default()
            },
            primary_action: CommandCenterAction {
                label: "Open run".to_string(),
                href: run_detail_href(&row.run_id, &scope_value),
            },
            secondary_action: Some(CommandCenterAction {
                label: "Open jobs".to_string(),
                href: jobs_scope_href(&scope_value),
            }),
        });
    }

    for row in recent_operations {
        if !scope.includes_agent(row.agent_id.as_deref()) {
            continue;
        }

        let occurred_at = row.ended_at.unwrap_or(row.started_at);
        let scope_value = job_scope_string(row.agent_id.as_deref());
        let severity = match row.status.as_str() {
            "failed" => Severity::Critical,
            "canceled" => Severity::Warning,
            _ => Severity::Info,
        };
        items.push(CommandCenterItem {
            id: format!("activity-op:{}", row.operation_id),
            kind: format!("operation_{}_{}", row.kind, row.status),
            severity,
            title: format!(
                "{} {}",
                row.job_name,
                activity_suffix_for_operation(&row.kind, &row.status)
            ),
            summary: row
                .error
                .clone()
                .unwrap_or_else(|| format!("{} operation linked to this run.", row.kind)),
            occurred_at,
            scope: scope_value.clone(),
            context: CommandCenterItemContext {
                run_id: Some(row.run_id.clone()),
                job_id: Some(row.job_id.clone()),
                job_name: Some(row.job_name.clone()),
                node_id: Some(node_id_for_agent(row.agent_id.as_deref())),
                node_name: node_name_for_agent(row.agent_id.as_deref(), row.agent_name.as_deref()),
                operation_id: Some(row.operation_id.clone()),
                status: Some(row.status.clone()),
                error: row.error.clone(),
                ..Default::default()
            },
            primary_action: CommandCenterAction {
                label: "Open run".to_string(),
                href: run_detail_href(&row.run_id, &scope_value),
            },
            secondary_action: None,
        });
    }

    items.sort_by(|left, right| {
        activity_rank(&left.kind)
            .cmp(&activity_rank(&right.kind))
            .then_with(|| right.occurred_at.cmp(&left.occurred_at))
    });
    items.truncate(8);

    Ok(section_from_items(items))
}

async fn build_watchlist(
    state: &AppState,
    scope: &RequestedScope,
    range: ResolvedRange,
) -> anyhow::Result<CommandCenterSection<CommandCenterItem>> {
    let recent_runs = load_recent_runs(state, range).await?;
    let recent_operations = load_recent_operations(state, range).await?;
    let queued_notifications = load_queued_notifications(state, range).await?;

    let mut items = Vec::new();

    for row in recent_runs {
        if !scope.includes_agent(row.agent_id.as_deref()) {
            continue;
        }
        if row.status != "running" && row.status != "queued" {
            continue;
        }

        let scope_value = job_scope_string(row.agent_id.as_deref());
        items.push(CommandCenterItem {
            id: format!("watch-run:{}", row.run_id),
            kind: format!("run_{}", row.status),
            severity: Severity::Info,
            title: format!("{} {}", row.job_name, activity_suffix_for_run(&row.status)),
            summary: "This run is still active and worth following.".to_string(),
            occurred_at: row.started_at,
            scope: scope_value.clone(),
            context: CommandCenterItemContext {
                run_id: Some(row.run_id.clone()),
                job_id: Some(row.job_id.clone()),
                job_name: Some(row.job_name.clone()),
                node_id: Some(node_id_for_agent(row.agent_id.as_deref())),
                node_name: node_name_for_agent(row.agent_id.as_deref(), row.agent_name.as_deref()),
                status: Some(row.status.clone()),
                ..Default::default()
            },
            primary_action: CommandCenterAction {
                label: "Open run".to_string(),
                href: run_detail_href(&row.run_id, &scope_value),
            },
            secondary_action: None,
        });
    }

    for row in recent_operations {
        if !scope.includes_agent(row.agent_id.as_deref()) {
            continue;
        }
        if row.status != "running" {
            continue;
        }

        let scope_value = job_scope_string(row.agent_id.as_deref());
        items.push(CommandCenterItem {
            id: format!("watch-op:{}", row.operation_id),
            kind: format!("operation_{}_running", row.kind),
            severity: Severity::Info,
            title: format!("{} {}", row.job_name, activity_suffix_for_operation(&row.kind, &row.status)),
            summary: "An operator workflow is still in progress for this backup.".to_string(),
            occurred_at: row.started_at,
            scope: scope_value.clone(),
            context: CommandCenterItemContext {
                run_id: Some(row.run_id.clone()),
                job_id: Some(row.job_id.clone()),
                job_name: Some(row.job_name.clone()),
                node_id: Some(node_id_for_agent(row.agent_id.as_deref())),
                node_name: node_name_for_agent(row.agent_id.as_deref(), row.agent_name.as_deref()),
                operation_id: Some(row.operation_id.clone()),
                status: Some(row.status.clone()),
                ..Default::default()
            },
            primary_action: CommandCenterAction {
                label: "Open run".to_string(),
                href: run_detail_href(&row.run_id, &scope_value),
            },
            secondary_action: None,
        });
    }

    for row in queued_notifications {
        if !scope.includes_agent(row.agent_id.as_deref()) {
            continue;
        }

        let scope_value = job_scope_string(row.agent_id.as_deref());
        items.push(CommandCenterItem {
            id: format!("watch-notification:{}", row.notification_id),
            kind: "notification_queued".to_string(),
            severity: Severity::Info,
            title: format!("{} notification pending", row.job_name),
            summary: format!("{} delivery is still queued.", row.channel),
            occurred_at: row.occurred_at,
            scope: scope_value.clone(),
            context: CommandCenterItemContext {
                run_id: Some(row.run_id.clone()),
                job_id: Some(row.job_id.clone()),
                job_name: Some(row.job_name.clone()),
                node_id: Some(node_id_for_agent(row.agent_id.as_deref())),
                node_name: node_name_for_agent(row.agent_id.as_deref(), row.agent_name.as_deref()),
                notification_id: Some(row.notification_id.clone()),
                channel: Some(row.channel.clone()),
                ..Default::default()
            },
            primary_action: CommandCenterAction {
                label: "Open queue".to_string(),
                href: notifications_queue_href("queued"),
            },
            secondary_action: None,
        });
    }

    items.sort_by(|left, right| right.occurred_at.cmp(&left.occurred_at));
    items.truncate(6);

    Ok(section_from_items(items))
}

async fn build_recovery_readiness(
    state: &AppState,
    scope: &RequestedScope,
) -> anyhow::Result<RecoveryReadiness> {
    let active_jobs = load_active_jobs(state).await?;
    let backup_coverage = load_backup_job_signals(state).await?;
    let verify_coverage = load_verify_job_signals(state).await?;
    let latest_backup = load_latest_successful_backup(state).await?;
    let latest_verify = load_latest_successful_verify(state).await?;

    let scoped_jobs: Vec<_> = active_jobs
        .into_iter()
        .filter(|row| scope.includes_agent(row.agent_id.as_deref()))
        .collect();
    let active_jobs_count = scoped_jobs.len() as i64;

    if active_jobs_count == 0 {
        return Ok(RecoveryReadiness {
            state: SectionState::Empty,
            overall: ReadinessOverall::Empty,
            backup: RecoveryReadinessSignal {
                recent_success_at: None,
                recent_run_id: None,
                recent_job_id: None,
                recent_job_name: None,
                recent_operation_id: None,
                active_jobs: 0,
                covered_jobs: 0,
            },
            verify: RecoveryReadinessSignal {
                recent_success_at: None,
                recent_run_id: None,
                recent_job_id: None,
                recent_job_name: None,
                recent_operation_id: None,
                active_jobs: 0,
                covered_jobs: 0,
            },
            blockers: Vec::new(),
        });
    }

    let mut backup_covered_jobs = 0_i64;
    for row in &backup_coverage {
        if row.latest_success_at.is_some()
            && scoped_jobs.iter().any(|job| job.job_id == row.job_id)
        {
            backup_covered_jobs += 1;
        }
    }

    let mut verify_covered_jobs = 0_i64;
    for row in &verify_coverage {
        if row.latest_success_at.is_some()
            && scoped_jobs.iter().any(|job| job.job_id == row.job_id)
        {
            verify_covered_jobs += 1;
        }
    }

    let latest_backup = latest_backup
        .into_iter()
        .find(|row| scope.includes_agent(row.agent_id.as_deref()));
    let latest_verify = latest_verify
        .into_iter()
        .find(|row| scope.includes_agent(row.agent_id.as_deref()));

    let mut blockers = Vec::new();

    if latest_backup.is_none() {
        blockers.push(RecoveryReadinessBlocker {
            kind: "missing_backup".to_string(),
            title: "No successful backup is available".to_string(),
            summary: "At least one successful backup is required before recovery can be trusted.".to_string(),
            href: jobs_scope_href(&scope.as_str()),
        });
    } else if backup_covered_jobs < active_jobs_count {
        blockers.push(RecoveryReadinessBlocker {
            kind: "partial_backup_coverage".to_string(),
            title: "Some jobs have never completed successfully".to_string(),
            summary: format!(
                "{} of {} active jobs have a successful backup.",
                backup_covered_jobs, active_jobs_count
            ),
            href: jobs_scope_href(&scope.as_str()),
        });
    }

    if latest_verify.is_none() {
        blockers.push(RecoveryReadinessBlocker {
            kind: "missing_verification".to_string(),
            title: "Verification signal is missing".to_string(),
            summary: "Backups exist, but no successful verify operation has been recorded for this scope.".to_string(),
            href: runs_index_href(&scope.as_str()),
        });
    } else if verify_covered_jobs < active_jobs_count {
        blockers.push(RecoveryReadinessBlocker {
            kind: "partial_verification_coverage".to_string(),
            title: "Verification coverage is incomplete".to_string(),
            summary: format!(
                "{} of {} active jobs have a successful verify operation.",
                verify_covered_jobs, active_jobs_count
            ),
            href: runs_index_href(&scope.as_str()),
        });
    }

    if let (Some(backup), Some(verify)) = (&latest_backup, &latest_verify)
        && verify.ended_at < backup.ended_at
    {
        blockers.push(RecoveryReadinessBlocker {
            kind: "verify_older_than_backup".to_string(),
            title: "Verification is older than the latest backup".to_string(),
            summary: "A newer successful backup exists than the newest successful verify signal.".to_string(),
            href: run_detail_href(&backup.run_id, &job_scope_string(backup.agent_id.as_deref())),
        });
    }

    let overall = if blockers.is_empty() {
        ReadinessOverall::Healthy
    } else {
        ReadinessOverall::Degraded
    };

    Ok(RecoveryReadiness {
        state: if blockers.is_empty() {
            SectionState::Ready
        } else {
            SectionState::Degraded
        },
        overall,
        backup: RecoveryReadinessSignal {
            recent_success_at: latest_backup.as_ref().map(|row| row.ended_at),
            recent_run_id: latest_backup.as_ref().map(|row| row.run_id.clone()),
            recent_job_id: latest_backup.as_ref().map(|row| row.job_id.clone()),
            recent_job_name: latest_backup.as_ref().map(|row| row.job_name.clone()),
            recent_operation_id: None,
            active_jobs: active_jobs_count,
            covered_jobs: backup_covered_jobs,
        },
        verify: RecoveryReadinessSignal {
            recent_success_at: latest_verify.as_ref().map(|row| row.ended_at),
            recent_run_id: latest_verify.as_ref().map(|row| row.run_id.clone()),
            recent_job_id: latest_verify.as_ref().map(|row| row.job_id.clone()),
            recent_job_name: latest_verify.as_ref().map(|row| row.job_name.clone()),
            recent_operation_id: latest_verify.as_ref().map(|row| row.operation_id.clone()),
            active_jobs: active_jobs_count,
            covered_jobs: verify_covered_jobs,
        },
        blockers,
    })
}

fn section_from_items<T>(items: Vec<T>) -> CommandCenterSection<T> {
    if items.is_empty() {
        CommandCenterSection {
            state: SectionState::Empty,
            items,
            note: None,
        }
    } else {
        CommandCenterSection {
            state: SectionState::Ready,
            items,
            note: None,
        }
    }
}

async fn load_failed_runs(
    state: &AppState,
    range: ResolvedRange,
) -> anyhow::Result<Vec<AttentionRunRow>> {
    let rows = sqlx::query(
        r#"
        SELECT
          r.id AS run_id,
          r.status AS status,
          r.ended_at AS ended_at,
          r.started_at AS started_at,
          r.error AS error,
          j.id AS job_id,
          j.name AS job_name,
          j.agent_id AS agent_id,
          a.name AS agent_name
        FROM runs r
        JOIN jobs j ON j.id = r.job_id
        LEFT JOIN agents a ON a.id = j.agent_id
        WHERE COALESCE(r.ended_at, r.started_at) >= ?
          AND r.status IN ('failed', 'rejected')
        ORDER BY COALESCE(r.ended_at, r.started_at) DESC
        LIMIT 32
        "#,
    )
    .bind(range.from)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| AttentionRunRow {
            run_id: row.get::<String, _>("run_id"),
            job_id: row.get::<String, _>("job_id"),
            job_name: row.get::<String, _>("job_name"),
            agent_id: row.get::<Option<String>, _>("agent_id"),
            agent_name: row.get::<Option<String>, _>("agent_name"),
            status: row.get::<String, _>("status"),
            occurred_at: row
                .get::<Option<i64>, _>("ended_at")
                .unwrap_or_else(|| row.get::<i64, _>("started_at")),
            error: row.get::<Option<String>, _>("error"),
        })
        .collect())
}

async fn load_failed_notifications(
    state: &AppState,
    range: ResolvedRange,
) -> anyhow::Result<Vec<NotificationFailureRow>> {
    let rows = sqlx::query(
        r#"
        SELECT
          n.id AS notification_id,
          n.run_id AS run_id,
          n.channel AS channel,
          n.updated_at AS updated_at,
          n.last_error AS last_error,
          j.id AS job_id,
          j.name AS job_name,
          j.agent_id AS agent_id,
          a.name AS agent_name
        FROM notifications n
        JOIN runs r ON r.id = n.run_id
        JOIN jobs j ON j.id = r.job_id
        LEFT JOIN agents a ON a.id = j.agent_id
        WHERE n.status = 'failed'
          AND n.updated_at >= ?
        ORDER BY n.updated_at DESC
        LIMIT 20
        "#,
    )
    .bind(range.from)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| NotificationFailureRow {
            notification_id: row.get::<String, _>("notification_id"),
            run_id: row.get::<String, _>("run_id"),
            job_id: row.get::<String, _>("job_id"),
            job_name: row.get::<String, _>("job_name"),
            agent_id: row.get::<Option<String>, _>("agent_id"),
            agent_name: row.get::<Option<String>, _>("agent_name"),
            channel: row.get::<String, _>("channel"),
            occurred_at: row.get::<i64, _>("updated_at"),
            error: row.get::<Option<String>, _>("last_error"),
        })
        .collect())
}

async fn load_agent_issues(state: &AppState) -> anyhow::Result<Vec<AgentIssueRow>> {
    let now_ts = time::OffsetDateTime::now_utc().unix_timestamp();
    let online_cutoff = now_ts.saturating_sub(60);
    let rows = sqlx::query(
        r#"
        SELECT id, name, created_at, revoked_at, last_seen_at
        FROM agents
        WHERE revoked_at IS NOT NULL
           OR last_seen_at IS NULL
           OR last_seen_at < ?
        ORDER BY COALESCE(revoked_at, last_seen_at, created_at) DESC
        LIMIT 20
        "#,
    )
    .bind(online_cutoff)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| AgentIssueRow {
            agent_id: row.get::<String, _>("id"),
            agent_name: row.get::<Option<String>, _>("name"),
            revoked_at: row.get::<Option<i64>, _>("revoked_at"),
            last_seen_at: row.get::<Option<i64>, _>("last_seen_at"),
            created_at: row.get::<i64, _>("created_at"),
        })
        .collect())
}

async fn load_recent_runs(
    state: &AppState,
    range: ResolvedRange,
) -> anyhow::Result<Vec<ActivityRunRow>> {
    let rows = sqlx::query(
        r#"
        SELECT
          r.id AS run_id,
          r.status AS status,
          r.started_at AS started_at,
          r.ended_at AS ended_at,
          r.error AS error,
          j.id AS job_id,
          j.name AS job_name,
          j.agent_id AS agent_id,
          a.name AS agent_name
        FROM runs r
        JOIN jobs j ON j.id = r.job_id
        LEFT JOIN agents a ON a.id = j.agent_id
        WHERE COALESCE(r.ended_at, r.started_at) >= ?
        ORDER BY COALESCE(r.ended_at, r.started_at) DESC
        LIMIT 40
        "#,
    )
    .bind(range.from)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ActivityRunRow {
            run_id: row.get::<String, _>("run_id"),
            job_id: row.get::<String, _>("job_id"),
            job_name: row.get::<String, _>("job_name"),
            agent_id: row.get::<Option<String>, _>("agent_id"),
            agent_name: row.get::<Option<String>, _>("agent_name"),
            status: row.get::<String, _>("status"),
            started_at: row.get::<i64, _>("started_at"),
            ended_at: row.get::<Option<i64>, _>("ended_at"),
            error: row.get::<Option<String>, _>("error"),
        })
        .collect())
}

async fn load_recent_operations(
    state: &AppState,
    range: ResolvedRange,
) -> anyhow::Result<Vec<ActivityOperationRow>> {
    let rows = sqlx::query(
        r#"
        SELECT
          o.id AS operation_id,
          o.kind AS kind,
          o.status AS status,
          o.started_at AS started_at,
          o.ended_at AS ended_at,
          o.error AS error,
          r.id AS run_id,
          j.id AS job_id,
          j.name AS job_name,
          j.agent_id AS agent_id,
          a.name AS agent_name
        FROM operations o
        JOIN runs r ON o.subject_kind = 'run' AND o.subject_id = r.id
        JOIN jobs j ON j.id = r.job_id
        LEFT JOIN agents a ON a.id = j.agent_id
        WHERE COALESCE(o.ended_at, o.started_at) >= ?
          AND o.kind IN ('restore', 'verify')
        ORDER BY COALESCE(o.ended_at, o.started_at) DESC
        LIMIT 24
        "#,
    )
    .bind(range.from)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ActivityOperationRow {
            operation_id: row.get::<String, _>("operation_id"),
            kind: row.get::<String, _>("kind"),
            status: row.get::<String, _>("status"),
            run_id: row.get::<String, _>("run_id"),
            job_id: row.get::<String, _>("job_id"),
            job_name: row.get::<String, _>("job_name"),
            agent_id: row.get::<Option<String>, _>("agent_id"),
            agent_name: row.get::<Option<String>, _>("agent_name"),
            started_at: row.get::<i64, _>("started_at"),
            ended_at: row.get::<Option<i64>, _>("ended_at"),
            error: row.get::<Option<String>, _>("error"),
        })
        .collect())
}

async fn load_queued_notifications(
    state: &AppState,
    range: ResolvedRange,
) -> anyhow::Result<Vec<NotificationFailureRow>> {
    let rows = sqlx::query(
        r#"
        SELECT
          n.id AS notification_id,
          n.run_id AS run_id,
          n.channel AS channel,
          n.updated_at AS updated_at,
          n.last_error AS last_error,
          j.id AS job_id,
          j.name AS job_name,
          j.agent_id AS agent_id,
          a.name AS agent_name
        FROM notifications n
        JOIN runs r ON r.id = n.run_id
        JOIN jobs j ON j.id = r.job_id
        LEFT JOIN agents a ON a.id = j.agent_id
        WHERE n.status IN ('queued', 'sending')
          AND n.updated_at >= ?
        ORDER BY n.updated_at DESC
        LIMIT 16
        "#,
    )
    .bind(range.from)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| NotificationFailureRow {
            notification_id: row.get::<String, _>("notification_id"),
            run_id: row.get::<String, _>("run_id"),
            job_id: row.get::<String, _>("job_id"),
            job_name: row.get::<String, _>("job_name"),
            agent_id: row.get::<Option<String>, _>("agent_id"),
            agent_name: row.get::<Option<String>, _>("agent_name"),
            channel: row.get::<String, _>("channel"),
            occurred_at: row.get::<i64, _>("updated_at"),
            error: row.get::<Option<String>, _>("last_error"),
        })
        .collect())
}

async fn load_active_jobs(state: &AppState) -> anyhow::Result<Vec<ActiveJobRow>> {
    let rows = sqlx::query(
        "SELECT id, name, agent_id FROM jobs WHERE archived_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ActiveJobRow {
            job_id: row.get::<String, _>("id"),
            agent_id: row.get::<Option<String>, _>("agent_id"),
        })
        .collect())
}

async fn load_backup_job_signals(state: &AppState) -> anyhow::Result<Vec<JobSignalRow>> {
    let rows = sqlx::query(
        r#"
        SELECT j.id AS job_id, MAX(r.ended_at) AS latest_success_at
        FROM jobs j
        LEFT JOIN runs r
          ON r.job_id = j.id
         AND r.status = 'success'
         AND r.ended_at IS NOT NULL
        WHERE j.archived_at IS NULL
        GROUP BY j.id
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| JobSignalRow {
            job_id: row.get::<String, _>("job_id"),
            latest_success_at: row.get::<Option<i64>, _>("latest_success_at"),
        })
        .collect())
}

async fn load_verify_job_signals(state: &AppState) -> anyhow::Result<Vec<JobSignalRow>> {
    let rows = sqlx::query(
        r#"
        SELECT j.id AS job_id, MAX(o.ended_at) AS latest_success_at
        FROM jobs j
        LEFT JOIN runs r ON r.job_id = j.id
        LEFT JOIN operations o
          ON o.subject_kind = 'run'
         AND o.subject_id = r.id
         AND o.kind = 'verify'
         AND o.status = 'success'
         AND o.ended_at IS NOT NULL
        WHERE j.archived_at IS NULL
        GROUP BY j.id
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| JobSignalRow {
            job_id: row.get::<String, _>("job_id"),
            latest_success_at: row.get::<Option<i64>, _>("latest_success_at"),
        })
        .collect())
}

async fn load_latest_successful_backup(
    state: &AppState,
) -> anyhow::Result<Vec<LatestBackupRow>> {
    let rows = sqlx::query(
        r#"
        SELECT
          r.id AS run_id,
          j.id AS job_id,
          j.name AS job_name,
          j.agent_id AS agent_id,
          r.ended_at AS ended_at
        FROM runs r
        JOIN jobs j ON j.id = r.job_id
        WHERE j.archived_at IS NULL
          AND r.status = 'success'
          AND r.ended_at IS NOT NULL
        ORDER BY r.ended_at DESC
        LIMIT 40
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| LatestBackupRow {
            run_id: row.get::<String, _>("run_id"),
            job_id: row.get::<String, _>("job_id"),
            job_name: row.get::<String, _>("job_name"),
            agent_id: row.get::<Option<String>, _>("agent_id"),
            ended_at: row.get::<i64, _>("ended_at"),
        })
        .collect())
}

async fn load_latest_successful_verify(
    state: &AppState,
) -> anyhow::Result<Vec<LatestVerifyRow>> {
    let rows = sqlx::query(
        r#"
        SELECT
          o.id AS operation_id,
          r.id AS run_id,
          j.id AS job_id,
          j.name AS job_name,
          j.agent_id AS agent_id,
          o.ended_at AS ended_at
        FROM operations o
        JOIN runs r ON o.subject_kind = 'run' AND o.subject_id = r.id
        JOIN jobs j ON j.id = r.job_id
        WHERE j.archived_at IS NULL
          AND o.kind = 'verify'
          AND o.status = 'success'
          AND o.ended_at IS NOT NULL
        ORDER BY o.ended_at DESC
        LIMIT 40
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| LatestVerifyRow {
            operation_id: row.get::<String, _>("operation_id"),
            run_id: row.get::<String, _>("run_id"),
            job_id: row.get::<String, _>("job_id"),
            job_name: row.get::<String, _>("job_name"),
            agent_id: row.get::<Option<String>, _>("agent_id"),
            ended_at: row.get::<i64, _>("ended_at"),
        })
        .collect())
}

fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Critical => 0,
        Severity::Warning => 1,
        Severity::Info => 2,
    }
}

fn activity_rank(kind: &str) -> u8 {
    if kind.contains("failed") {
        return 0;
    }
    if kind.contains("rejected") || kind.contains("canceled") {
        return 1;
    }
    if kind.contains("running") {
        return 2;
    }
    if kind.contains("verify") || kind.contains("restore") {
        return 3;
    }
    4
}

fn job_scope_string(agent_id: Option<&str>) -> String {
    match agent_id {
        Some(agent_id) => format!("agent:{agent_id}"),
        None => "hub".to_string(),
    }
}

fn node_id_for_agent(agent_id: Option<&str>) -> String {
    agent_id.unwrap_or("hub").to_string()
}

fn node_name_for_agent(agent_id: Option<&str>, agent_name: Option<&str>) -> Option<String> {
    match (agent_id, agent_name) {
        (None, _) => Some("Hub".to_string()),
        (Some(_agent_id), Some(agent_name)) if !agent_name.trim().is_empty() => {
            Some(agent_name.to_string())
        }
        (Some(agent_id), _) => Some(agent_id.to_string()),
    }
}

fn scope_label_from_agent(agent_id: Option<&str>) -> &'static str {
    if agent_id.is_some() {
        "Agent"
    } else {
        "Hub"
    }
}

fn run_detail_href(run_id: &str, scope: &str) -> String {
    format!("/runs/{run_id}?{}", query_string(&[("from_scope", scope)]))
}

fn jobs_scope_href(scope: &str) -> String {
    format!("/jobs?{}", query_string(&[("scope", scope)]))
}

fn runs_index_href(scope: &str) -> String {
    format!("/runs?{}", query_string(&[("scope", scope)]))
}

fn fleet_href(status: &str) -> String {
    format!("/fleet?{}", query_string(&[("status", status)]))
}

fn notifications_queue_href(status: &str) -> String {
    format!(
        "/integrations/notifications/queue?{}",
        query_string(&[("status", status)])
    )
}

fn query_string(params: &[(&str, &str)]) -> String {
    let mut serializer = Serializer::new(String::new());
    for (key, value) in params {
        serializer.append_pair(key, value);
    }
    serializer.finish()
}

fn activity_suffix_for_run(status: &str) -> &'static str {
    match status {
        "failed" => "failed",
        "rejected" => "was rejected",
        "running" => "is running",
        "queued" => "is queued",
        "success" => "completed successfully",
        "canceled" => "was canceled",
        _ => "changed state",
    }
}

fn activity_suffix_for_operation(kind: &str, status: &str) -> String {
    match (kind, status) {
        ("verify", "success") => "verification completed".to_string(),
        ("verify", "failed") => "verification failed".to_string(),
        ("verify", "running") => "verification is running".to_string(),
        ("restore", "success") => "restore completed".to_string(),
        ("restore", "failed") => "restore failed".to_string(),
        ("restore", "running") => "restore is running".to_string(),
        _ => format!("{kind} {status}"),
    }
}
