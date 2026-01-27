use axum::body::Body;
use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use tower_cookies::Cookies;
use tower_cookies::cookie::{Cookie, SameSite, time::Duration};

use super::{AppState, shared};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocsLocale {
    En,
    Zh,
}

impl DocsLocale {
    fn docs_root(self) -> &'static str {
        match self {
            Self::En => "/docs/",
            Self::Zh => "/docs/zh/",
        }
    }

    fn cookie_value(self) -> &'static str {
        match self {
            Self::En => "en-US",
            Self::Zh => "zh-CN",
        }
    }
}

fn parse_lang_param(value: &str) -> Option<DocsLocale> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("zh") || v.eq_ignore_ascii_case("zh-CN") {
        return Some(DocsLocale::Zh);
    }
    if v.eq_ignore_ascii_case("en") || v.eq_ignore_ascii_case("en-US") {
        return Some(DocsLocale::En);
    }
    None
}

fn docs_locale_from_query(uri: &Uri) -> Option<DocsLocale> {
    let query = uri.query()?;
    for (k, v) in url::form_urlencoded::parse(query.as_bytes()) {
        if k == "lang"
            && let Some(locale) = parse_lang_param(&v)
        {
            return Some(locale);
        }
    }
    None
}

fn docs_locale_from_cookie(cookies: &Cookies) -> Option<DocsLocale> {
    let cookie = cookies.get(shared::LOCALE_COOKIE_NAME)?;
    parse_lang_param(cookie.value())
}

fn docs_locale_from_accept_language(headers: &HeaderMap) -> Option<DocsLocale> {
    let value = headers
        .get(axum::http::header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())?;
    // Spec rule: any `zh*` chooses Chinese; otherwise English.
    for part in value.split(',') {
        let lang = part.split(';').next().unwrap_or("").trim();
        if lang.len() < 2 {
            continue;
        }
        if lang.as_bytes()[..2].eq_ignore_ascii_case(b"zh") {
            return Some(DocsLocale::Zh);
        }
    }
    None
}

fn resolve_docs_locale(uri: &Uri, cookies: &Cookies, headers: &HeaderMap) -> (DocsLocale, bool) {
    if let Some(locale) = docs_locale_from_query(uri) {
        return (locale, true);
    }
    if let Some(locale) = docs_locale_from_cookie(cookies) {
        return (locale, false);
    }
    if let Some(locale) = docs_locale_from_accept_language(headers) {
        return (locale, false);
    }
    (DocsLocale::En, false)
}

fn docs_redirect_response(target: DocsLocale) -> Response {
    Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header(axum::http::header::LOCATION, target.docs_root())
        // Avoid shared cache issues (reverse proxies / CDN) across different users.
        .header(axum::http::header::CACHE_CONTROL, "no-store")
        .header(axum::http::header::VARY, "Accept-Language, Cookie")
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn docs_locale_from_path(path: &str) -> DocsLocale {
    if path == "/docs/zh" || path.starts_with("/docs/zh/") {
        DocsLocale::Zh
    } else {
        DocsLocale::En
    }
}

fn maybe_set_locale_cookie(
    state: &AppState,
    headers: &HeaderMap,
    peer_ip: std::net::IpAddr,
    cookies: &Cookies,
    locale: DocsLocale,
) {
    let is_secure = shared::request_is_https(state, headers, peer_ip);

    let mut cookie = Cookie::new(
        shared::LOCALE_COOKIE_NAME,
        locale.cookie_value().to_string(),
    );
    cookie.set_http_only(false);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    cookie.set_secure(is_secure);
    cookie.set_max_age(Duration::days(365));

    cookies.add(cookie);
}

pub(super) async fn docs_redirect(uri: Uri, headers: HeaderMap, cookies: Cookies) -> Response {
    // `/docs` always redirects so relative links resolve correctly under a locale root.
    let (locale, _from_query) = resolve_docs_locale(&uri, &cookies, &headers);
    docs_redirect_response(locale)
}

pub(super) async fn docs_fallback(
    State(state): State<AppState>,
    cookies: Cookies,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    method: Method,
    uri: Uri,
) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return StatusCode::NOT_FOUND.into_response();
    }

    // Locale-aware entrypoint: `/docs/` is the English docs root, but should redirect to Chinese
    // when a better locale is detected.
    if uri.path() == "/docs/" {
        let (locale, from_query) = resolve_docs_locale(&uri, &cookies, &headers);
        if from_query || locale == DocsLocale::Zh {
            return docs_redirect_response(locale);
        }
    }

    let request_locale = docs_locale_from_path(uri.path());

    // Strip the "/docs" prefix (router matches /docs/*).
    let mut path = uri.path().strip_prefix("/docs").unwrap_or(uri.path());
    path = path.trim_start_matches('/');

    let mut candidates = Vec::<String>::new();
    candidates.push(normalize_docs_candidate(path));

    // Convenience: allow "/docs/foo" (no extension) to load "foo.html" when present.
    if !path.is_empty() && !path.contains('.') && !path.ends_with('/') {
        candidates.push(format!("{path}.html"));
    }

    // Fall back to a static 404 page (generated by VitePress).
    candidates.push("404.html".to_string());

    for (idx, candidate) in candidates.iter().enumerate() {
        let served_path = candidate.as_str();
        let is_fallback_404 = idx + 1 == candidates.len() && served_path == "404.html";
        if !is_safe_docs_path(served_path) {
            continue;
        }

        #[cfg(feature = "embed-docs")]
        {
            if let Some(bytes) = load_docs_asset_bytes(served_path) {
                let mut resp = serve_embed(method.clone(), served_path, bytes, &headers);
                if served_path.ends_with(".html") {
                    maybe_set_locale_cookie(&state, &headers, peer.ip(), &cookies, request_locale);
                }
                if is_fallback_404 && resp.status() == StatusCode::OK {
                    *resp.status_mut() = StatusCode::NOT_FOUND;
                }
                return resp;
            }
        }

        #[cfg(not(feature = "embed-docs"))]
        {
            if let Some(resp) = serve_fs(method.clone(), served_path, &headers).await {
                let mut resp = resp;
                if served_path.ends_with(".html") {
                    maybe_set_locale_cookie(&state, &headers, peer.ip(), &cookies, request_locale);
                }
                if is_fallback_404 && resp.status() == StatusCode::OK {
                    *resp.status_mut() = StatusCode::NOT_FOUND;
                }
                return resp;
            }
        }

        // Only allow the 404 fallback at the end.
        if idx + 1 == candidates.len() {
            break;
        }
    }

    StatusCode::NOT_FOUND.into_response()
}

fn normalize_docs_candidate(path: &str) -> String {
    let path = path.trim();
    if path.is_empty() {
        return "index.html".to_string();
    }
    if path.ends_with('/') {
        return format!("{path}index.html");
    }
    path.to_string()
}

fn is_safe_docs_path(path: &str) -> bool {
    use std::path::{Component, Path};
    Path::new(path)
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
}

fn cache_control_for_docs_path(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "no-cache"
    } else if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    }
}

fn etag_matches(headers: &HeaderMap, etag: &str) -> bool {
    let Some(value) = headers.get(axum::http::header::IF_NONE_MATCH) else {
        return false;
    };
    let Ok(value) = value.to_str() else {
        return false;
    };
    value.split(',').any(|v| v.trim() == etag)
}

fn with_docs_headers(mut resp: Response, served_path: &str, etag: &str) -> Response {
    let mime = mime_guess::from_path(served_path).first_or_octet_stream();
    resp.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_str(mime.as_ref())
            .unwrap_or_else(|_| axum::http::HeaderValue::from_static("application/octet-stream")),
    );

    let cache_control = cache_control_for_docs_path(served_path);
    let _ = resp.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static(cache_control),
    );
    let _ = resp.headers_mut().insert(
        axum::http::header::ETAG,
        axum::http::HeaderValue::from_str(etag)
            .unwrap_or_else(|_| axum::http::HeaderValue::from_static("")),
    );
    resp
}

fn not_modified(etag: &str, served_path: &str) -> Response {
    let resp = Response::builder()
        .status(StatusCode::NOT_MODIFIED)
        .header(axum::http::header::ETAG, etag)
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    with_docs_headers(resp, served_path, etag)
}

#[cfg(feature = "embed-docs")]
fn serve_embed(
    method: Method,
    served_path: &str,
    bytes: &'static [u8],
    headers: &HeaderMap,
) -> Response {
    let etag = embed_etag(bytes);
    if etag_matches(headers, &etag) {
        return not_modified(&etag, served_path);
    }

    let body = if method == Method::HEAD {
        Body::empty()
    } else {
        Body::from(bytes.to_vec())
    };

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::ETAG, &etag)
        .body(body)
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    with_docs_headers(resp, served_path, &etag)
}

#[cfg(feature = "embed-docs")]
fn load_docs_asset_bytes(path: &str) -> Option<&'static [u8]> {
    static DOCS_DIST: include_dir::Dir<'static> =
        include_dir::include_dir!("$CARGO_MANIFEST_DIR/../../docs/.vitepress/dist");
    let file = DOCS_DIST.get_file(path)?;
    Some(file.contents())
}

#[cfg(feature = "embed-docs")]
fn embed_etag(bytes: &[u8]) -> String {
    // Same format as UI etags; content-addressed by length + stable hash.
    let hash = fnv1a64(bytes);
    format!("W/\"{:x}-{:016x}\"", bytes.len(), hash)
}

#[cfg(feature = "embed-docs")]
fn fnv1a64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(not(feature = "embed-docs"))]
async fn serve_fs(method: Method, served_path: &str, headers: &HeaderMap) -> Option<Response> {
    use tokio_util::io::ReaderStream;

    let (file, meta) = try_open_docs_file(served_path).await?;

    let etag = file_etag(&meta);
    if etag_matches(headers, &etag) {
        return Some(not_modified(&etag, served_path));
    }

    let len = meta.len();
    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::ETAG, &etag);
    if len > 0 {
        resp = resp.header(axum::http::header::CONTENT_LENGTH, len.to_string());
    }

    let body = if method == Method::HEAD {
        Body::empty()
    } else {
        Body::from_stream(ReaderStream::new(file))
    };

    let resp = resp
        .body(body)
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    Some(with_docs_headers(resp, served_path, &etag))
}

#[cfg(all(test, not(feature = "embed-docs")))]
static TEST_DOCS_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

#[cfg(not(feature = "embed-docs"))]
async fn try_open_docs_file(path: &str) -> Option<(tokio::fs::File, std::fs::Metadata)> {
    use std::path::PathBuf;

    #[cfg(all(test, not(feature = "embed-docs")))]
    if let Some(base) = TEST_DOCS_DIR.get() {
        let full = base.join(path);
        let file = tokio::fs::File::open(full).await.ok()?;
        let meta = file.metadata().await.ok()?;
        return Some((file, meta));
    }

    let base = std::env::var("BASTION_DOCS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("docs/.vitepress/dist"));
    let full = base.join(path);
    let file = tokio::fs::File::open(full).await.ok()?;
    let meta = file.metadata().await.ok()?;
    Some((file, meta))
}

#[cfg(not(feature = "embed-docs"))]
fn file_etag(meta: &std::fs::Metadata) -> String {
    use std::time::UNIX_EPOCH;

    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("W/\"{:x}-{:x}\"", meta.len(), modified)
}

#[cfg(all(test, not(feature = "embed-docs")))]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use axum::http::StatusCode;
    use tempfile::TempDir;

    use crate::http::AppState;

    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
        GUARD.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    fn write_file(dir: &std::path::Path, rel: &str, content: &str) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("mkdirs");
        }
        std::fs::write(path, content).expect("write file");
    }

    async fn start_test_server(
        app: axum::Router,
    ) -> (tokio::task::JoinHandle<()>, std::net::SocketAddr) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            .expect("serve");
        });
        (server, addr)
    }

    fn test_state(tmp: &TempDir, db: sqlx::SqlitePool) -> AppState {
        use std::sync::Arc;

        use bastion_config::Config;
        use bastion_engine::agent_manager::AgentManager;
        use bastion_storage::secrets::SecretsCrypto;

        let config = Arc::new(Config {
            bind: "127.0.0.1:0".parse().expect("bind"),
            data_dir: tmp.path().to_path_buf(),
            insecure_http: true,
            debug_errors: false,
            hub_timezone: "UTC".to_string(),
            run_retention_days: 180,
            incomplete_cleanup_days: 7,
            trusted_proxies: vec![
                "127.0.0.1/32".parse().expect("proxy"),
                "::1/128".parse().expect("proxy"),
            ],
        });

        let secrets = Arc::new(SecretsCrypto::load_or_create(&config.data_dir).expect("secrets"));

        AppState {
            config,
            db,
            secrets,
            agent_manager: AgentManager::default(),
            run_queue_notify: Arc::new(tokio::sync::Notify::new()),
            incomplete_cleanup_notify: Arc::new(tokio::sync::Notify::new()),
            artifact_delete_notify: Arc::new(tokio::sync::Notify::new()),
            jobs_notify: Arc::new(tokio::sync::Notify::new()),
            notifications_notify: Arc::new(tokio::sync::Notify::new()),
            bulk_ops_notify: Arc::new(tokio::sync::Notify::new()),
            run_events_bus: Arc::new(bastion_engine::run_events_bus::RunEventsBus::new()),
            hub_runtime_config: Default::default(),
        }
    }

    fn base_url(addr: std::net::SocketAddr) -> String {
        format!("http://{addr}")
    }

    fn reset_test_docs_dir() -> std::path::PathBuf {
        let base = super::TEST_DOCS_DIR
            .get_or_init(|| TempDir::new().expect("docs dir").keep())
            .clone();

        // Ensure a clean slate per test. Tests hold a global mutex via `env_guard()` to avoid
        // concurrent mutation of this shared directory.
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("mkdir docs dir");
        base
    }

    fn assert_locale_redirect_headers(resp: &reqwest::Response) {
        assert_eq!(
            resp.headers()
                .get(axum::http::header::CACHE_CONTROL)
                .and_then(|v| v.to_str().ok()),
            Some("no-store")
        );
        assert_eq!(
            resp.headers()
                .get(axum::http::header::VARY)
                .and_then(|v| v.to_str().ok()),
            Some("Accept-Language, Cookie")
        );
    }

    #[tokio::test]
    async fn docs_are_public_redirect_and_path_safety_work_in_fs_mode() {
        let _guard = env_guard();

        let tmp = TempDir::new().expect("tempdir");
        let db = bastion_storage::db::init(tmp.path())
            .await
            .expect("db init");

        let base = reset_test_docs_dir();
        write_file(&base, "index.html", "<a href=\"./agents.html\">Agents</a>");
        write_file(&base, "agents.html", "agents");
        write_file(&base, "404.html", "not found");

        let app = super::super::router(test_state(&tmp, db));
        let (server, addr) = start_test_server(app).await;

        let client = reqwest::Client::builder()
            // Don't auto-follow so we can assert redirect.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("client");

        let resp = client
            .get(format!("{}/docs", base_url(addr)))
            .send()
            .await
            .expect("request");
        assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            resp.headers()
                .get(axum::http::header::LOCATION)
                .and_then(|v| v.to_str().ok()),
            Some("/docs/")
        );
        assert_locale_redirect_headers(&resp);

        let resp = client
            .get(format!("{}/docs/", base_url(addr)))
            .send()
            .await
            .expect("request");
        assert_eq!(resp.status(), StatusCode::OK);

        // Path traversal is rejected.
        let resp = client
            .get(format!("{}/docs/..%2Fsecret", base_url(addr)))
            .send()
            .await
            .expect("request");
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        server.abort();
    }

    #[tokio::test]
    async fn docs_entrypoint_root_redirects_to_zh_by_accept_language() {
        let _guard = env_guard();

        let tmp = TempDir::new().expect("tempdir");
        let db = bastion_storage::db::init(tmp.path())
            .await
            .expect("db init");

        let base = reset_test_docs_dir();
        write_file(&base, "index.html", "en index");
        write_file(&base, "zh/index.html", "zh index");
        write_file(&base, "404.html", "not found");

        let app = super::super::router(test_state(&tmp, db));
        let (server, addr) = start_test_server(app).await;

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("client");

        let resp = client
            .get(format!("{}/docs/", base_url(addr)))
            .header(
                axum::http::header::ACCEPT_LANGUAGE,
                "zh-CN,zh;q=0.9,en;q=0.8",
            )
            .send()
            .await
            .expect("request");
        assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            resp.headers()
                .get(axum::http::header::LOCATION)
                .and_then(|v| v.to_str().ok()),
            Some("/docs/zh/")
        );
        assert_locale_redirect_headers(&resp);

        server.abort();
    }

    #[tokio::test]
    async fn docs_entrypoint_root_stays_english_by_default_and_sets_cookie() {
        let _guard = env_guard();

        let tmp = TempDir::new().expect("tempdir");
        let db = bastion_storage::db::init(tmp.path())
            .await
            .expect("db init");

        let base = reset_test_docs_dir();
        write_file(&base, "index.html", "en index");
        write_file(&base, "zh/index.html", "zh index");
        write_file(&base, "404.html", "not found");

        let app = super::super::router(test_state(&tmp, db));
        let (server, addr) = start_test_server(app).await;

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("client");

        let resp = client
            .get(format!("{}/docs/", base_url(addr)))
            .send()
            .await
            .expect("request");
        assert_eq!(resp.status(), StatusCode::OK);

        let set_cookie = resp
            .headers()
            .get_all(axum::http::header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            set_cookie.contains("bastion_locale=en-US"),
            "expected bastion_locale=en-US in Set-Cookie, got:\n{set_cookie}"
        );

        server.abort();
    }

    #[tokio::test]
    async fn docs_entrypoint_root_query_param_overrides_cookie() {
        let _guard = env_guard();

        let tmp = TempDir::new().expect("tempdir");
        let db = bastion_storage::db::init(tmp.path())
            .await
            .expect("db init");

        let base = reset_test_docs_dir();
        write_file(&base, "index.html", "en index");
        write_file(&base, "zh/index.html", "zh index");
        write_file(&base, "404.html", "not found");

        let app = super::super::router(test_state(&tmp, db));
        let (server, addr) = start_test_server(app).await;

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("client");

        let resp = client
            .get(format!("{}/docs/?lang=en", base_url(addr)))
            .header(axum::http::header::COOKIE, "bastion_locale=zh-CN")
            .send()
            .await
            .expect("request");
        assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            resp.headers()
                .get(axum::http::header::LOCATION)
                .and_then(|v| v.to_str().ok()),
            Some("/docs/")
        );
        assert_locale_redirect_headers(&resp);

        server.abort();
    }
}
