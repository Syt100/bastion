use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug)]
pub(in crate::http) struct AppError {
    status: StatusCode,
    code: &'static str,
    message: String,
    details: Option<serde_json::Value>,
}

impl AppError {
    pub(in crate::http) fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: message.into(),
            details: None,
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
        }
    }

    pub(in crate::http) fn unauthorized(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code,
            message: message.into(),
            details: None,
        }
    }

    pub(in crate::http) fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code,
            message: message.into(),
            details: None,
        }
    }

    pub(in crate::http) fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code,
            message: message.into(),
            details: None,
        }
    }

    pub(in crate::http) fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        let error: anyhow::Error = error.into();
        tracing::error!(error = %error, "request failed");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: "Internal server error".to_string(),
            details: None,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct Body {
            error: &'static str,
            message: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            details: Option<serde_json::Value>,
        }

        let body = Json(Body {
            error: self.code,
            message: self.message,
            details: self.details,
        });
        (self.status, body).into_response()
    }
}
