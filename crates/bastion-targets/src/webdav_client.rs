use std::path::Path;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use percent_encoding::percent_decode_str;
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use reqwest::{Method, StatusCode};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use url::Url;

pub(crate) fn redact_url(url: &Url) -> String {
    let mut redacted = url.clone();
    let _ = redacted.set_username("");
    let _ = redacted.set_password(None);
    redacted.set_query(None);
    redacted.set_fragment(None);
    redacted.to_string()
}

fn parent_collection_url(url: &Url) -> Option<Url> {
    // `Url` always uses absolute paths. We treat `/` as the root boundary and do not attempt to
    // `MKCOL` it (servers typically reject that).
    let trimmed = url.path().trim_end_matches('/');
    let slash = trimmed.rfind('/')?;
    if slash == 0 {
        return None;
    }

    let mut parent = url.clone();
    parent.set_path(&format!("{}/", &trimmed[..slash]));
    Some(parent)
}

#[derive(Debug, Clone)]
pub struct WebdavCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
struct WebdavCredentialsJson {
    username: String,
    password: String,
}

impl WebdavCredentials {
    pub fn from_json(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        let payload: WebdavCredentialsJson = serde_json::from_slice(bytes)?;
        Ok(Self {
            username: payload.username,
            password: payload.password,
        })
    }
}

#[derive(Debug, Clone)]
pub struct WebdavClient {
    http: reqwest::Client,
    #[allow(dead_code)]
    base_url: Url,
    credentials: WebdavCredentials,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebdavPropfindEntry {
    pub href: String,
    pub name: String,
    pub kind: String,
    pub size: Option<u64>,
    pub mtime: Option<i64>,
}

#[derive(Debug)]
pub struct WebdavNotDirectoryError {
    pub href: String,
}

impl std::fmt::Display for WebdavNotDirectoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "webdav path is not a directory: {}", self.href)
    }
}

impl std::error::Error for WebdavNotDirectoryError {}

#[derive(Debug)]
pub struct WebdavHttpError {
    pub status: StatusCode,
    pub message: String,
}

impl std::fmt::Display for WebdavHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "webdav request failed: HTTP {}: {}",
            self.status, self.message
        )
    }
}

impl std::error::Error for WebdavHttpError {}

impl WebdavClient {
    pub fn new(base_url: Url, credentials: WebdavCredentials) -> Result<Self, anyhow::Error> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;
        Ok(Self {
            http,
            base_url,
            credentials,
        })
    }

    fn authed(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.basic_auth(
            self.credentials.username.clone(),
            Some(self.credentials.password.clone()),
        )
    }

    #[allow(dead_code)]
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub async fn ensure_collection(&self, url: &Url) -> Result<(), anyhow::Error> {
        // WebDAV `MKCOL` doesn't create intermediate collections; many servers return HTTP 409
        // Conflict if parent collections are missing. We iteratively create parents first.
        let mut pending = Vec::<Url>::new();
        let mut current = url.clone();
        let mut base_ready = false;

        for _ in 0..=32 {
            tracing::debug!(url = %redact_url(&current), "webdav mkcol");
            let res = self
                .authed(
                    self.http
                        .request(Method::from_bytes(b"MKCOL")?, current.clone()),
                )
                .send()
                .await?;

            match res.status() {
                StatusCode::CREATED | StatusCode::METHOD_NOT_ALLOWED => {
                    base_ready = true;
                    break;
                }
                StatusCode::CONFLICT => {
                    let parent = parent_collection_url(&current).ok_or_else(|| {
                        anyhow::anyhow!("MKCOL failed: HTTP 409 (missing parent collections)")
                    })?;
                    pending.push(current);
                    current = parent;
                    continue;
                }
                s => return Err(anyhow::anyhow!("MKCOL failed: HTTP {s}")),
            }
        }

        if !base_ready {
            anyhow::bail!("webdav ensure_collection recursion limit exceeded");
        }

        while let Some(next) = pending.pop() {
            tracing::debug!(url = %redact_url(&next), "webdav mkcol");
            let res = self
                .authed(
                    self.http
                        .request(Method::from_bytes(b"MKCOL")?, next.clone()),
                )
                .send()
                .await?;

            match res.status() {
                StatusCode::CREATED | StatusCode::METHOD_NOT_ALLOWED => {}
                s => return Err(anyhow::anyhow!("MKCOL failed: HTTP {s}")),
            }
        }

        Ok(())
    }

    pub async fn head_size(&self, url: &Url) -> Result<Option<u64>, anyhow::Error> {
        tracing::debug!(url = %redact_url(url), "webdav head");
        let res = self.authed(self.http.head(url.clone())).send().await?;

        match res.status() {
            StatusCode::OK => {
                let len = res
                    .headers()
                    .get(CONTENT_LENGTH)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .ok_or_else(|| anyhow::anyhow!("missing Content-Length"))?;
                Ok(Some(len))
            }
            StatusCode::NOT_FOUND => Ok(None),
            s => Err(anyhow::anyhow!("HEAD failed: HTTP {s}")),
        }
    }

    pub async fn put_file(&self, url: &Url, path: &Path, size: u64) -> Result<(), anyhow::Error> {
        tracing::debug!(
            url = %redact_url(url),
            path = %path.display(),
            size,
            "webdav put"
        );
        let file = tokio::fs::File::open(path).await?;
        let stream = ReaderStream::new(file);
        let body = reqwest::Body::wrap_stream(stream);

        let res = self
            .authed(self.http.put(url.clone()))
            .header(CONTENT_TYPE, "application/octet-stream")
            .header(CONTENT_LENGTH, size)
            .body(body)
            .send()
            .await?;

        match res.status() {
            StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT => Ok(()),
            s => Err(anyhow::anyhow!("PUT failed: HTTP {s}")),
        }
    }

    pub async fn put_bytes(
        &self,
        url: &Url,
        bytes: Vec<u8>,
        content_type: &'static str,
    ) -> Result<(), anyhow::Error> {
        tracing::debug!(url = %redact_url(url), size = bytes.len(), "webdav put bytes");
        let res = self
            .authed(self.http.put(url.clone()))
            .header(CONTENT_TYPE, content_type)
            .header(CONTENT_LENGTH, bytes.len() as u64)
            .body(bytes)
            .send()
            .await?;

        match res.status() {
            StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT => Ok(()),
            s => Err(anyhow::anyhow!("PUT failed: HTTP {s}")),
        }
    }

    pub async fn put_file_with_retries(
        &self,
        url: &Url,
        path: &Path,
        size: u64,
        max_attempts: u32,
    ) -> Result<(), anyhow::Error> {
        let mut attempt = 1u32;
        let mut backoff = Duration::from_secs(1);
        loop {
            match self.put_file(url, path, size).await {
                Ok(()) => return Ok(()),
                Err(error) if attempt < max_attempts => {
                    tracing::debug!(
                        url = %redact_url(url),
                        attempt,
                        max_attempts,
                        backoff_seconds = backoff.as_secs(),
                        error = %error,
                        "webdav put failed; retrying"
                    );
                    tokio::time::sleep(backoff).await;
                    backoff = std::cmp::min(backoff * 2, Duration::from_secs(30));
                    attempt += 1;
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub async fn get_bytes(&self, url: &Url) -> Result<Vec<u8>, anyhow::Error> {
        tracing::debug!(url = %redact_url(url), "webdav get bytes");
        let res = self.authed(self.http.get(url.clone())).send().await?;

        match res.status() {
            StatusCode::OK => Ok(res.bytes().await?.to_vec()),
            StatusCode::NOT_FOUND => Err(anyhow::anyhow!("GET failed: HTTP 404")),
            s => Err(anyhow::anyhow!("GET failed: HTTP {s}")),
        }
    }

    pub async fn propfind_depth1(
        &self,
        url: &Url,
    ) -> Result<Vec<WebdavPropfindEntry>, anyhow::Error> {
        const BODY: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propfind xmlns:D="DAV:">
  <D:prop>
    <D:resourcetype/>
    <D:getcontentlength/>
    <D:getlastmodified/>
  </D:prop>
</D:propfind>
"#;

        async fn send(
            client: &WebdavClient,
            url: &Url,
            body: &'static str,
        ) -> Result<reqwest::Response, anyhow::Error> {
            let depth_name = reqwest::header::HeaderName::from_static("depth");
            tracing::debug!(url = %redact_url(url), "webdav propfind depth=1");
            Ok(client
                .authed(
                    client
                        .http
                        .request(Method::from_bytes(b"PROPFIND")?, url.clone()),
                )
                .header(depth_name, "1")
                .header(CONTENT_TYPE, "application/xml")
                .body(body)
                .send()
                .await?)
        }

        let res = send(self, url, BODY).await?;
        let status = res.status();
        if status == StatusCode::NOT_FOUND && !url.path().ends_with('/') {
            let mut alt = url.clone();
            alt.set_path(&format!("{}/", alt.path()));
            let alt_res = send(self, &alt, BODY).await?;
            let alt_status = alt_res.status();
            if alt_status == StatusCode::MULTI_STATUS || alt_status == StatusCode::OK {
                let text = alt_res.text().await?;
                let mut entries = parse_propfind_multistatus(&text)?;
                return filter_depth1_self(url, &mut entries);
            }

            let message = alt_res.text().await.unwrap_or_default();
            return Err(WebdavHttpError {
                status: alt_status,
                message,
            }
            .into());
        }

        if status == StatusCode::MULTI_STATUS || status == StatusCode::OK {
            let text = res.text().await?;
            let mut entries = parse_propfind_multistatus(&text)?;
            return filter_depth1_self(url, &mut entries);
        }

        let message = res.text().await.unwrap_or_default();
        Err(WebdavHttpError { status, message }.into())
    }

    pub async fn delete(&self, url: &Url) -> Result<bool, anyhow::Error> {
        tracing::debug!(url = %redact_url(url), "webdav delete");
        let res = self.authed(self.http.delete(url.clone())).send().await?;

        match res.status() {
            StatusCode::NOT_FOUND => Ok(false),
            s if s.is_success() => Ok(true),
            s => Err(anyhow::anyhow!("DELETE failed: HTTP {s}")),
        }
    }

    pub async fn get_to_file(
        &self,
        url: &Url,
        dest: &Path,
        expected_size: Option<u64>,
        max_attempts: u32,
    ) -> Result<u64, anyhow::Error> {
        let mut attempt = 1u32;
        let mut backoff = Duration::from_secs(1);
        loop {
            match self.get_to_file_once(url, dest, expected_size).await {
                Ok(n) => return Ok(n),
                Err(error) if attempt < max_attempts => {
                    tracing::debug!(
                        url = %redact_url(url),
                        dest = %dest.display(),
                        attempt,
                        max_attempts,
                        backoff_seconds = backoff.as_secs(),
                        error = %error,
                        "webdav get failed; retrying"
                    );
                    tokio::time::sleep(backoff).await;
                    backoff = std::cmp::min(backoff * 2, Duration::from_secs(30));
                    attempt += 1;
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
    }

    async fn get_to_file_once(
        &self,
        url: &Url,
        dest: &Path,
        expected_size: Option<u64>,
    ) -> Result<u64, anyhow::Error> {
        tracing::debug!(url = %redact_url(url), dest = %dest.display(), "webdav get to file");
        let res = self.authed(self.http.get(url.clone())).send().await?;

        if res.status() != StatusCode::OK {
            anyhow::bail!("GET failed: HTTP {}", res.status());
        }

        if let Some(expected) = expected_size
            && let Some(len) = res
                .headers()
                .get(CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
            && len != expected
        {
            anyhow::bail!("Content-Length mismatch: expected {expected}, got {len}");
        }

        let file_name = dest
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow::anyhow!("invalid destination file name"))?;
        let tmp = dest.with_file_name(format!("{file_name}.partial"));
        let _ = tokio::fs::remove_file(&tmp).await;

        let mut file = tokio::fs::File::create(&tmp).await?;
        let mut written = 0u64;
        let mut res = res;
        while let Some(chunk) = res.chunk().await? {
            file.write_all(&chunk).await?;
            written = written.saturating_add(chunk.len() as u64);
        }
        file.flush().await?;

        if let Some(expected) = expected_size
            && written != expected
        {
            anyhow::bail!("download size mismatch: expected {expected}, got {written}");
        }

        let _ = tokio::fs::remove_file(dest).await;
        tokio::fs::rename(&tmp, dest).await?;
        Ok(written)
    }
}

fn parse_propfind_multistatus(xml: &str) -> Result<Vec<WebdavPropfindEntry>, anyhow::Error> {
    let doc = roxmltree::Document::parse(xml)?;

    let mut out = Vec::<WebdavPropfindEntry>::new();
    for response in doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "response")
    {
        let Some(mut href) = response
            .children()
            .find(|n| n.is_element() && n.tag_name().name() == "href")
            .and_then(|n| n.text())
            .and_then(decode_href_path)
        else {
            continue;
        };

        let mut kind = "file".to_string();
        let mut size = None::<u64>;
        let mut mtime = None::<i64>;

        for propstat in response
            .children()
            .filter(|n| n.is_element() && n.tag_name().name() == "propstat")
        {
            let status = propstat
                .children()
                .find(|n| n.is_element() && n.tag_name().name() == "status")
                .and_then(|n| n.text())
                .unwrap_or("");
            if !status.contains(" 200 ") {
                continue;
            }

            let Some(prop) = propstat
                .children()
                .find(|n| n.is_element() && n.tag_name().name() == "prop")
            else {
                continue;
            };

            if let Some(resourcetype) = prop
                .children()
                .find(|n| n.is_element() && n.tag_name().name() == "resourcetype")
            {
                let is_dir = resourcetype
                    .children()
                    .any(|n| n.is_element() && n.tag_name().name() == "collection");
                if is_dir {
                    kind = "dir".to_string();
                    if !href.ends_with('/') {
                        href.push('/');
                    }
                }
            }

            if let Some(v) = prop
                .children()
                .find(|n| n.is_element() && n.tag_name().name() == "getcontentlength")
                .and_then(|n| n.text())
                .map(str::trim)
                .filter(|v| !v.is_empty())
            {
                size = v.parse::<u64>().ok();
            }

            if let Some(v) = prop
                .children()
                .find(|n| n.is_element() && n.tag_name().name() == "getlastmodified")
                .and_then(|n| n.text())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                && let Ok(t) = httpdate::parse_http_date(v)
                && let Ok(d) = t.duration_since(UNIX_EPOCH)
            {
                mtime = Some(d.as_secs() as i64);
            }

            break;
        }

        out.push(WebdavPropfindEntry {
            name: basename_from_href(&href),
            href,
            kind,
            size,
            mtime,
        });
    }

    Ok(out)
}

fn decode_href_path(href: &str) -> Option<String> {
    let raw = href.trim();
    if raw.is_empty() {
        return None;
    }

    let mut path_raw = if raw.starts_with("http://") || raw.starts_with("https://") {
        // Absolute URL href (some servers).
        Url::parse(raw).ok()?.path().to_string()
    } else {
        raw.to_string()
    };

    if !path_raw.starts_with('/') {
        path_raw = format!("/{}", path_raw);
    }

    let trailing_slash = path_raw.ends_with('/');
    let parts = path_raw
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| percent_decode_str(s).decode_utf8_lossy().to_string())
        .collect::<Vec<_>>();
    let mut out = format!("/{}", parts.join("/"));
    if trailing_slash && !out.ends_with('/') {
        out.push('/');
    }

    Some(out)
}

fn basename_from_href(href: &str) -> String {
    let trimmed = href.trim().trim_end_matches('/');
    if trimmed == "/" || trimmed.is_empty() {
        return "/".to_string();
    }
    trimmed.rsplit('/').next().unwrap_or(trimmed).to_string()
}

fn filter_depth1_self(
    request_url: &Url,
    entries: &mut Vec<WebdavPropfindEntry>,
) -> Result<Vec<WebdavPropfindEntry>, anyhow::Error> {
    let request_href =
        decode_href_path(request_url.path()).unwrap_or_else(|| request_url.path().to_string());
    let request_href_slash = if request_href.ends_with('/') {
        request_href.clone()
    } else {
        format!("{request_href}/")
    };

    if let Some(self_entry) = entries
        .iter()
        .find(|e| e.href == request_href || e.href == request_href_slash)
        && self_entry.kind != "dir"
    {
        return Err(WebdavNotDirectoryError {
            href: self_entry.href.clone(),
        }
        .into());
    }

    entries.retain(|e| e.href != request_href && e.href != request_href_slash);
    Ok(std::mem::take(entries))
}

#[cfg(test)]
mod tests {
    use super::{
        basename_from_href, decode_href_path, filter_depth1_self, parse_propfind_multistatus,
    };
    use url::Url;

    #[test]
    fn parse_propfind_depth1_extracts_common_properties() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:">
  <d:response>
    <d:href>/backup/</d:href>
    <d:propstat>
      <d:prop>
        <d:resourcetype><d:collection/></d:resourcetype>
        <d:getlastmodified>Mon, 12 Jan 2026 10:00:00 GMT</d:getlastmodified>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/backup/dir/</d:href>
    <d:propstat>
      <d:prop>
        <d:resourcetype><d:collection/></d:resourcetype>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/backup/file.txt</d:href>
    <d:propstat>
      <d:prop>
        <d:resourcetype/>
        <d:getcontentlength>5</d:getcontentlength>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>
"#;

        let entries = parse_propfind_multistatus(xml).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].href, "/backup/");
        assert_eq!(entries[0].name, "backup");
        assert_eq!(entries[0].kind, "dir");
        assert!(entries[0].mtime.is_some());

        assert_eq!(entries[1].href, "/backup/dir/");
        assert_eq!(entries[1].name, "dir");
        assert_eq!(entries[1].kind, "dir");

        assert_eq!(entries[2].href, "/backup/file.txt");
        assert_eq!(entries[2].name, "file.txt");
        assert_eq!(entries[2].kind, "file");
        assert_eq!(entries[2].size, Some(5));
    }

    #[test]
    fn parse_propfind_depth1_skips_non_200_propstat() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:">
  <d:response>
    <d:href>/backup/file.txt</d:href>
    <d:propstat>
      <d:prop><d:getcontentlength>999</d:getcontentlength></d:prop>
      <d:status>HTTP/1.1 404 Not Found</d:status>
    </d:propstat>
    <d:propstat>
      <d:prop><d:getcontentlength>5</d:getcontentlength></d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>
"#;

        let entries = parse_propfind_multistatus(xml).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].size, Some(5));
    }

    #[test]
    fn parse_propfind_depth1_normalizes_dir_slash() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:">
  <d:response>
    <d:href>/backup/dir</d:href>
    <d:propstat>
      <d:prop>
        <d:resourcetype><d:collection/></d:resourcetype>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>
"#;

        let entries = parse_propfind_multistatus(xml).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].href, "/backup/dir/");
        assert_eq!(entries[0].kind, "dir");
    }

    #[test]
    fn decode_href_path_decodes_percent_encoding() {
        assert_eq!(
            decode_href_path("/backup/foo%20bar/").unwrap(),
            "/backup/foo bar/"
        );
    }

    #[test]
    fn basename_from_href_handles_root() {
        assert_eq!(basename_from_href("/"), "/");
        assert_eq!(basename_from_href(""), "/");
        assert_eq!(basename_from_href("/backup/"), "backup");
        assert_eq!(basename_from_href("/backup/file.txt"), "file.txt");
    }

    #[test]
    fn filter_depth1_self_returns_not_directory_error_for_file() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:">
  <d:response>
    <d:href>/backup/file.txt</d:href>
    <d:propstat>
      <d:prop>
        <d:resourcetype/>
        <d:getcontentlength>5</d:getcontentlength>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>
"#;

        let mut entries = parse_propfind_multistatus(xml).unwrap();
        let url = Url::parse("http://example/backup/file.txt").unwrap();
        let err = filter_depth1_self(&url, &mut entries).unwrap_err();
        assert!(err.to_string().contains("not a directory"));
    }
}
