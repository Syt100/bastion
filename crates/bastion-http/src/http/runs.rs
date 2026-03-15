use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Row};
use tower_cookies::Cookies;
use time::OffsetDateTime;

use bastion_backup::restore;
use bastion_core::agent_protocol::{HubToAgentMessageV1, PROTOCOL_VERSION};
use bastion_engine::cancel_registry::global_cancel_registry;
use bastion_engine::run_events;
use bastion_storage::agent_tasks_repo;
use bastion_storage::runs_repo;

use super::shared::{require_csrf, require_session};
use super::{AppError, AppState};

fn invalid_kind_error(message: impl Into<String>) -> AppError {
    AppError::bad_request("invalid_kind", message)
        .with_reason("unsupported_value")
        .with_field("kind")
}

fn invalid_page_size_error(reason: &'static str, message: impl Into<String>) -> AppError {
    AppError::bad_request("invalid_page_size", message)
        .with_reason(reason)
        .with_field("page_size")
}

fn invalid_page_error(reason: &'static str, message: impl Into<String>) -> AppError {
    AppError::bad_request("invalid_page", message)
        .with_reason(reason)
        .with_field("page")
}

fn invalid_event_filter_error(field: &'static str, reason: &'static str) -> AppError {
    AppError::bad_request("invalid_filter", format!("invalid {field} filter"))
        .with_reason(reason)
        .with_field(field)
}

fn invalid_anchor_error() -> AppError {
    AppError::bad_request("invalid_anchor", "invalid anchor")
        .with_reason("unsupported_value")
        .with_field("anchor")
}

fn normalize_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
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

        Err(AppError::bad_request("invalid_scope", "invalid scope")
            .with_reason("unsupported_value")
            .with_field("scope"))
    }

    fn as_str(&self) -> String {
        match self {
            Self::All => "all".to_string(),
            Self::Hub => "hub".to_string(),
            Self::Agent(agent_id) => format!("agent:{agent_id}"),
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
        let raw = raw
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("24h");
        let (preset, seconds) = match raw {
            "24h" => ("24h", 24 * 60 * 60),
            "7d" => ("7d", 7 * 24 * 60 * 60),
            "30d" => ("30d", 30 * 24 * 60 * 60),
            _ => {
                return Err(AppError::bad_request("invalid_range", "invalid range")
                    .with_reason("unsupported_value")
                    .with_field("range"));
            }
        };

        Ok(Self {
            preset,
            from: now_ts.saturating_sub(seconds),
            to: now_ts,
        })
    }
}

fn scope_string(agent_id: Option<&str>) -> String {
    match agent_id {
        Some(agent_id) => format!("agent:{agent_id}"),
        None => "hub".to_string(),
    }
}

fn parse_page(page: Option<i64>) -> Result<i64, AppError> {
    match page {
        None => Ok(1),
        Some(value) if value > 0 => Ok(value),
        Some(_) => Err(invalid_page_error("invalid_value", "page must be positive")),
    }
}

fn parse_page_size(page_size: Option<i64>) -> Result<i64, AppError> {
    match page_size {
        None => Ok(20),
        Some(value) if value <= 0 => Err(invalid_page_size_error(
            "invalid_value",
            "page_size must be positive",
        )),
        Some(value) if value > 100 => Err(invalid_page_size_error(
            "too_large",
            "page_size must not exceed 100",
        )),
        Some(value) => Ok(value),
    }
}

fn parse_kind_filter(raw: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(raw) = normalize_optional_string(raw) else {
        return Ok(None);
    };
    if matches!(raw.as_str(), "backup" | "restore" | "verify" | "cleanup") {
        return Ok(Some(raw));
    }
    Err(invalid_kind_error("invalid kind"))
}

fn derive_run_kind(progress: Option<&serde_json::Value>) -> String {
    let kind = progress
        .and_then(|value| value.get("kind"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| matches!(*value, "backup" | "restore" | "verify" | "cleanup"));
    kind.unwrap_or("backup").to_string()
}

#[derive(Debug, Serialize)]
pub(super) struct RunsWorkspaceScopeEcho {
    requested: String,
    effective: String,
}

#[derive(Debug, Serialize)]
pub(super) struct RunsWorkspaceFilters {
    q: String,
    status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    job_id: Option<String>,
    kind: String,
    range: String,
}

#[derive(Debug, Serialize)]
pub(super) struct RunWorkspaceListItem {
    id: String,
    job_id: String,
    job_name: String,
    scope: String,
    node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    node_name: Option<String>,
    status: runs_repo::RunStatus,
    kind: String,
    started_at: i64,
    ended_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    failure_title: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct RunsWorkspaceListResponse {
    scope: RunsWorkspaceScopeEcho,
    filters: RunsWorkspaceFilters,
    items: Vec<RunWorkspaceListItem>,
    page: i64,
    page_size: i64,
    total: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct RunWorkspaceRun {
    id: String,
    job_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    job_name: Option<String>,
    scope: String,
    node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    node_name: Option<String>,
    status: runs_repo::RunStatus,
    kind: String,
    started_at: i64,
    ended_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cancel_requested_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cancel_requested_by_user_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cancel_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct RunWorkspaceDiagnostics {
    state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    failure_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    failure_stage: Option<String>,
    failure_title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    failure_hint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    first_error_event_seq: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    root_cause_event_seq: Option<i64>,
}

#[derive(Debug, Serialize)]
pub(super) struct RunWorkspaceCapabilities {
    can_cancel: bool,
    can_restore: bool,
    can_verify: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct RunWorkspaceRelatedSummary {
    operations_total: i64,
    artifacts_total: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct RunWorkspaceResponse {
    run: RunWorkspaceRun,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    progress: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    summary: Option<serde_json::Value>,
    diagnostics: RunWorkspaceDiagnostics,
    capabilities: RunWorkspaceCapabilities,
    related: RunWorkspaceRelatedSummary,
}

#[derive(Debug, Serialize)]
pub(super) struct RunEventConsoleFiltersEcho {
    q: String,
    levels: Vec<String>,
    kinds: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct RunEventConsoleWindow {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    first_seq: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_seq: Option<i64>,
    has_older: bool,
    has_newer: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct RunEventConsoleLocators {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    first_error_seq: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    root_cause_seq: Option<i64>,
}

#[derive(Debug, Serialize)]
pub(super) struct RunEventConsoleResponse {
    filters: RunEventConsoleFiltersEcho,
    window: RunEventConsoleWindow,
    locators: RunEventConsoleLocators,
    items: Vec<runs_repo::RunEvent>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ListRunsWorkspaceQuery {
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    job_id: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    range: Option<String>,
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    page: Option<i64>,
    #[serde(default)]
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(super) struct RunEventConsoleQuery {
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    levels: Option<String>,
    #[serde(default)]
    kinds: Option<String>,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    before_seq: Option<i64>,
    #[serde(default)]
    after_seq: Option<i64>,
    #[serde(default)]
    anchor: Option<String>,
}

#[derive(Debug, Clone)]
enum RunEventAnchor {
    Tail,
    FirstError,
    Seq(i64),
}

impl RunEventAnchor {
    fn parse(raw: Option<&str>) -> Result<Self, AppError> {
        let Some(raw) = normalize_optional_string(raw) else {
            return Ok(Self::Tail);
        };
        if raw == "tail" {
            return Ok(Self::Tail);
        }
        if raw == "first_error" {
            return Ok(Self::FirstError);
        }
        if let Some(value) = raw.strip_prefix("seq:") {
            let parsed = value
                .parse::<i64>()
                .ok()
                .filter(|v| *v > 0)
                .ok_or_else(invalid_anchor_error)?;
            return Ok(Self::Seq(parsed));
        }
        Err(invalid_anchor_error())
    }
}

fn parse_csv_filter(raw: Option<&str>, field: &'static str) -> Result<Vec<String>, AppError> {
    let Some(raw) = normalize_optional_string(raw) else {
        return Ok(Vec::new());
    };
    let mut values = Vec::new();
    for part in raw.split(',') {
        let value = part.trim().to_lowercase();
        if value.is_empty() {
            continue;
        }
        values.push(value);
    }
    if values.is_empty() {
        return Ok(Vec::new());
    }
    if values.iter().any(|value| value.is_empty()) {
        return Err(invalid_event_filter_error(field, "unsupported_value"));
    }
    Ok(values)
}

fn apply_runs_list_filters<'a>(
    qb: &mut QueryBuilder<'a, sqlx::Sqlite>,
    scope: &'a RequestedScope,
    range: ResolvedRange,
    status: Option<&'a str>,
    job_id: Option<&'a str>,
    kind: Option<&'a str>,
    q: Option<&'a str>,
) {
    qb.push(" WHERE r.started_at >= ");
    qb.push_bind(range.from);
    qb.push(" AND r.started_at <= ");
    qb.push_bind(range.to);

    match scope {
        RequestedScope::All => {}
        RequestedScope::Hub => {
            qb.push(" AND j.agent_id IS NULL");
        }
        RequestedScope::Agent(agent_id) => {
            qb.push(" AND j.agent_id = ");
            qb.push_bind(agent_id);
        }
    }

    if let Some(status) = status {
        qb.push(" AND r.status = ");
        qb.push_bind(status);
    }
    if let Some(job_id) = job_id {
        qb.push(" AND r.job_id = ");
        qb.push_bind(job_id);
    }
    if let Some(kind) = kind {
        qb.push(" AND COALESCE(NULLIF(json_extract(r.progress_json, '$.kind'), ''), 'backup') = ");
        qb.push_bind(kind);
    }
    if let Some(q) = q {
        let pattern = format!("%{}%", q.to_lowercase());
        qb.push(" AND (LOWER(r.id) LIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR LOWER(COALESCE(j.name, '')) LIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR LOWER(COALESCE(r.error, '')) LIKE ");
        qb.push_bind(pattern);
        qb.push(")");
    }
}

fn apply_run_event_filters<'a>(
    qb: &mut QueryBuilder<'a, sqlx::Sqlite>,
    run_id: &'a str,
    q: Option<&'a str>,
    levels: &'a [String],
    kinds: &'a [String],
) {
    qb.push(" WHERE run_id = ");
    qb.push_bind(run_id);

    if let Some(q) = q {
        let pattern = format!("%{}%", q.to_lowercase());
        qb.push(" AND (LOWER(message) LIKE ");
        qb.push_bind(pattern.clone());
        qb.push(" OR LOWER(COALESCE(fields_json, '')) LIKE ");
        qb.push_bind(pattern);
        qb.push(")");
    }

    if !levels.is_empty() {
        qb.push(" AND LOWER(level) IN (");
        let mut separated = qb.separated(", ");
        for level in levels {
            separated.push_bind(level);
        }
        separated.push_unseparated(")");
    }

    if !kinds.is_empty() {
        qb.push(" AND LOWER(kind) IN (");
        let mut separated = qb.separated(", ");
        for kind in kinds {
            separated.push_bind(kind);
        }
        separated.push_unseparated(")");
    }
}

async fn load_first_error_event(
    db: &sqlx::SqlitePool,
    run_id: &str,
) -> Result<Option<runs_repo::RunEvent>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT run_id, seq, ts, level, kind, message, fields_json
        FROM run_events
        WHERE run_id = ?
          AND (LOWER(level) = 'error' OR kind IN ('failed', 'rejected', 'canceled'))
        ORDER BY seq ASC
        LIMIT 1
        "#,
    )
    .bind(run_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let fields = row
        .get::<Option<String>, _>("fields_json")
        .map(|value| serde_json::from_str::<serde_json::Value>(&value))
        .transpose()?;

    Ok(Some(runs_repo::RunEvent {
        run_id: row.get::<String, _>("run_id"),
        seq: row.get::<i64, _>("seq"),
        ts: row.get::<i64, _>("ts"),
        level: row.get::<String, _>("level"),
        kind: row.get::<String, _>("kind"),
        message: row.get::<String, _>("message"),
        fields,
    }))
}

fn diagnostics_from_run(
    run: &RunResponse,
    first_error_event: Option<&runs_repo::RunEvent>,
) -> RunWorkspaceDiagnostics {
    let progress_stage = run
        .progress
        .as_ref()
        .and_then(|value| value.get("stage"))
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let event_fields = first_error_event.and_then(|event| event.fields.as_ref());
    let event_failure_kind = event_fields
        .and_then(|fields| fields.get("error_envelope"))
        .and_then(|value| value.get("kind"))
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let event_failure_hint = event_fields
        .and_then(|fields| fields.get("hint"))
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let event_stage = event_fields
        .and_then(|fields| fields.get("stage"))
        .and_then(|value| value.as_str())
        .map(str::to_string);

    let failure_title = if let Some(event) = first_error_event {
        event.message.clone()
    } else if let Some(error) = run.error.clone() {
        error
    } else if run.status == runs_repo::RunStatus::Success {
        "Run completed successfully".to_string()
    } else if run.status == runs_repo::RunStatus::Canceled {
        "Run was canceled".to_string()
    } else {
        "Run diagnostics unavailable".to_string()
    };

    let first_error_seq = first_error_event.map(|event| event.seq);
    let failure_kind = event_failure_kind.or_else(|| run.error.clone());
    let failure_stage = progress_stage.or(event_stage);
    let state = if first_error_seq.is_some() || failure_stage.is_some() || failure_kind.is_some() {
        "structured"
    } else {
        "fallback"
    };

    RunWorkspaceDiagnostics {
        state: state.to_string(),
        failure_kind,
        failure_stage,
        failure_title,
        failure_hint: event_failure_hint,
        first_error_event_seq: first_error_seq,
        root_cause_event_seq: first_error_seq,
    }
}

#[derive(Debug, Serialize)]
pub(super) struct RunResponse {
    id: String,
    job_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    job_name: Option<String>,
    node_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    node_name: Option<String>,
    status: runs_repo::RunStatus,
    started_at: i64,
    ended_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cancel_requested_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cancel_requested_by_user_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cancel_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    progress: Option<serde_json::Value>,
    summary: Option<serde_json::Value>,
    error: Option<String>,
}

async fn get_run_response(
    db: &sqlx::SqlitePool,
    run_id: &str,
) -> Result<Option<RunResponse>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT
          r.id AS id,
          r.job_id AS job_id,
          r.status AS status,
          r.started_at AS started_at,
          r.ended_at AS ended_at,
          r.cancel_requested_at AS cancel_requested_at,
          r.cancel_requested_by_user_id AS cancel_requested_by_user_id,
          r.cancel_reason AS cancel_reason,
          r.progress_json AS progress_json,
          r.summary_json AS summary_json,
          r.error AS error,
          j.name AS job_name,
          COALESCE(j.agent_id, 'hub') AS node_id,
          a.name AS node_name
        FROM runs r
        JOIN jobs j ON j.id = r.job_id
        LEFT JOIN agents a ON a.id = j.agent_id
        WHERE r.id = ?
        LIMIT 1
        "#,
    )
    .bind(run_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let status = row
        .get::<String, _>("status")
        .parse::<runs_repo::RunStatus>()?;
    let progress_json = row.get::<Option<String>, _>("progress_json");
    let progress = progress_json
        .map(|value| serde_json::from_str::<serde_json::Value>(&value))
        .transpose()?;
    let summary_json = row.get::<Option<String>, _>("summary_json");
    let summary = summary_json
        .map(|value| serde_json::from_str::<serde_json::Value>(&value))
        .transpose()?;

    Ok(Some(RunResponse {
        id: row.get::<String, _>("id"),
        job_id: row.get::<String, _>("job_id"),
        job_name: row.get::<Option<String>, _>("job_name"),
        node_id: row.get::<String, _>("node_id"),
        node_name: row.get::<Option<String>, _>("node_name"),
        status,
        started_at: row.get::<i64, _>("started_at"),
        ended_at: row.get::<Option<i64>, _>("ended_at"),
        cancel_requested_at: row.get::<Option<i64>, _>("cancel_requested_at"),
        cancel_requested_by_user_id: row.get::<Option<i64>, _>("cancel_requested_by_user_id"),
        cancel_reason: row.get::<Option<String>, _>("cancel_reason"),
        progress,
        summary,
        error: row.get::<Option<String>, _>("error"),
    }))
}

pub(super) async fn get_run(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(run_id): Path<String>,
) -> Result<Json<RunResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let run = get_run_response(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;

    Ok(Json(run))
}

pub(super) async fn list_runs_workspace(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Query(query): Query<ListRunsWorkspaceQuery>,
) -> Result<Json<RunsWorkspaceListResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let requested_scope = RequestedScope::parse(query.scope.as_deref())?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let range = ResolvedRange::parse(query.range.as_deref(), now)?;
    let page = parse_page(query.page)?;
    let page_size = parse_page_size(query.page_size)?;
    let q = normalize_optional_string(query.q.as_deref());
    let job_id = normalize_optional_string(query.job_id.as_deref());
    let kind = parse_kind_filter(query.kind.as_deref())?;
    let status = normalize_optional_string(query.status.as_deref());
    if let Some(ref status) = status {
        status.parse::<runs_repo::RunStatus>().map_err(|_| {
            AppError::bad_request("invalid_status", "invalid status")
                .with_reason("unsupported_value")
                .with_field("status")
        })?;
    }

    let mut count_qb: QueryBuilder<'_, sqlx::Sqlite> = QueryBuilder::new(
        r#"
        SELECT COUNT(*) AS total
        FROM runs r
        JOIN jobs j ON j.id = r.job_id
        "#,
    );
    apply_runs_list_filters(
        &mut count_qb,
        &requested_scope,
        range,
        status.as_deref(),
        job_id.as_deref(),
        kind.as_deref(),
        q.as_deref(),
    );
    let total = count_qb
        .build_query_scalar::<i64>()
        .fetch_one(&state.db)
        .await?;

    let offset = (page - 1) * page_size;
    let mut qb: QueryBuilder<'_, sqlx::Sqlite> = QueryBuilder::new(
        r#"
        SELECT
          r.id AS id,
          r.job_id AS job_id,
          j.name AS job_name,
          j.agent_id AS agent_id,
          a.name AS node_name,
          r.status AS status,
          r.started_at AS started_at,
          r.ended_at AS ended_at,
          r.error AS error,
          r.progress_json AS progress_json
        FROM runs r
        JOIN jobs j ON j.id = r.job_id
        LEFT JOIN agents a ON a.id = j.agent_id
        "#,
    );
    apply_runs_list_filters(
        &mut qb,
        &requested_scope,
        range,
        status.as_deref(),
        job_id.as_deref(),
        kind.as_deref(),
        q.as_deref(),
    );
    qb.push(" ORDER BY r.started_at DESC, r.id DESC LIMIT ");
    qb.push_bind(page_size);
    qb.push(" OFFSET ");
    qb.push_bind(offset);

    let rows = qb.build().fetch_all(&state.db).await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let status = row
            .get::<String, _>("status")
            .parse::<runs_repo::RunStatus>()?;
        let progress = row
            .get::<Option<String>, _>("progress_json")
            .map(|value| serde_json::from_str::<serde_json::Value>(&value))
            .transpose()?;
        let kind = derive_run_kind(progress.as_ref());
        let failure_title = row.get::<Option<String>, _>("error");
        let agent_id = row.get::<Option<String>, _>("agent_id");
        let node_id = agent_id.clone().unwrap_or_else(|| "hub".to_string());
        items.push(RunWorkspaceListItem {
            id: row.get::<String, _>("id"),
            job_id: row.get::<String, _>("job_id"),
            job_name: row.get::<String, _>("job_name"),
            scope: scope_string(agent_id.as_deref()),
            node_id,
            node_name: row.get::<Option<String>, _>("node_name"),
            status,
            kind,
            started_at: row.get::<i64, _>("started_at"),
            ended_at: row.get::<Option<i64>, _>("ended_at"),
            error: row.get::<Option<String>, _>("error"),
            failure_title,
        });
    }

    Ok(Json(RunsWorkspaceListResponse {
        scope: RunsWorkspaceScopeEcho {
            requested: requested_scope.as_str(),
            effective: requested_scope.as_str(),
        },
        filters: RunsWorkspaceFilters {
            q: q.unwrap_or_default(),
            status: status.unwrap_or_else(|| "all".to_string()),
            job_id,
            kind: kind.unwrap_or_else(|| "all".to_string()),
            range: range.preset.to_string(),
        },
        items,
        page,
        page_size,
        total,
    }))
}

pub(super) async fn get_run_workspace(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(run_id): Path<String>,
) -> Result<Json<RunWorkspaceResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let run = get_run_response(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;

    let first_error_event = load_first_error_event(&state.db, &run_id).await?;
    let diagnostics = diagnostics_from_run(&run, first_error_event.as_ref());

    let operations_total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM operations WHERE subject_kind = 'run' AND subject_id = ?",
    )
    .bind(&run_id)
    .fetch_one(&state.db)
    .await?;

    let artifacts_total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM run_artifacts WHERE run_id = ?",
    )
    .bind(&run_id)
    .fetch_one(&state.db)
    .await?;

    let capabilities = RunWorkspaceCapabilities {
        can_cancel: run.status.is_cancelable() && run.cancel_requested_at.is_none(),
        can_restore: run.status == runs_repo::RunStatus::Success,
        can_verify: run.status == runs_repo::RunStatus::Success,
    };
    let kind = derive_run_kind(run.progress.as_ref());

    Ok(Json(RunWorkspaceResponse {
        run: RunWorkspaceRun {
            id: run.id.clone(),
            job_id: run.job_id.clone(),
            job_name: run.job_name.clone(),
            scope: scope_string((run.node_id != "hub").then_some(run.node_id.as_str())),
            node_id: run.node_id.clone(),
            node_name: run.node_name.clone(),
            status: run.status,
            kind,
            started_at: run.started_at,
            ended_at: run.ended_at,
            cancel_requested_at: run.cancel_requested_at,
            cancel_requested_by_user_id: run.cancel_requested_by_user_id,
            cancel_reason: run.cancel_reason.clone(),
            error: run.error.clone(),
        },
        progress: run.progress.clone(),
        summary: run.summary.clone(),
        diagnostics,
        capabilities,
        related: RunWorkspaceRelatedSummary {
            operations_total,
            artifacts_total,
        },
    }))
}

pub(super) async fn list_run_event_console(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(run_id): Path<String>,
    Query(query): Query<RunEventConsoleQuery>,
) -> Result<Json<RunEventConsoleResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    if runs_repo::get_run(&state.db, &run_id).await?.is_none() {
        return Err(AppError::not_found("run_not_found", "Run not found"));
    }

    if query.before_seq.is_some() && query.after_seq.is_some() {
        return Err(AppError::bad_request(
            "invalid_window",
            "before_seq and after_seq cannot be combined",
        ));
    }

    let q = normalize_optional_string(query.q.as_deref());
    let levels = parse_csv_filter(query.levels.as_deref(), "levels")?;
    let kinds = parse_csv_filter(query.kinds.as_deref(), "kinds")?;
    let limit = match query.limit {
        None => 100,
        Some(value) if value <= 0 => {
            return Err(AppError::bad_request("invalid_limit", "limit must be positive")
                .with_reason("invalid_value")
                .with_field("limit"));
        }
        Some(value) if value > 200 => {
            return Err(AppError::bad_request("invalid_limit", "limit must not exceed 200")
                .with_reason("too_large")
                .with_field("limit"));
        }
        Some(value) => value,
    };
    let anchor = RunEventAnchor::parse(query.anchor.as_deref())?;

    let first_error_event = load_first_error_event(&state.db, &run_id).await?;
    let first_error_seq = first_error_event.as_ref().map(|event| event.seq);
    let anchor_seq = match anchor {
        RunEventAnchor::Tail => None,
        RunEventAnchor::FirstError => first_error_seq,
        RunEventAnchor::Seq(seq) => Some(seq),
    };

    let descending = query.before_seq.is_some() || (query.after_seq.is_none() && anchor_seq.is_none());
    let mut qb: QueryBuilder<'_, sqlx::Sqlite> = QueryBuilder::new(
        "SELECT run_id, seq, ts, level, kind, message, fields_json FROM run_events",
    );
    apply_run_event_filters(&mut qb, &run_id, q.as_deref(), &levels, &kinds);

    if let Some(before_seq) = query.before_seq {
        qb.push(" AND seq < ");
        qb.push_bind(before_seq);
    }
    if let Some(after_seq) = query.after_seq {
        qb.push(" AND seq > ");
        qb.push_bind(after_seq);
    }
    if let Some(anchor_seq) = anchor_seq
        && query.before_seq.is_none()
        && query.after_seq.is_none()
    {
        qb.push(" AND seq >= ");
        qb.push_bind(anchor_seq);
    }

    if descending {
        qb.push(" ORDER BY seq DESC");
    } else {
        qb.push(" ORDER BY seq ASC");
    }
    qb.push(" LIMIT ");
    qb.push_bind(limit);

    let rows = qb.build().fetch_all(&state.db).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let fields = row
            .get::<Option<String>, _>("fields_json")
            .map(|value| serde_json::from_str::<serde_json::Value>(&value))
            .transpose()?;
        items.push(runs_repo::RunEvent {
            run_id: row.get::<String, _>("run_id"),
            seq: row.get::<i64, _>("seq"),
            ts: row.get::<i64, _>("ts"),
            level: row.get::<String, _>("level"),
            kind: row.get::<String, _>("kind"),
            message: row.get::<String, _>("message"),
            fields,
        });
    }
    if descending {
        items.reverse();
    }

    let first_seq = items.first().map(|item| item.seq);
    let last_seq = items.last().map(|item| item.seq);

    let has_older = if let Some(first_seq) = first_seq {
        let mut qb: QueryBuilder<'_, sqlx::Sqlite> =
            QueryBuilder::new("SELECT 1 FROM run_events");
        apply_run_event_filters(&mut qb, &run_id, q.as_deref(), &levels, &kinds);
        qb.push(" AND seq < ");
        qb.push_bind(first_seq);
        qb.push(" LIMIT 1");
        qb.build().fetch_optional(&state.db).await?.is_some()
    } else {
        false
    };

    let has_newer = if let Some(last_seq) = last_seq {
        let mut qb: QueryBuilder<'_, sqlx::Sqlite> =
            QueryBuilder::new("SELECT 1 FROM run_events");
        apply_run_event_filters(&mut qb, &run_id, q.as_deref(), &levels, &kinds);
        qb.push(" AND seq > ");
        qb.push_bind(last_seq);
        qb.push(" LIMIT 1");
        qb.build().fetch_optional(&state.db).await?.is_some()
    } else {
        false
    };

    Ok(Json(RunEventConsoleResponse {
        filters: RunEventConsoleFiltersEcho {
            q: q.unwrap_or_default(),
            levels,
            kinds,
        },
        window: RunEventConsoleWindow {
            first_seq,
            last_seq,
            has_older,
            has_newer,
        },
        locators: RunEventConsoleLocators {
            first_error_seq,
            root_cause_seq: first_error_seq,
        },
        items,
    }))
}

#[derive(Debug, Deserialize)]
pub(super) struct CancelRunRequest {
    #[serde(default)]
    reason: Option<String>,
}

pub(super) async fn cancel_run(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(run_id): Path<String>,
    Json(req): Json<CancelRunRequest>,
) -> Result<Json<RunResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let before = runs_repo::get_run(&state.db, &run_id)
        .await?
        .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;

    let run =
        runs_repo::request_run_cancel(&state.db, &run_id, session.user_id, req.reason.as_deref())
            .await?
            .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;

    if before.status == runs_repo::RunStatus::Running {
        let _ = run_events::append_and_broadcast(
            &state.db,
            &state.run_events_bus,
            &run.id,
            "info",
            "cancel_requested",
            "cancel requested",
            None,
        )
        .await;
        let _ = global_cancel_registry().cancel_run(&run.id);
        if let Ok(Some(task)) = agent_tasks_repo::get_task(&state.db, &run.id).await
            && task.completed_at.is_none()
        {
            let _ = state
                .agent_manager
                .send_json(
                    &task.agent_id,
                    &HubToAgentMessageV1::CancelRunTask {
                        v: PROTOCOL_VERSION,
                        run_id: run.id.clone(),
                    },
                )
                .await;
        }
    } else if before.status == runs_repo::RunStatus::Queued
        && run.status == runs_repo::RunStatus::Canceled
    {
        let _ = run_events::append_and_broadcast(
            &state.db,
            &state.run_events_bus,
            &run.id,
            "info",
            "canceled",
            "canceled",
            None,
        )
        .await;
    }

    let response = get_run_response(&state.db, &run.id)
        .await?
        .ok_or_else(|| AppError::not_found("run_not_found", "Run not found"))?;

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub(super) struct ListRunEntriesQuery {
    #[serde(default)]
    prefix: Option<String>,
    #[serde(default)]
    cursor: Option<u64>,
    #[serde(default)]
    limit: Option<u64>,
    #[serde(default)]
    q: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    hide_dotfiles: Option<bool>,
    #[serde(default)]
    min_size_bytes: Option<u64>,
    #[serde(default)]
    max_size_bytes: Option<u64>,
    #[serde(default)]
    type_sort: Option<String>,
}

pub(super) async fn list_run_entries(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(run_id): Path<String>,
    Query(query): Query<ListRunEntriesQuery>,
) -> Result<Json<restore::RunEntriesChildrenResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let ListRunEntriesQuery {
        prefix,
        cursor,
        limit,
        q,
        kind,
        hide_dotfiles,
        min_size_bytes,
        max_size_bytes,
        type_sort,
    } = query;

    let cursor = cursor.unwrap_or(0);
    let limit = limit.unwrap_or(200);
    let kind = match kind.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        None => None,
        Some(v) if matches!(v, "file" | "dir" | "symlink") => Some(v.to_string()),
        Some(_) => return Err(invalid_kind_error("invalid kind")),
    };
    let hide_dotfiles = hide_dotfiles.unwrap_or(false);
    let (min_size_bytes, max_size_bytes) = match (min_size_bytes, max_size_bytes) {
        (Some(a), Some(b)) if a > b => (Some(b), Some(a)),
        other => other,
    };
    let type_sort = type_sort
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let type_sort_file_first = match type_sort {
        None | Some("dir_first") => false,
        Some("file_first") => true,
        Some(_) => {
            return Err(
                AppError::bad_request("invalid_type_sort", "invalid type_sort")
                    .with_details(serde_json::json!({ "field": "type_sort" })),
            );
        }
    };

    let result = restore::list_run_entries_children_with_options(
        &state.db,
        state.secrets.as_ref(),
        &state.config.data_dir,
        &run_id,
        restore::ListRunEntriesChildrenOptions {
            prefix,
            cursor,
            limit,
            q,
            kind,
            hide_dotfiles,
            min_size_bytes,
            max_size_bytes,
            type_sort_file_first,
        },
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
