use std::future::Future;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::http) struct AppErrorRenderOptions {
    pub debug_errors: bool,
}

tokio::task_local! {
    static APP_ERROR_RENDER_OPTIONS: AppErrorRenderOptions;
}

pub(in crate::http) async fn with_app_error_render_options<T>(
    options: AppErrorRenderOptions,
    future: impl Future<Output = T>,
) -> T {
    APP_ERROR_RENDER_OPTIONS.scope(options, future).await
}

fn current_app_error_render_options() -> AppErrorRenderOptions {
    APP_ERROR_RENDER_OPTIONS
        .try_with(|options| *options)
        .unwrap_or_default()
}

#[derive(Debug)]
pub(in crate::http) struct AppError {
    status: StatusCode,
    code: &'static str,
    message: String,
    details: Option<serde_json::Value>,
    debug_details: Option<serde_json::Value>,
}

impl AppError {
    #[cfg(test)]
    pub(in crate::http) fn code(&self) -> &'static str {
        self.code
    }

    #[cfg(test)]
    pub(in crate::http) fn status(&self) -> StatusCode {
        self.status
    }

    #[cfg(test)]
    pub(in crate::http) fn details(&self) -> Option<&serde_json::Value> {
        self.details.as_ref()
    }

    pub(in crate::http) fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: message.into(),
            details: None,
            debug_details: None,
        }
    }

    pub(in crate::http) fn too_many_requests(
        code: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            code,
            message: message.into(),
            details: None,
            debug_details: None,
        }
    }

    pub(in crate::http) fn unauthorized(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code,
            message: message.into(),
            details: None,
            debug_details: None,
        }
    }

    pub(in crate::http) fn forbidden(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code,
            message: message.into(),
            details: None,
            debug_details: None,
        }
    }

    pub(in crate::http) fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code,
            message: message.into(),
            details: None,
            debug_details: None,
        }
    }

    pub(in crate::http) fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code,
            message: message.into(),
            details: None,
            debug_details: None,
        }
    }

    fn ensure_details_object(&mut self) -> &mut serde_json::Map<String, serde_json::Value> {
        let replace = !matches!(self.details, Some(serde_json::Value::Object(_)));
        if replace {
            self.details = Some(serde_json::Value::Object(serde_json::Map::new()));
        }
        match self.details {
            Some(serde_json::Value::Object(ref mut obj)) => obj,
            _ => unreachable!("details should be an object"),
        }
    }

    fn ensure_params_object(&mut self) -> &mut serde_json::Map<String, serde_json::Value> {
        let details = self.ensure_details_object();
        let replace = !matches!(details.get("params"), Some(serde_json::Value::Object(_)));
        if replace {
            details.insert(
                "params".to_string(),
                serde_json::Value::Object(serde_json::Map::new()),
            );
        }
        match details.get_mut("params") {
            Some(serde_json::Value::Object(obj)) => obj,
            _ => unreachable!("params should be an object"),
        }
    }

    pub(in crate::http) fn with_details(mut self, details: serde_json::Value) -> Self {
        match details {
            serde_json::Value::Object(obj) => {
                self.ensure_details_object().extend(obj);
            }
            value => {
                self.details = Some(value);
            }
        }
        self
    }

    pub(in crate::http) fn with_reason(mut self, reason: &'static str) -> Self {
        self.ensure_details_object().insert(
            "reason".to_string(),
            serde_json::Value::String(reason.to_string()),
        );
        self
    }

    pub(in crate::http) fn with_field(mut self, field: &'static str) -> Self {
        self.ensure_details_object().insert(
            "field".to_string(),
            serde_json::Value::String(field.to_string()),
        );
        self
    }

    pub(in crate::http) fn with_param(mut self, key: &'static str, value: impl Serialize) -> Self {
        let value = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
        self.ensure_params_object().insert(key.to_string(), value);
        self
    }

    pub(in crate::http) fn with_violation(
        mut self,
        field: &'static str,
        reason: &'static str,
        params: Option<serde_json::Value>,
    ) -> Self {
        let details = self.ensure_details_object();
        let replace = !matches!(details.get("violations"), Some(serde_json::Value::Array(_)));
        if replace {
            details.insert(
                "violations".to_string(),
                serde_json::Value::Array(Vec::new()),
            );
        }

        if let Some(serde_json::Value::Array(violations)) = details.get_mut("violations") {
            let mut violation = serde_json::Map::new();
            violation.insert(
                "field".to_string(),
                serde_json::Value::String(field.to_string()),
            );
            violation.insert(
                "reason".to_string(),
                serde_json::Value::String(reason.to_string()),
            );
            if let Some(serde_json::Value::Object(params_obj)) = params {
                violation.insert("params".to_string(), serde_json::Value::Object(params_obj));
            }
            violations.push(serde_json::Value::Object(violation));
        }

        self
    }
}

fn classify_error(error: &anyhow::Error) -> Option<AppError> {
    for cause in error.chain() {
        if let Some(io) = cause.downcast_ref::<std::io::Error>() {
            return match io.kind() {
                std::io::ErrorKind::PermissionDenied => Some(AppError::forbidden(
                    "permission_denied",
                    "Permission denied",
                )),
                std::io::ErrorKind::NotFound => {
                    Some(AppError::not_found("path_not_found", "Path not found"))
                }
                _ => None,
            };
        }

        if let Some(sqlx_error) = cause.downcast_ref::<sqlx::Error>()
            && matches!(sqlx_error, sqlx::Error::RowNotFound)
        {
            return Some(AppError::not_found("not_found", "Not found"));
        }
    }

    None
}

fn debug_details(error: &anyhow::Error) -> serde_json::Value {
    let chain = error
        .chain()
        .take(8)
        .map(|cause| {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "type".to_string(),
                serde_json::Value::String(std::any::type_name_of_val(cause).to_string()),
            );
            if let Some(io) = cause.downcast_ref::<std::io::Error>() {
                obj.insert(
                    "io_kind".to_string(),
                    serde_json::Value::String(format!("{:?}", io.kind())),
                );
            }
            if let Some(sqlx_error) = cause.downcast_ref::<sqlx::Error>()
                && matches!(sqlx_error, sqlx::Error::RowNotFound)
            {
                obj.insert(
                    "sqlx_kind".to_string(),
                    serde_json::Value::String("RowNotFound".to_string()),
                );
            }
            serde_json::Value::Object(obj)
        })
        .collect::<Vec<_>>();

    serde_json::json!({ "debug": { "chain": chain } })
}

fn merge_response_details(
    details: Option<serde_json::Value>,
    debug_details: Option<serde_json::Value>,
    options: AppErrorRenderOptions,
) -> Option<serde_json::Value> {
    if !options.debug_errors {
        return details;
    }

    match (details, debug_details) {
        (None, debug_details) => debug_details,
        (details, None) => details,
        (
            Some(serde_json::Value::Object(mut details)),
            Some(serde_json::Value::Object(debug_details)),
        ) => {
            details.extend(debug_details);
            Some(serde_json::Value::Object(details))
        }
        (details, Some(_debug_details)) => details,
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        let error: anyhow::Error = error.into();

        if let Some(classified) = classify_error(&error) {
            tracing::debug!(error = %error, code = classified.code, "request failed");
            return classified;
        }

        tracing::error!(error = %error, "request failed");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: "Internal server error".to_string(),
            details: None,
            debug_details: Some(debug_details(&error)),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.into_response_with_options(current_app_error_render_options())
    }
}

impl AppError {
    pub(in crate::http) fn into_response_with_options(
        self,
        options: AppErrorRenderOptions,
    ) -> Response {
        #[derive(Serialize)]
        struct Body {
            error: &'static str,
            message: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            details: Option<serde_json::Value>,
        }

        let details = merge_response_details(self.details, self.debug_details, options);
        let body = Json(Body {
            error: self.code,
            message: self.message,
            details,
        });
        (self.status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    use axum::body::to_bytes;
    use axum::response::IntoResponse;

    use super::{AppError, AppErrorRenderOptions, with_app_error_render_options};

    #[test]
    fn classify_permission_denied_as_403() {
        let err = anyhow::Error::new(std::io::Error::from(ErrorKind::PermissionDenied))
            .context("stat failed");
        let app: AppError = err.into();
        assert_eq!(app.status, axum::http::StatusCode::FORBIDDEN);
        assert_eq!(app.code, "permission_denied");
    }

    #[test]
    fn classify_not_found_as_404() {
        let err = anyhow::Error::new(std::io::Error::from(ErrorKind::NotFound)).context("stat");
        let app: AppError = err.into();
        assert_eq!(app.status, axum::http::StatusCode::NOT_FOUND);
        assert_eq!(app.code, "path_not_found");
    }

    #[test]
    fn classify_sqlx_row_not_found_as_404() {
        let err = anyhow::Error::new(sqlx::Error::RowNotFound);
        let app: AppError = err.into();
        assert_eq!(app.status, axum::http::StatusCode::NOT_FOUND);
        assert_eq!(app.code, "not_found");
    }

    #[test]
    fn structured_details_helpers_merge_correctly() {
        let app = AppError::bad_request("invalid_password", "Password is invalid")
            .with_reason("min_length")
            .with_field("password")
            .with_param("min_length", 12)
            .with_details(serde_json::json!({ "legacy": true }))
            .with_violation(
                "password",
                "min_length",
                Some(serde_json::json!({ "min_length": 12 })),
            );

        let details = app.details.expect("details");
        assert_eq!(details["reason"], "min_length");
        assert_eq!(details["field"], "password");
        assert_eq!(details["params"]["min_length"], 12);
        assert_eq!(details["legacy"], true);
        assert_eq!(details["violations"][0]["field"], "password");
        assert_eq!(details["violations"][0]["reason"], "min_length");
        assert_eq!(details["violations"][0]["params"]["min_length"], 12);
    }

    async fn render_internal_error_details(
        options: AppErrorRenderOptions,
    ) -> Option<serde_json::Value> {
        with_app_error_render_options(options, async {
            let app: AppError = anyhow::anyhow!("boom").into();
            let resp = app.into_response();
            let bytes = to_bytes(resp.into_body(), 1024 * 1024)
                .await
                .expect("body bytes");
            let body: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
            body.get("details").cloned()
        })
        .await
    }

    #[tokio::test]
    async fn debug_details_are_gated_by_render_options() {
        let disabled = render_internal_error_details(AppErrorRenderOptions {
            debug_errors: false,
        })
        .await;
        assert!(disabled.is_none());

        let enabled = render_internal_error_details(AppErrorRenderOptions { debug_errors: true })
            .await
            .expect("details");
        assert!(enabled.get("debug").is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn render_options_do_not_leak_between_concurrent_scopes() {
        let (disabled, enabled) = tokio::join!(
            render_internal_error_details(AppErrorRenderOptions {
                debug_errors: false,
            }),
            render_internal_error_details(AppErrorRenderOptions { debug_errors: true }),
        );

        assert!(disabled.is_none());
        assert!(
            enabled
                .as_ref()
                .and_then(|value| value.get("debug"))
                .is_some()
        );
    }
}
