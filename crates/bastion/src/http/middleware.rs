use axum::extract::ConnectInfo;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;

use super::shared;
use super::{AppError, AppState};

pub(super) async fn require_secure_middleware(
    state: axum::extract::State<AppState>,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();
    let allow_insecure = matches!(path, "/api/health" | "/api/system" | "/api/setup/status");

    if allow_insecure {
        return next.run(req).await;
    }

    let peer_ip = peer.ip();
    let https = shared::request_is_https(&state, req.headers(), peer_ip);
    let allow = https || state.config.insecure_http || peer_ip.is_loopback();
    if allow {
        return next.run(req).await;
    }

    AppError::bad_request(
        "insecure_not_allowed",
        "HTTPS required (reverse proxy) or start with --insecure-http",
    )
    .into_response()
}
