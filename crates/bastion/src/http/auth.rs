use axum::Json;
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, StatusCode};
use serde::Serialize;
use tower_cookies::Cookies;
use tower_cookies::cookie::Cookie;

use super::shared;
use super::shared::require_csrf;
use super::{AppError, AppState};
use crate::auth;

#[derive(Debug, Serialize)]
pub(super) struct SetupStatusResponse {
    needs_setup: bool,
}

pub(super) async fn setup_status(
    state: axum::extract::State<AppState>,
) -> Result<Json<SetupStatusResponse>, AppError> {
    let count = auth::users_count(&state.db).await?;
    Ok(Json(SetupStatusResponse {
        needs_setup: count == 0,
    }))
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct SetupInitializeRequest {
    username: String,
    password: String,
}

pub(super) async fn setup_initialize(
    state: axum::extract::State<AppState>,
    Json(req): Json<SetupInitializeRequest>,
) -> Result<StatusCode, AppError> {
    let count = auth::users_count(&state.db).await?;
    if count != 0 {
        return Err(AppError::conflict(
            "already_initialized",
            "Setup is already complete",
        ));
    }

    auth::create_user(&state.db, &req.username, &req.password).await?;
    tracing::info!("setup initialized");
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub(super) struct LoginResponse {
    csrf_token: String,
}

pub(super) async fn login(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let client_ip = shared::effective_client_ip(&state, &headers, peer.ip());
    let client_ip_str = client_ip.to_string();

    if let Some(retry_after) =
        auth::login_throttle_retry_after_seconds(&state.db, &client_ip_str, now).await?
    {
        tracing::warn!(client_ip = %client_ip, retry_after, "login rate limited");
        return Err(AppError::too_many_requests(
            "rate_limited",
            format!("Too many login attempts. Retry after {retry_after}s."),
        ));
    }

    let Some(user) = auth::find_user_by_username(&state.db, &req.username).await? else {
        let _ = auth::record_login_failure(&state.db, &client_ip_str, now).await;
        tracing::debug!(client_ip = %client_ip, "login failed: user not found");
        return Err(AppError::unauthorized(
            "invalid_credentials",
            "Invalid credentials",
        ));
    };

    if !auth::verify_password(&user.password_hash, &req.password)? {
        let _ = auth::record_login_failure(&state.db, &client_ip_str, now).await;
        tracing::debug!(client_ip = %client_ip, user_id = user.id, "login failed: bad password");
        return Err(AppError::unauthorized(
            "invalid_credentials",
            "Invalid credentials",
        ));
    }

    let _ = auth::clear_login_throttle(&state.db, &client_ip_str).await;

    let session = auth::create_session(&state.db, user.id).await?;
    shared::set_session_cookie(&state, &headers, peer.ip(), &cookies, &session.id)?;

    tracing::info!(client_ip = %client_ip, user_id = user.id, "login succeeded");
    Ok(Json(LoginResponse {
        csrf_token: session.csrf_token,
    }))
}

pub(super) async fn logout(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let session_id = cookies
        .get(shared::SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string());

    let Some(session_id) = session_id else {
        return Ok(StatusCode::NO_CONTENT);
    };

    let session = auth::get_session(&state.db, &session_id).await?;
    let Some(session) = session else {
        let mut cookie = Cookie::new(shared::SESSION_COOKIE_NAME, "");
        cookie.set_path("/");
        cookies.remove(cookie);
        return Ok(StatusCode::NO_CONTENT);
    };

    require_csrf(&headers, &session)?;

    auth::delete_session(&state.db, &session_id).await?;
    let mut cookie = Cookie::new(shared::SESSION_COOKIE_NAME, "");
    cookie.set_path("/");
    cookies.remove(cookie);
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
pub(super) struct SessionResponse {
    authenticated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    csrf_token: Option<String>,
}

pub(super) async fn session(
    state: axum::extract::State<AppState>,
    cookies: Cookies,
) -> Result<Json<SessionResponse>, AppError> {
    let session_id = cookies
        .get(shared::SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string());

    let Some(session_id) = session_id else {
        return Ok(Json(SessionResponse {
            authenticated: false,
            csrf_token: None,
        }));
    };

    let session = auth::get_session(&state.db, &session_id).await?;
    let Some(session) = session else {
        return Ok(Json(SessionResponse {
            authenticated: false,
            csrf_token: None,
        }));
    };

    Ok(Json(SessionResponse {
        authenticated: true,
        csrf_token: Some(session.csrf_token),
    }))
}
