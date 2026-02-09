use axum::http::HeaderMap;
use tower_cookies::Cookies;
use tower_cookies::cookie::{Cookie, SameSite};

use super::{AppError, AppState};
use bastion_storage::auth;

pub(in crate::http) const SESSION_COOKIE_NAME: &str = "bastion_session";
pub(in crate::http) const LOCALE_COOKIE_NAME: &str = "bastion_locale";
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

fn effective_client_ip_from_forwarded(
    headers: &HeaderMap,
    peer_ip: std::net::IpAddr,
    is_trusted_proxy: impl Fn(std::net::IpAddr) -> bool,
) -> std::net::IpAddr {
    if !is_trusted_proxy(peer_ip) {
        return peer_ip;
    }

    let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) else {
        return peer_ip;
    };

    let mut chain = Vec::<std::net::IpAddr>::new();
    for part in xff.split(",") {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let Ok(ip) = part.parse::<std::net::IpAddr>() else {
            return peer_ip;
        };
        chain.push(ip);
    }

    if chain.is_empty() {
        return peer_ip;
    }

    // Use right-to-left trusted-hop stripping and return the first untrusted hop.
    chain.push(peer_ip);
    for ip in chain.into_iter().rev() {
        if is_trusted_proxy(ip) {
            continue;
        }
        return ip;
    }

    peer_ip
}

pub(in crate::http) fn effective_client_ip(
    state: &AppState,
    headers: &HeaderMap,
    peer_ip: std::net::IpAddr,
) -> std::net::IpAddr {
    effective_client_ip_from_forwarded(headers, peer_ip, |ip| {
        state
            .config
            .trusted_proxies
            .iter()
            .any(|net| net.contains(&ip))
    })
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};

    use super::effective_client_ip_from_forwarded;

    #[test]
    fn forwarded_leftmost_spoof_is_ignored_with_trusted_hops() {
        let trusted = |ip: std::net::IpAddr| ip.to_string().starts_with("10.");
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("198.51.100.200, 203.0.113.7"),
        );

        let peer = "10.0.0.10".parse().expect("peer");
        let effective = effective_client_ip_from_forwarded(&headers, peer, trusted);
        assert_eq!(
            effective,
            "203.0.113.7".parse::<std::net::IpAddr>().expect("ip")
        );
    }

    #[test]
    fn forwarded_multi_hop_trusted_proxy_chain_resolves_client() {
        let trusted = |ip: std::net::IpAddr| {
            let v = ip.to_string();
            v.starts_with("10.") || v.starts_with("192.168.")
        };
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("198.51.100.1, 203.0.113.7, 192.168.1.9"),
        );

        let peer = "10.0.0.10".parse().expect("peer");
        let effective = effective_client_ip_from_forwarded(&headers, peer, trusted);
        assert_eq!(
            effective,
            "203.0.113.7".parse::<std::net::IpAddr>().expect("ip")
        );
    }

    #[test]
    fn malformed_forwarded_value_falls_back_to_peer_ip() {
        let trusted = |ip: std::net::IpAddr| ip.to_string().starts_with("10.");
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("bad-ip"));

        let peer = "10.0.0.10".parse().expect("peer");
        let effective = effective_client_ip_from_forwarded(&headers, peer, trusted);
        assert_eq!(effective, peer);
    }

    #[test]
    fn untrusted_peer_ignores_forwarded_chain() {
        let trusted = |ip: std::net::IpAddr| ip.to_string().starts_with("10.");
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.7"));

        let peer = "198.51.100.30".parse().expect("peer");
        let effective = effective_client_ip_from_forwarded(&headers, peer, trusted);
        assert_eq!(effective, peer);
    }
}
