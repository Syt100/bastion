use axum::Json;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tower_cookies::Cookies;

use bastion_core::HUB_NODE_ID;
use bastion_storage::{bulk_operations_repo, jobs_repo, secrets_repo};

use super::agents::{LabelsMode, normalize_labels, parse_labels_mode};
use super::shared::{require_csrf, require_session};
use super::{AppError, AppState};

const DEFAULT_LIST_LIMIT: i64 = 50;

fn normalize_node_ids(values: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = values
        .into_iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    out.sort();
    out.dedup();
    out
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct BulkSelectorRequest {
    node_ids: Option<Vec<String>>,
    labels: Option<Vec<String>>,
    labels_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct AgentLabelsPayloadRequest {
    labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct WebdavDistributePayloadRequest {
    name: String,
    overwrite: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct JobDeployPayloadRequest {
    source_job_id: String,
    name_template: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(in crate::http) struct CreateBulkOperationRequest {
    kind: String,
    selector: BulkSelectorRequest,
    payload: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct CreateBulkOperationResponse {
    op_id: String,
}

fn validate_kind(kind: &str) -> Result<&str, AppError> {
    match kind {
        "agent_labels_add"
        | "agent_labels_remove"
        | "sync_config_now"
        | "job_deploy"
        | "webdav_secret_distribute" => Ok(kind),
        _ => Err(AppError::bad_request("invalid_kind", "Invalid kind")),
    }
}

async fn validate_agent_ids_exist(
    db: &sqlx::SqlitePool,
    agent_ids: &[String],
) -> Result<(), AppError> {
    if agent_ids.is_empty() {
        return Err(AppError::bad_request(
            "invalid_selector",
            "Selector is required",
        ));
    }

    let mut qb: sqlx::QueryBuilder<sqlx::Sqlite> =
        sqlx::QueryBuilder::new("SELECT id FROM agents WHERE id IN (");
    let mut separated = qb.separated(", ");
    for id in agent_ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");

    let rows = qb.build().fetch_all(db).await?;
    if rows.len() != agent_ids.len() {
        return Err(AppError::bad_request(
            "invalid_selector",
            "One or more agents were not found",
        ));
    }
    Ok(())
}

async fn resolve_selector(
    db: &sqlx::SqlitePool,
    selector: BulkSelectorRequest,
) -> Result<(Vec<String>, serde_json::Value), AppError> {
    let node_ids = selector.node_ids.unwrap_or_default();
    let selector_labels = selector.labels.unwrap_or_default();

    if !node_ids.is_empty() {
        let ids = normalize_node_ids(node_ids);
        validate_agent_ids_exist(db, &ids).await?;
        return Ok((ids.clone(), serde_json::json!({ "node_ids": ids })));
    }

    if !selector_labels.is_empty() {
        let labels = normalize_labels(selector_labels)?;
        let mode = parse_labels_mode(selector.labels_mode.as_deref())?;
        let mode_str = match mode {
            LabelsMode::And => "and",
            LabelsMode::Or => "or",
        };

        let ids = bulk_operations_repo::resolve_agent_ids_by_selector_labels(db, &labels, mode_str)
            .await?;
        if ids.is_empty() {
            return Err(AppError::bad_request(
                "invalid_selector",
                "Selector resolved to no agents",
            ));
        }
        return Ok((
            ids,
            serde_json::json!({ "labels": labels, "labels_mode": mode_str }),
        ));
    }

    Err(AppError::bad_request(
        "invalid_selector",
        "Selector is required",
    ))
}

pub(in crate::http) async fn create_bulk_operation(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Json(req): Json<CreateBulkOperationRequest>,
) -> Result<Json<CreateBulkOperationResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let kind = validate_kind(req.kind.trim())?.to_string();

    let (target_agent_ids, selector_json) = resolve_selector(&state.db, req.selector).await?;

    let payload_json = match kind.as_str() {
        "agent_labels_add" | "agent_labels_remove" => {
            let Some(payload) = req.payload else {
                return Err(AppError::bad_request(
                    "invalid_payload",
                    "Payload is required",
                ));
            };
            let parsed: AgentLabelsPayloadRequest = serde_json::from_value(payload)
                .map_err(|_| AppError::bad_request("invalid_payload", "Invalid payload"))?;

            let payload_labels = normalize_labels(parsed.labels)?;
            if payload_labels.is_empty() {
                return Err(AppError::bad_request(
                    "invalid_payload",
                    "Labels is required",
                ));
            }

            serde_json::json!({ "labels": payload_labels })
        }
        "sync_config_now" => serde_json::json!({}),
        "job_deploy" => {
            let Some(payload) = req.payload else {
                return Err(AppError::bad_request(
                    "invalid_payload",
                    "Payload is required",
                ));
            };
            let parsed: JobDeployPayloadRequest = serde_json::from_value(payload)
                .map_err(|_| AppError::bad_request("invalid_payload", "Invalid payload"))?;

            let source_job_id = parsed.source_job_id.trim().to_string();
            if source_job_id.is_empty() {
                return Err(AppError::bad_request(
                    "invalid_payload",
                    "source_job_id is required",
                ));
            }

            let source = jobs_repo::get_job(&state.db, &source_job_id).await?;
            if source.is_none() {
                return Err(AppError::bad_request(
                    "invalid_payload",
                    "Source job not found",
                ));
            }

            let template = parsed.name_template.unwrap_or_default().trim().to_string();
            let template = if template.is_empty() {
                "{name} ({node})".to_string()
            } else {
                template
            };

            serde_json::json!({
                "source_job_id": source_job_id,
                "name_template": template,
            })
        }
        "webdav_secret_distribute" => {
            let Some(payload) = req.payload else {
                return Err(AppError::bad_request(
                    "invalid_payload",
                    "Payload is required",
                ));
            };
            let parsed: WebdavDistributePayloadRequest = serde_json::from_value(payload)
                .map_err(|_| AppError::bad_request("invalid_payload", "Invalid payload"))?;

            let name = parsed.name.trim().to_string();
            if name.is_empty() {
                return Err(AppError::bad_request("invalid_payload", "Name is required"));
            }

            let exists =
                secrets_repo::secret_exists(&state.db, HUB_NODE_ID, "webdav", &name).await?;
            if !exists {
                return Err(AppError::bad_request(
                    "invalid_payload",
                    "WebDAV credential not found",
                ));
            }

            serde_json::json!({
                "name": name,
                "overwrite": parsed.overwrite.unwrap_or(false),
            })
        }
        _ => return Err(AppError::bad_request("invalid_kind", "Invalid kind")),
    };

    let op_id = bulk_operations_repo::create_operation(
        &state.db,
        session.user_id,
        &kind,
        &selector_json,
        &payload_json,
        &target_agent_ids,
    )
    .await?;

    state.bulk_ops_notify.notify_one();

    Ok(Json(CreateBulkOperationResponse { op_id }))
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct WebdavDistributePreviewItem {
    agent_id: String,
    agent_name: Option<String>,
    action: String,
    note: Option<String>,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct JobDeployPreviewItem {
    agent_id: String,
    agent_name: Option<String>,
    planned_name: String,
    valid: bool,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind")]
pub(in crate::http) enum BulkOperationPreviewResponse {
    #[serde(rename = "webdav_secret_distribute")]
    WebdavSecretDistribute {
        secret_name: String,
        overwrite: bool,
        items: Vec<WebdavDistributePreviewItem>,
    },
    #[serde(rename = "job_deploy")]
    JobDeploy {
        source_job_id: String,
        name_template: String,
        items: Vec<JobDeployPreviewItem>,
    },
}

pub(in crate::http) async fn preview_bulk_operation(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Json(req): Json<CreateBulkOperationRequest>,
) -> Result<Json<BulkOperationPreviewResponse>, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let kind = validate_kind(req.kind.trim())?.to_string();
    let (target_agent_ids, _selector_json) = resolve_selector(&state.db, req.selector).await?;

    let Some(payload) = req.payload else {
        return Err(AppError::bad_request(
            "invalid_payload",
            "Payload is required",
        ));
    };
    match kind.as_str() {
        "webdav_secret_distribute" => {
            let parsed: WebdavDistributePayloadRequest = serde_json::from_value(payload)
                .map_err(|_| AppError::bad_request("invalid_payload", "Invalid payload"))?;

            let secret_name = parsed.name.trim().to_string();
            if secret_name.is_empty() {
                return Err(AppError::bad_request("invalid_payload", "Name is required"));
            }
            let overwrite = parsed.overwrite.unwrap_or(false);

            let exists =
                secrets_repo::secret_exists(&state.db, HUB_NODE_ID, "webdav", &secret_name).await?;
            if !exists {
                return Err(AppError::bad_request(
                    "invalid_payload",
                    "WebDAV credential not found",
                ));
            }

            let mut qb: sqlx::QueryBuilder<sqlx::Sqlite> = sqlx::QueryBuilder::new(
                "SELECT node_id FROM secrets WHERE kind = 'webdav' AND name = ",
            );
            qb.push_bind(&secret_name);
            qb.push(" AND node_id IN (");
            let mut separated = qb.separated(", ");
            for id in &target_agent_ids {
                separated.push_bind(id);
            }
            separated.push_unseparated(")");

            let rows = qb.build().fetch_all(&state.db).await?;
            let existing = rows
                .into_iter()
                .map(|r| r.get::<String, _>("node_id"))
                .collect::<std::collections::HashSet<String>>();

            let mut qb: sqlx::QueryBuilder<sqlx::Sqlite> =
                sqlx::QueryBuilder::new("SELECT id, name FROM agents WHERE id IN (");
            let mut separated = qb.separated(", ");
            for id in &target_agent_ids {
                separated.push_bind(id);
            }
            separated.push_unseparated(")");
            let rows = qb.build().fetch_all(&state.db).await?;
            let names = rows
                .into_iter()
                .map(|r| (r.get::<String, _>("id"), r.get::<Option<String>, _>("name")))
                .collect::<std::collections::HashMap<String, Option<String>>>();

            let mut items = Vec::with_capacity(target_agent_ids.len());
            for agent_id in target_agent_ids {
                let has = existing.contains(&agent_id);
                let (action, note) = if has && !overwrite {
                    ("skip", Some("already exists".to_string()))
                } else {
                    ("update", None)
                };
                items.push(WebdavDistributePreviewItem {
                    agent_id: agent_id.clone(),
                    agent_name: names.get(&agent_id).cloned().unwrap_or(None),
                    action: action.to_string(),
                    note,
                });
            }

            Ok(Json(BulkOperationPreviewResponse::WebdavSecretDistribute {
                secret_name,
                overwrite,
                items,
            }))
        }
        "job_deploy" => {
            use bastion_core::job_spec;
            use bastion_engine::agent_job_resolver;

            let parsed: JobDeployPayloadRequest = serde_json::from_value(payload)
                .map_err(|_| AppError::bad_request("invalid_payload", "Invalid payload"))?;

            let source_job_id = parsed.source_job_id.trim().to_string();
            if source_job_id.is_empty() {
                return Err(AppError::bad_request(
                    "invalid_payload",
                    "source_job_id is required",
                ));
            }

            let Some(source) = jobs_repo::get_job(&state.db, &source_job_id).await? else {
                return Err(AppError::bad_request(
                    "invalid_payload",
                    "Source job not found",
                ));
            };

            let name_template = parsed.name_template.unwrap_or_default().trim().to_string();
            let name_template = if name_template.is_empty() {
                "{name} ({node})".to_string()
            } else {
                name_template
            };

            let mut qb: sqlx::QueryBuilder<sqlx::Sqlite> =
                sqlx::QueryBuilder::new("SELECT id, name FROM agents WHERE id IN (");
            let mut separated = qb.separated(", ");
            for id in &target_agent_ids {
                separated.push_bind(id);
            }
            separated.push_unseparated(")");
            let rows = qb.build().fetch_all(&state.db).await?;
            let names = rows
                .into_iter()
                .map(|r| (r.get::<String, _>("id"), r.get::<Option<String>, _>("name")))
                .collect::<std::collections::HashMap<String, Option<String>>>();

            let spec = job_spec::parse_value(&source.spec)
                .map_err(|_| AppError::bad_request("invalid_payload", "Invalid job spec"))?;
            if let Err(error) = job_spec::validate(&spec) {
                return Err(AppError::bad_request(
                    "invalid_payload",
                    format!("Invalid job spec: {error}"),
                ));
            }

            let mut items = Vec::with_capacity(target_agent_ids.len());
            for agent_id in target_agent_ids {
                let planned_base = name_template
                    .replace("{name}", &source.name)
                    .replace("{node}", &agent_id)
                    .trim()
                    .to_string();

                if planned_base.is_empty() {
                    items.push(JobDeployPreviewItem {
                        agent_id: agent_id.clone(),
                        agent_name: names.get(&agent_id).cloned().unwrap_or(None),
                        planned_name: planned_base,
                        valid: false,
                        error: Some("name_template produced empty name".to_string()),
                    });
                    continue;
                }

                let mut planned_name = planned_base.clone();
                let existing = jobs_repo::list_jobs_for_agent(&state.db, &agent_id).await?;
                let used = existing
                    .into_iter()
                    .map(|j| j.name)
                    .collect::<std::collections::HashSet<String>>();
                if used.contains(&planned_name) {
                    let mut i = 2;
                    loop {
                        let candidate = format!("{planned_base} #{i}");
                        if !used.contains(&candidate) {
                            planned_name = candidate;
                            break;
                        }
                        i += 1;
                    }
                }

                let validation = agent_job_resolver::resolve_job_spec_for_agent(
                    &state.db,
                    state.secrets.as_ref(),
                    &agent_id,
                    spec.clone(),
                )
                .await;

                match validation {
                    Ok(_) => items.push(JobDeployPreviewItem {
                        agent_id: agent_id.clone(),
                        agent_name: names.get(&agent_id).cloned().unwrap_or(None),
                        planned_name,
                        valid: true,
                        error: None,
                    }),
                    Err(error) => items.push(JobDeployPreviewItem {
                        agent_id: agent_id.clone(),
                        agent_name: names.get(&agent_id).cloned().unwrap_or(None),
                        planned_name,
                        valid: false,
                        error: Some(error.to_string()),
                    }),
                }
            }

            Ok(Json(BulkOperationPreviewResponse::JobDeploy {
                source_job_id,
                name_template,
                items,
            }))
        }
        _ => Err(AppError::bad_request(
            "preview_not_supported",
            "Preview not supported for this kind",
        )),
    }
}

pub(in crate::http) async fn list_bulk_operations(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<Vec<bulk_operations_repo::BulkOperationListItem>>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let ops = bulk_operations_repo::list_operations(&state.db, DEFAULT_LIST_LIMIT).await?;
    Ok(Json(ops))
}

pub(in crate::http) async fn get_bulk_operation(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    Path(op_id): Path<String>,
) -> Result<Json<bulk_operations_repo::BulkOperationDetail>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let op = bulk_operations_repo::get_operation(&state.db, &op_id)
        .await?
        .ok_or_else(|| {
            AppError::not_found("bulk_operation_not_found", "Bulk operation not found")
        })?;
    Ok(Json(op))
}

pub(in crate::http) async fn cancel_bulk_operation(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(op_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    bulk_operations_repo::cancel_operation(&state.db, &op_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(in crate::http) async fn retry_bulk_operation_failed(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Path(op_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    let changed = bulk_operations_repo::retry_failed(&state.db, &op_id)
        .await
        .map_err(|_| AppError::bad_request("cannot_retry", "Cannot retry this operation"))?;
    if changed > 0 {
        state.bulk_ops_notify.notify_one();
    }
    Ok(StatusCode::NO_CONTENT)
}
