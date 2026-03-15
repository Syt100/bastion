use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use serde::Serialize;
use tower_cookies::Cookies;

use bastion_storage::hub_runtime_config_repo;

use super::shared::{require_csrf, require_session};
use super::{AppError, AppState, ConfigValueSource, normalize_public_base_url};

#[derive(Debug, Serialize)]
pub(in crate::http) struct HubRuntimeConfigFieldMeta {
    env: &'static str,
    source: ConfigValueSource,
    editable: bool,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct HubRuntimeConfigFieldsMeta {
    bind_host: HubRuntimeConfigFieldMeta,
    bind_port: HubRuntimeConfigFieldMeta,
    data_dir: HubRuntimeConfigFieldMeta,
    insecure_http: HubRuntimeConfigFieldMeta,
    trusted_proxies: HubRuntimeConfigFieldMeta,
    debug_errors: HubRuntimeConfigFieldMeta,

    hub_timezone: HubRuntimeConfigFieldMeta,
    run_retention_days: HubRuntimeConfigFieldMeta,
    incomplete_cleanup_days: HubRuntimeConfigFieldMeta,
    public_base_url: HubRuntimeConfigFieldMeta,

    log_filter: HubRuntimeConfigFieldMeta,
    log_file: HubRuntimeConfigFieldMeta,
    log_rotation: HubRuntimeConfigFieldMeta,
    log_keep_files: HubRuntimeConfigFieldMeta,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct HubRuntimeConfigEffective {
    bind_host: String,
    bind_port: u16,
    data_dir: String,
    insecure_http: bool,
    trusted_proxies: Vec<String>,
    debug_errors: bool,

    hub_timezone: String,
    run_retention_days: i64,
    incomplete_cleanup_days: i64,
    public_base_url: Option<String>,

    log_filter: String,
    log_file: Option<String>,
    log_rotation: String,
    log_keep_files: usize,
}

#[derive(Debug, Serialize)]
pub(in crate::http) struct HubRuntimeConfigGetResponse {
    requires_restart: bool,
    effective: HubRuntimeConfigEffective,
    saved: hub_runtime_config_repo::HubRuntimeConfig,
    fields: HubRuntimeConfigFieldsMeta,
}

fn is_overridden(source: ConfigValueSource) -> bool {
    matches!(
        source,
        ConfigValueSource::Cli | ConfigValueSource::Env | ConfigValueSource::EnvRustLog
    )
}

fn editable_policy_field(source: ConfigValueSource) -> bool {
    !is_overridden(source)
}

pub(in crate::http) async fn get_hub_runtime_config(
    state: State<AppState>,
    cookies: Cookies,
) -> Result<Json<HubRuntimeConfigGetResponse>, AppError> {
    let _session = require_session(&state, &cookies).await?;

    let saved = hub_runtime_config_repo::get(&state.db)
        .await?
        .unwrap_or_default();

    let sources = &state.hub_runtime_config.sources;

    let fields = HubRuntimeConfigFieldsMeta {
        bind_host: HubRuntimeConfigFieldMeta {
            env: "BASTION_HOST",
            source: sources.bind_host,
            editable: false,
        },
        bind_port: HubRuntimeConfigFieldMeta {
            env: "BASTION_PORT",
            source: sources.bind_port,
            editable: false,
        },
        data_dir: HubRuntimeConfigFieldMeta {
            env: "BASTION_DATA_DIR",
            source: sources.data_dir,
            editable: false,
        },
        insecure_http: HubRuntimeConfigFieldMeta {
            env: "BASTION_INSECURE_HTTP",
            source: sources.insecure_http,
            editable: false,
        },
        trusted_proxies: HubRuntimeConfigFieldMeta {
            env: "BASTION_TRUSTED_PROXIES",
            source: sources.trusted_proxies,
            editable: false,
        },
        debug_errors: HubRuntimeConfigFieldMeta {
            env: "BASTION_DEBUG_ERRORS",
            source: sources.debug_errors,
            editable: false,
        },
        hub_timezone: HubRuntimeConfigFieldMeta {
            env: "BASTION_HUB_TIMEZONE",
            source: sources.hub_timezone,
            editable: editable_policy_field(sources.hub_timezone),
        },
        run_retention_days: HubRuntimeConfigFieldMeta {
            env: "BASTION_RUN_RETENTION_DAYS",
            source: sources.run_retention_days,
            editable: editable_policy_field(sources.run_retention_days),
        },
        incomplete_cleanup_days: HubRuntimeConfigFieldMeta {
            env: "BASTION_INCOMPLETE_CLEANUP_DAYS",
            source: sources.incomplete_cleanup_days,
            editable: editable_policy_field(sources.incomplete_cleanup_days),
        },
        public_base_url: HubRuntimeConfigFieldMeta {
            env: "BASTION_PUBLIC_BASE_URL",
            source: sources.public_base_url,
            editable: editable_policy_field(sources.public_base_url),
        },
        log_filter: HubRuntimeConfigFieldMeta {
            env: "BASTION_LOG / RUST_LOG",
            source: sources.log_filter,
            editable: editable_policy_field(sources.log_filter),
        },
        log_file: HubRuntimeConfigFieldMeta {
            env: "BASTION_LOG_FILE",
            source: sources.log_file,
            editable: editable_policy_field(sources.log_file),
        },
        log_rotation: HubRuntimeConfigFieldMeta {
            env: "BASTION_LOG_ROTATION",
            source: sources.log_rotation,
            editable: editable_policy_field(sources.log_rotation),
        },
        log_keep_files: HubRuntimeConfigFieldMeta {
            env: "BASTION_LOG_KEEP_FILES",
            source: sources.log_keep_files,
            editable: editable_policy_field(sources.log_keep_files),
        },
    };

    let bind_host = state.config.bind.ip().to_string();
    let bind_port = state.config.bind.port();
    let trusted_proxies = state
        .config
        .trusted_proxies
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    let effective = HubRuntimeConfigEffective {
        bind_host,
        bind_port,
        data_dir: state.config.data_dir.display().to_string(),
        insecure_http: state.config.insecure_http,
        trusted_proxies,
        debug_errors: state.config.debug_errors,
        hub_timezone: state.config.hub_timezone.clone(),
        run_retention_days: state.config.run_retention_days,
        incomplete_cleanup_days: state.config.incomplete_cleanup_days,
        public_base_url: state.hub_runtime_config.public_base_url.clone(),
        log_filter: state.hub_runtime_config.logging.filter.clone(),
        log_file: state.hub_runtime_config.logging.file.clone(),
        log_rotation: state.hub_runtime_config.logging.rotation.clone(),
        log_keep_files: state.hub_runtime_config.logging.keep_files,
    };

    Ok(Json(HubRuntimeConfigGetResponse {
        requires_restart: true,
        effective,
        saved,
        fields,
    }))
}

fn normalize_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn normalize_rotation(value: Option<&str>) -> Result<Option<String>, AppError> {
    let v = normalize_optional_string(value).map(|v| v.to_lowercase());
    let Some(v) = v else { return Ok(None) };
    if v == "daily" || v == "hourly" || v == "never" {
        Ok(Some(v))
    } else {
        Err(
            AppError::bad_request("invalid_log_rotation", "Invalid log rotation")
                .with_reason("invalid_value")
                .with_field("log_rotation")
                .with_param("allowed", ["daily", "hourly", "never"]),
        )
    }
}

fn validate_timezone(value: Option<&str>) -> Result<Option<String>, AppError> {
    let v = normalize_optional_string(value);
    let Some(v) = v else { return Ok(None) };
    let _ = v.parse::<chrono_tz::Tz>().map_err(|_| {
        AppError::bad_request("invalid_timezone", "Invalid hub timezone")
            .with_reason("invalid_format")
            .with_field("hub_timezone")
    })?;
    Ok(Some(v))
}

pub(in crate::http) async fn put_hub_runtime_config(
    state: State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Json(mut req): Json<hub_runtime_config_repo::HubRuntimeConfig>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    if let Some(v) = req.run_retention_days
        && v <= 0
    {
        return Err(AppError::bad_request(
            "invalid_run_retention_days",
            "run_retention_days must be > 0",
        )
        .with_reason("must_be_positive")
        .with_field("run_retention_days")
        .with_param("min", 1));
    }

    if let Some(v) = req.incomplete_cleanup_days
        && v < 0
    {
        return Err(AppError::bad_request(
            "invalid_incomplete_cleanup_days",
            "incomplete_cleanup_days must be >= 0",
        )
        .with_reason("min_value")
        .with_field("incomplete_cleanup_days")
        .with_param("min", 0));
    }

    {
        const MAX_KEEP_LAST: u32 = 10_000;
        const MAX_KEEP_DAYS: u32 = 3650;
        const MAX_DELETE_PER_TICK: u32 = 10_000;
        const MAX_DELETE_PER_DAY: u32 = 100_000;

        let r = &req.default_backup_retention;

        if let Some(v) = r.keep_last
            && v > MAX_KEEP_LAST
        {
            return Err(AppError::bad_request(
                "invalid_default_backup_retention",
                format!("default_backup_retention.keep_last must be <= {MAX_KEEP_LAST}"),
            )
            .with_reason("max_exceeded")
            .with_field("default_backup_retention.keep_last")
            .with_param("max", MAX_KEEP_LAST));
        }

        if let Some(v) = r.keep_days
            && v > MAX_KEEP_DAYS
        {
            return Err(AppError::bad_request(
                "invalid_default_backup_retention",
                format!("default_backup_retention.keep_days must be <= {MAX_KEEP_DAYS}"),
            )
            .with_reason("max_exceeded")
            .with_field("default_backup_retention.keep_days")
            .with_param("max", MAX_KEEP_DAYS));
        }

        if r.max_delete_per_tick == 0 || r.max_delete_per_tick > MAX_DELETE_PER_TICK {
            return Err(AppError::bad_request(
                "invalid_default_backup_retention",
                format!(
                    "default_backup_retention.max_delete_per_tick must be within 1..={MAX_DELETE_PER_TICK}"
                ),
            )
            .with_reason("out_of_range")
            .with_field("default_backup_retention.max_delete_per_tick")
            .with_param("min", 1)
            .with_param("max", MAX_DELETE_PER_TICK));
        }

        if r.max_delete_per_day == 0 || r.max_delete_per_day > MAX_DELETE_PER_DAY {
            return Err(AppError::bad_request(
                "invalid_default_backup_retention",
                format!(
                    "default_backup_retention.max_delete_per_day must be within 1..={MAX_DELETE_PER_DAY}"
                ),
            )
            .with_reason("out_of_range")
            .with_field("default_backup_retention.max_delete_per_day")
            .with_param("min", 1)
            .with_param("max", MAX_DELETE_PER_DAY));
        }

        if r.enabled {
            let keep_last = r.keep_last.unwrap_or(0);
            let keep_days = r.keep_days.unwrap_or(0);
            if keep_last == 0 && keep_days == 0 {
                return Err(AppError::bad_request(
                    "invalid_default_backup_retention",
                    "default_backup_retention.enabled is true but both keep rules are empty",
                )
                .with_reason("keep_rule_required")
                .with_field("default_backup_retention")
                .with_violation(
                    "default_backup_retention.keep_last",
                    "required_when_enabled",
                    None,
                )
                .with_violation(
                    "default_backup_retention.keep_days",
                    "required_when_enabled",
                    None,
                ));
            }
        }
    }

    req.hub_timezone = validate_timezone(req.hub_timezone.as_deref())?;
    req.public_base_url =
        normalize_public_base_url(req.public_base_url.as_deref()).map_err(|reason| {
            let (reason, message) = match reason.as_str() {
                "unsupported_scheme" => (
                    "unsupported_scheme",
                    "public_base_url must use http or https",
                ),
                "query_or_fragment_not_allowed" => (
                    "query_or_fragment_not_allowed",
                    "public_base_url must not include query or fragment",
                ),
                "missing_host" => ("missing_host", "public_base_url must include a host"),
                _ => ("invalid_format", "Invalid public base URL"),
            };
            AppError::bad_request("invalid_public_base_url", message)
                .with_reason(reason)
                .with_field("public_base_url")
        })?;
    req.log_filter = normalize_optional_string(req.log_filter.as_deref());
    req.log_file = normalize_optional_string(req.log_file.as_deref());
    req.log_rotation = normalize_rotation(req.log_rotation.as_deref())?;

    hub_runtime_config_repo::upsert(&state.db, &req).await?;
    tracing::info!("hub runtime config saved (restart required)");
    Ok(StatusCode::NO_CONTENT)
}
