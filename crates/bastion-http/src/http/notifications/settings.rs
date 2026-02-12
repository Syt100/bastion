use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use time::OffsetDateTime;
use tower_cookies::Cookies;

use bastion_storage::notifications_repo;
use bastion_storage::notifications_settings_repo;

use super::super::shared::{require_csrf, require_session};
use super::super::{AppError, AppState};

pub(in crate::http) async fn get_settings(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<notifications_settings_repo::NotificationsSettings>, AppError> {
    let _session = require_session(&state, &cookies).await?;
    let settings = notifications_settings_repo::get_or_default(&state.db).await?;
    Ok(Json(settings))
}

fn invalid_template_error(
    reason: &'static str,
    field: &'static str,
    message: &'static str,
) -> AppError {
    AppError::bad_request("invalid_template", message)
        .with_reason(reason)
        .with_field(field)
}

pub(in crate::http) async fn put_settings(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    Json(req): Json<notifications_settings_repo::NotificationsSettings>,
) -> Result<StatusCode, AppError> {
    let session = require_session(&state, &cookies).await?;
    require_csrf(&headers, &session)?;

    if req.templates.wecom_markdown.trim().is_empty() {
        return Err(invalid_template_error(
            "required_wecom_markdown",
            "templates.wecom_markdown",
            "WeCom template is required",
        ));
    }
    if req.templates.email_subject.trim().is_empty() {
        return Err(invalid_template_error(
            "required_email_subject",
            "templates.email_subject",
            "Email subject template is required",
        ));
    }
    if req.templates.email_body.trim().is_empty() {
        return Err(invalid_template_error(
            "required_email_body",
            "templates.email_body",
            "Email body template is required",
        ));
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    if !req.enabled {
        let _ = notifications_repo::cancel_all_queued(
            &state.db,
            "canceled: notifications disabled",
            now,
        )
        .await?;
    } else {
        if !req.channels.wecom_bot.enabled {
            let _ = notifications_repo::cancel_queued_for_channel(
                &state.db,
                notifications_repo::CHANNEL_WECOM_BOT,
                "canceled: channel disabled",
                now,
            )
            .await?;
        }
        if !req.channels.email.enabled {
            let _ = notifications_repo::cancel_queued_for_channel(
                &state.db,
                notifications_repo::CHANNEL_EMAIL,
                "canceled: channel disabled",
                now,
            )
            .await?;
        }
    }

    notifications_settings_repo::upsert(&state.db, &req).await?;
    tracing::info!(
        enabled = req.enabled,
        wecom_enabled = req.channels.wecom_bot.enabled,
        email_enabled = req.channels.email.enabled,
        "notification settings updated"
    );
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_template_error_sets_reason_and_field() {
        let app = invalid_template_error(
            "required_email_subject",
            "templates.email_subject",
            "Email subject template is required",
        );

        assert_eq!(app.code(), "invalid_template");
        assert_eq!(app.status(), axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(
            app.details().and_then(|v| v.get("reason")),
            Some(&serde_json::Value::String(
                "required_email_subject".to_string()
            ))
        );
        assert_eq!(
            app.details().and_then(|v| v.get("field")),
            Some(&serde_json::Value::String(
                "templates.email_subject".to_string()
            ))
        );
    }
}
