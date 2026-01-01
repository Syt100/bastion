use axum::http::HeaderMap;
use tower_cookies::Cookies;
use tower_cookies::cookie::{Cookie, SameSite};

use super::{AppError, AppState};
use crate::auth;

pub(in crate::http) const SESSION_COOKIE_NAME: &str = "bastion_session";
pub(in crate::http) const CSRF_HEADER: &str = "x-csrf-token";

pub(in crate::http) async fn require_session(
    state: &AppState,
    cookies: &Cookies,
) -> Result<auth::SessionRow, AppError> {
    let session_id = cookies
        .get(SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::unauthorized("unauthorized", "Unauthorized"))?;

    let session = auth::get_session(&state.db, &session_id).await?;
    let Some(session) = session else {
        return Err(AppError::unauthorized("unauthorized", "Unauthorized"));
    };
    Ok(session)
}

pub(in crate::http) fn require_csrf(
    headers: &HeaderMap,
    session: &auth::SessionRow,
) -> Result<(), AppError> {
    let csrf = headers
        .get(CSRF_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if csrf != session.csrf_token {
        return Err(AppError::unauthorized("invalid_csrf", "Invalid CSRF token"));
    }
    Ok(())
}

pub(in crate::http) fn set_session_cookie(
    state: &AppState,
    headers: &HeaderMap,
    peer_ip: std::net::IpAddr,
    cookies: &Cookies,
    session_id: &str,
) -> Result<(), anyhow::Error> {
    let is_secure = request_is_https(state, headers, peer_ip);

    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, session_id.to_string());
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    cookie.set_secure(is_secure);

    cookies.add(cookie);
    Ok(())
}

pub(in crate::http) fn request_is_https(
    state: &AppState,
    headers: &HeaderMap,
    peer_ip: std::net::IpAddr,
) -> bool {
    if state.config.insecure_http {
        return false;
    }

    if !is_trusted_proxy(state, peer_ip) {
        return false;
    }

    let Some(proto) = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };

    proto.eq_ignore_ascii_case("https")
}

pub(in crate::http) fn is_trusted_proxy(state: &AppState, peer_ip: std::net::IpAddr) -> bool {
    state
        .config
        .trusted_proxies
        .iter()
        .any(|net| net.contains(&peer_ip))
}

pub(in crate::http) fn effective_client_ip(
    state: &AppState,
    headers: &HeaderMap,
    peer_ip: std::net::IpAddr,
) -> std::net::IpAddr {
    if !is_trusted_proxy(state, peer_ip) {
        return peer_ip;
    }

    let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) else {
        return peer_ip;
    };

    let first = xff.split(',').next().unwrap_or("").trim();
    first.parse().unwrap_or(peer_ip)
}
