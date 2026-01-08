use axum::body::Body;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};

pub(super) async fn ui_fallback(method: Method, uri: Uri, headers: HeaderMap) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return StatusCode::NOT_FOUND.into_response();
    }

    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    if !is_safe_ui_path(path) {
        return StatusCode::NOT_FOUND.into_response();
    }

    #[cfg(feature = "embed-ui")]
    {
        ui_fallback_embed(method, path, &headers)
    }

    #[cfg(not(feature = "embed-ui"))]
    {
        ui_fallback_fs(method, path, &headers).await
    }
}

#[cfg(feature = "embed-ui")]
fn ui_fallback_embed(method: Method, path: &str, headers: &HeaderMap) -> Response {
    let (served_path, bytes) = if let Some(bytes) = load_ui_asset_bytes(path) {
        (path, bytes)
    } else if let Some(bytes) = load_ui_asset_bytes("index.html") {
        ("index.html", bytes)
    } else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let etag = embed_etag(bytes);
    if etag_matches(headers, &etag) {
        return ui_not_modified(&etag, served_path);
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
    with_ui_headers(resp, served_path, &etag)
}

#[cfg(not(feature = "embed-ui"))]
async fn ui_fallback_fs(method: Method, path: &str, headers: &HeaderMap) -> Response {
    use tokio_util::io::ReaderStream;

    let (served_path, file, meta) = if let Some((file, meta)) = try_open_ui_file(path).await {
        (path, file, meta)
    } else if let Some((file, meta)) = try_open_ui_file("index.html").await {
        ("index.html", file, meta)
    } else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let etag = file_etag(&meta);
    if etag_matches(headers, &etag) {
        return ui_not_modified(&etag, served_path);
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
    with_ui_headers(resp, served_path, &etag)
}

#[cfg(feature = "embed-ui")]
fn load_ui_asset_bytes(path: &str) -> Option<&'static [u8]> {
    static UI_DIST: include_dir::Dir<'static> =
        include_dir::include_dir!("$CARGO_MANIFEST_DIR/../../ui/dist");
    let file = UI_DIST.get_file(path)?;
    Some(file.contents())
}

#[cfg(not(feature = "embed-ui"))]
async fn try_open_ui_file(path: &str) -> Option<(tokio::fs::File, std::fs::Metadata)> {
    use std::path::PathBuf;

    let base = std::env::var("BASTION_UI_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("ui/dist"));
    let full = base.join(path);
    let file = tokio::fs::File::open(full).await.ok()?;
    let meta = file.metadata().await.ok()?;
    Some((file, meta))
}

fn is_safe_ui_path(path: &str) -> bool {
    use std::path::{Component, Path};
    Path::new(path)
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
}

fn cache_control_for_ui_path(path: &str) -> &'static str {
    if path == "index.html" {
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

fn ui_not_modified(etag: &str, served_path: &str) -> Response {
    let resp = Response::builder()
        .status(StatusCode::NOT_MODIFIED)
        .header(axum::http::header::ETAG, etag)
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    with_ui_headers(resp, served_path, etag)
}

fn with_ui_headers(mut resp: Response, served_path: &str, etag: &str) -> Response {
    let mime = mime_guess::from_path(served_path).first_or_octet_stream();
    resp.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_str(mime.as_ref())
            .unwrap_or_else(|_| axum::http::HeaderValue::from_static("application/octet-stream")),
    );

    let cache_control = cache_control_for_ui_path(served_path);
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

#[cfg(feature = "embed-ui")]
fn embed_etag(bytes: &[u8]) -> String {
    let hash = fnv1a64(bytes);
    format!("W/\"{:x}-{:016x}\"", bytes.len(), hash)
}

#[cfg(not(feature = "embed-ui"))]
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

#[cfg(feature = "embed-ui")]
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
