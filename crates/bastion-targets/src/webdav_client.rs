use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::UNIX_EPOCH;

use futures_util::TryStreamExt as _;
use percent_encoding::percent_decode_str;
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE, RETRY_AFTER};
use reqwest::{Method, StatusCode};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
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

#[derive(Debug, Clone)]
pub struct WebdavRequestLimits {
    /// Maximum number of in-flight WebDAV requests.
    pub concurrency: u32,
    /// Max PUT requests per second (best-effort).
    pub put_qps: Option<u32>,
    /// Max HEAD requests per second (best-effort).
    pub head_qps: Option<u32>,
    /// Max MKCOL requests per second (best-effort).
    pub mkcol_qps: Option<u32>,
    /// Optional burst capacity for rate limits (best-effort).
    pub burst: Option<u32>,
}

impl Default for WebdavRequestLimits {
    fn default() -> Self {
        Self {
            concurrency: 4,
            put_qps: None,
            head_qps: None,
            mkcol_qps: None,
            burst: None,
        }
    }
}

impl From<&bastion_core::job_spec::WebdavRequestLimitsV1> for WebdavRequestLimits {
    fn from(value: &bastion_core::job_spec::WebdavRequestLimitsV1) -> Self {
        Self {
            concurrency: value.concurrency,
            put_qps: value.put_qps,
            head_qps: value.head_qps,
            mkcol_qps: value.mkcol_qps,
            burst: value.burst,
        }
    }
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
    limiter: Option<Arc<WebdavRequestLimiter>>,
}

#[derive(Debug, Clone, Copy)]
enum WebdavRequestClass {
    Put,
    Head,
    Mkcol,
    Other,
}

#[derive(Debug)]
struct WebdavRequestLimiter {
    limits: WebdavRequestLimits,
    concurrency: Semaphore,
    put_rate: Option<TokenBucketGate>,
    head_rate: Option<TokenBucketGate>,
    mkcol_rate: Option<TokenBucketGate>,
}

impl WebdavRequestLimiter {
    fn new(limits: WebdavRequestLimits) -> Self {
        let burst = limits.burst.unwrap_or(1).max(1);
        Self {
            concurrency: Semaphore::new(limits.concurrency.max(1) as usize),
            put_rate: limits
                .put_qps
                .and_then(|qps| TokenBucketGate::new(qps, burst)),
            head_rate: limits
                .head_qps
                .and_then(|qps| TokenBucketGate::new(qps, burst)),
            mkcol_rate: limits
                .mkcol_qps
                .and_then(|qps| TokenBucketGate::new(qps, burst)),
            limits,
        }
    }

    fn concurrency_limit(&self) -> usize {
        self.limits.concurrency.max(1) as usize
    }

    async fn rate_limit(&self, class: WebdavRequestClass) {
        let gate = match class {
            WebdavRequestClass::Put => self.put_rate.as_ref(),
            WebdavRequestClass::Head => self.head_rate.as_ref(),
            WebdavRequestClass::Mkcol => self.mkcol_rate.as_ref(),
            WebdavRequestClass::Other => None,
        };
        if let Some(gate) = gate {
            gate.wait().await;
        }
    }

    async fn acquire_concurrency(&self) -> tokio::sync::SemaphorePermit<'_> {
        // `Semaphore::acquire()` only errors if closed; we never close this semaphore.
        self.concurrency.acquire().await.expect("semaphore")
    }
}

#[derive(Debug)]
struct TokenBucketGate {
    rate_per_sec: f64,
    burst: f64,
    state: Mutex<TokenBucketState>,
}

#[derive(Debug)]
struct TokenBucketState {
    tokens: f64,
    last: tokio::time::Instant,
}

impl TokenBucketGate {
    fn new(qps: u32, burst: u32) -> Option<Self> {
        if qps == 0 {
            return None;
        }
        let burst = burst.max(1);
        Some(Self {
            rate_per_sec: qps as f64,
            burst: burst as f64,
            state: Mutex::new(TokenBucketState {
                tokens: burst as f64,
                last: tokio::time::Instant::now(),
            }),
        })
    }

    async fn wait(&self) {
        let delay = {
            let now = tokio::time::Instant::now();
            let mut guard = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let elapsed = now.duration_since(guard.last);
            let refill = elapsed.as_secs_f64() * self.rate_per_sec;
            guard.tokens = (guard.tokens + refill).min(self.burst);
            guard.last = now;

            // Reserve a token for this request (negative tokens represent debt).
            guard.tokens -= 1.0;
            if guard.tokens >= 0.0 {
                None
            } else {
                let deficit = -guard.tokens;
                Some(Duration::from_secs_f64(deficit / self.rate_per_sec))
            }
        };

        if let Some(d) = delay
            && d > Duration::from_millis(0)
        {
            tokio::time::sleep(d).await;
        }
    }
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
    pub retry_after: Option<Duration>,
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
        Self::new_with_limits(base_url, credentials, None)
    }

    pub fn new_with_limits(
        base_url: Url,
        credentials: WebdavCredentials,
        limits: Option<WebdavRequestLimits>,
    ) -> Result<Self, anyhow::Error> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;
        Ok(Self {
            http,
            base_url,
            credentials,
            limiter: limits.map(|l| Arc::new(WebdavRequestLimiter::new(l))),
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

    pub fn request_concurrency_limit(&self) -> usize {
        self.limiter
            .as_ref()
            .map(|lim| lim.concurrency_limit())
            .unwrap_or(1)
    }

    async fn send_limited(
        &self,
        class: WebdavRequestClass,
        builder: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, anyhow::Error> {
        if let Some(limiter) = self.limiter.as_ref() {
            // Don't consume concurrency while we wait for rate slots.
            limiter.rate_limit(class).await;
            let _permit = limiter.acquire_concurrency().await;
            Ok(self.authed(builder).send().await?)
        } else {
            Ok(self.authed(builder).send().await?)
        }
    }

    pub async fn ensure_collection(&self, url: &Url) -> Result<(), anyhow::Error> {
        async fn mkcol(client: &WebdavClient, url: &Url) -> Result<StatusCode, anyhow::Error> {
            let mut attempt = 1u32;
            let max_attempts = 3u32;
            let mut backoff = Duration::from_secs(1);
            loop {
                let res = client
                    .send_limited(
                        WebdavRequestClass::Mkcol,
                        client
                            .http
                            .request(Method::from_bytes(b"MKCOL")?, url.clone()),
                    )
                    .await?;
                let status = res.status();
                match status {
                    StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE
                        if attempt < max_attempts =>
                    {
                        let delay = parse_retry_after(&res)
                            .unwrap_or_else(|| std::cmp::min(backoff, Duration::from_secs(30)));
                        tokio::time::sleep(std::cmp::min(delay, Duration::from_secs(60))).await;
                        backoff = std::cmp::min(backoff * 2, Duration::from_secs(30));
                        attempt += 1;
                        continue;
                    }
                    _ => return Ok(status),
                }
            }
        }

        // WebDAV `MKCOL` doesn't create intermediate collections; many servers return HTTP 409
        // Conflict if parent collections are missing. We iteratively create parents first.
        let mut pending = Vec::<Url>::new();
        let mut current = url.clone();
        let mut base_ready = false;

        for _ in 0..=32 {
            tracing::debug!(url = %redact_url(&current), "webdav mkcol");
            let status = mkcol(self, &current).await?;

            match status {
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
                s => {
                    return Err(WebdavHttpError {
                        status: s,
                        message: "MKCOL failed".to_string(),
                        retry_after: None,
                    }
                    .into());
                }
            }
        }

        if !base_ready {
            anyhow::bail!("webdav ensure_collection recursion limit exceeded");
        }

        while let Some(next) = pending.pop() {
            tracing::debug!(url = %redact_url(&next), "webdav mkcol");
            let status = mkcol(self, &next).await?;

            match status {
                StatusCode::CREATED | StatusCode::METHOD_NOT_ALLOWED => {}
                s => {
                    return Err(WebdavHttpError {
                        status: s,
                        message: "MKCOL failed".to_string(),
                        retry_after: None,
                    }
                    .into());
                }
            }
        }

        Ok(())
    }

    pub async fn head_size(&self, url: &Url) -> Result<Option<u64>, anyhow::Error> {
        tracing::debug!(url = %redact_url(url), "webdav head");
        let mut attempt = 1u32;
        let max_attempts = 3u32;
        let mut backoff = Duration::from_secs(1);
        loop {
            let res = self
                .send_limited(WebdavRequestClass::Head, self.http.head(url.clone()))
                .await?;
            let status = res.status();
            match status {
                StatusCode::OK => {
                    let len = res
                        .headers()
                        .get(CONTENT_LENGTH)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok())
                        .ok_or_else(|| anyhow::anyhow!("missing Content-Length"))?;
                    return Ok(Some(len));
                }
                StatusCode::NOT_FOUND => return Ok(None),
                StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE
                    if attempt < max_attempts =>
                {
                    let delay = parse_retry_after(&res)
                        .unwrap_or_else(|| std::cmp::min(backoff, Duration::from_secs(30)));
                    tokio::time::sleep(std::cmp::min(delay, Duration::from_secs(60))).await;
                    backoff = std::cmp::min(backoff * 2, Duration::from_secs(30));
                    attempt += 1;
                    continue;
                }
                s => {
                    let retry_after = parse_retry_after(&res);
                    let message = res.text().await.unwrap_or_default();
                    return Err(WebdavHttpError {
                        status: s,
                        message,
                        retry_after,
                    }
                    .into());
                }
            }
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

        let req = self
            .http
            .put(url.clone())
            .header(CONTENT_TYPE, "application/octet-stream")
            .header(CONTENT_LENGTH, size)
            .body(body);
        let res = self.send_limited(WebdavRequestClass::Put, req).await?;

        let status = res.status();
        match status {
            StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT => Ok(()),
            s => {
                let retry_after = parse_retry_after(&res);
                let message = res.text().await.unwrap_or_default();
                Err(WebdavHttpError {
                    status: s,
                    message,
                    retry_after,
                }
                .into())
            }
        }
    }

    /// Upload a file while computing a BLAKE3 hash of the uploaded bytes.
    ///
    /// This is used by streaming upload pipelines that want to avoid reading the payload twice
    /// (once for hashing and once for upload).
    pub async fn put_file_hash_blake3(
        &self,
        url: &Url,
        path: &Path,
        size: u64,
    ) -> Result<(String, Option<std::fs::Metadata>), anyhow::Error> {
        use tokio::io::AsyncReadExt as _;

        tracing::debug!(
            url = %redact_url(url),
            path = %path.display(),
            size,
            "webdav put (blake3)"
        );

        // Use a std file so we can `try_clone` for best-effort handle fingerprinting after upload.
        let file = std::fs::File::open(path)?;
        let file_clone = file.try_clone()?;

        let hasher = Arc::new(Mutex::new(blake3::Hasher::new()));
        let hasher_for_stream = hasher.clone();
        let bytes_read = Arc::new(AtomicU64::new(0));
        let bytes_read_for_stream = bytes_read.clone();

        let file = tokio::fs::File::from_std(file);
        let stream = ReaderStream::new(file.take(size)).inspect_ok(move |chunk| {
            let mut guard = hasher_for_stream
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            guard.update(chunk.as_ref());
            bytes_read_for_stream.fetch_add(chunk.len() as u64, Ordering::Relaxed);
        });
        let body = reqwest::Body::wrap_stream(stream);

        let req = self
            .http
            .put(url.clone())
            .header(CONTENT_TYPE, "application/octet-stream")
            .header(CONTENT_LENGTH, size)
            .body(body);
        let res = self.send_limited(WebdavRequestClass::Put, req).await?;

        let status = res.status();
        match status {
            StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT => {}
            s => {
                let retry_after = parse_retry_after(&res);
                let message = res.text().await.unwrap_or_default();
                return Err(WebdavHttpError {
                    status: s,
                    message,
                    retry_after,
                }
                .into());
            }
        }

        let uploaded = bytes_read.load(Ordering::Relaxed);
        if uploaded != size {
            anyhow::bail!("upload size mismatch: expected {size}, got {uploaded}");
        }

        let hash = {
            let mut guard = hasher
                .lock()
                .map_err(|_| anyhow::anyhow!("blake3 hasher mutex poisoned"))?;
            let hasher = std::mem::replace(&mut *guard, blake3::Hasher::new());
            hasher.finalize().to_hex().to_string()
        };

        let after_handle_meta = file_clone.metadata().ok();
        Ok((hash, after_handle_meta))
    }

    pub async fn put_file_hash_blake3_with_retries(
        &self,
        url: &Url,
        path: &Path,
        size: u64,
        max_attempts: u32,
    ) -> Result<(String, Option<std::fs::Metadata>), anyhow::Error> {
        let mut attempt = 1u32;
        let mut backoff = Duration::from_secs(1);
        loop {
            match self.put_file_hash_blake3(url, path, size).await {
                Ok(v) => return Ok(v),
                Err(error) if attempt < max_attempts => {
                    if let Some(http) = error.downcast_ref::<WebdavHttpError>()
                        && (http.status == StatusCode::TOO_MANY_REQUESTS
                            || http.status == StatusCode::SERVICE_UNAVAILABLE)
                        && let Some(delay) = http.retry_after
                    {
                        tokio::time::sleep(std::cmp::min(delay, Duration::from_secs(60))).await;
                        attempt += 1;
                        continue;
                    }
                    tracing::debug!(
                        url = %redact_url(url),
                        attempt,
                        max_attempts,
                        backoff_seconds = backoff.as_secs(),
                        error = %error,
                        "webdav put (blake3) failed; retrying"
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

    pub async fn put_bytes(
        &self,
        url: &Url,
        bytes: Vec<u8>,
        content_type: &'static str,
    ) -> Result<(), anyhow::Error> {
        tracing::debug!(url = %redact_url(url), size = bytes.len(), "webdav put bytes");
        let req = self
            .http
            .put(url.clone())
            .header(CONTENT_TYPE, content_type)
            .header(CONTENT_LENGTH, bytes.len() as u64)
            .body(bytes);
        let res = self.send_limited(WebdavRequestClass::Put, req).await?;

        let status = res.status();
        match status {
            StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT => Ok(()),
            s => {
                let retry_after = parse_retry_after(&res);
                let message = res.text().await.unwrap_or_default();
                Err(WebdavHttpError {
                    status: s,
                    message,
                    retry_after,
                }
                .into())
            }
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
                    if let Some(http) = error.downcast_ref::<WebdavHttpError>()
                        && (http.status == StatusCode::TOO_MANY_REQUESTS
                            || http.status == StatusCode::SERVICE_UNAVAILABLE)
                        && let Some(delay) = http.retry_after
                    {
                        tokio::time::sleep(std::cmp::min(delay, Duration::from_secs(60))).await;
                        attempt += 1;
                        continue;
                    }
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
        let res = self
            .send_limited(WebdavRequestClass::Other, self.http.get(url.clone()))
            .await?;

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
            let req = client
                .http
                .request(Method::from_bytes(b"PROPFIND")?, url.clone())
                .header(depth_name, "1")
                .header(CONTENT_TYPE, "application/xml")
                .body(body);
            client.send_limited(WebdavRequestClass::Other, req).await
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
                retry_after: None,
            }
            .into());
        }

        if status == StatusCode::MULTI_STATUS || status == StatusCode::OK {
            let text = res.text().await?;
            let mut entries = parse_propfind_multistatus(&text)?;
            return filter_depth1_self(url, &mut entries);
        }

        let message = res.text().await.unwrap_or_default();
        Err(WebdavHttpError {
            status,
            message,
            retry_after: None,
        }
        .into())
    }

    pub async fn delete(&self, url: &Url) -> Result<bool, anyhow::Error> {
        tracing::debug!(url = %redact_url(url), "webdav delete");
        let res = self
            .send_limited(WebdavRequestClass::Other, self.http.delete(url.clone()))
            .await?;

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
        let res = self
            .send_limited(WebdavRequestClass::Other, self.http.get(url.clone()))
            .await?;

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

fn parse_retry_after(res: &reqwest::Response) -> Option<Duration> {
    let v = res.headers().get(RETRY_AFTER)?.to_str().ok()?.trim();
    if v.is_empty() {
        return None;
    }

    if let Ok(secs) = v.parse::<u64>() {
        return Some(Duration::from_secs(secs));
    }

    // Retry-After also supports HTTP-date.
    if let Ok(t) = httpdate::parse_http_date(v) {
        if let Ok(d) = t.duration_since(std::time::SystemTime::now()) {
            return Some(d);
        }
        return Some(Duration::from_secs(0));
    }

    None
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
        WebdavClient, WebdavCredentials, WebdavRequestLimits, basename_from_href, decode_href_path,
        filter_depth1_self, parse_propfind_multistatus,
    };
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use axum::Router;
    use axum::body::Body;
    use axum::extract::State;
    use axum::http::{Method, Request, StatusCode};
    use axum::response::IntoResponse;
    use axum::routing::any;
    use tempfile::TempDir;
    use tokio::net::TcpListener;
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

    #[tokio::test]
    async fn request_limiter_caps_concurrency() {
        #[derive(Clone, Default)]
        struct TestState {
            inflight: Arc<AtomicUsize>,
            max_inflight: Arc<AtomicUsize>,
        }

        async fn handler(State(state): State<TestState>, req: Request<Body>) -> impl IntoResponse {
            struct Guard(Arc<AtomicUsize>);
            impl Drop for Guard {
                fn drop(&mut self) {
                    self.0.fetch_sub(1, Ordering::SeqCst);
                }
            }

            let current = state.inflight.fetch_add(1, Ordering::SeqCst) + 1;
            state.max_inflight.fetch_max(current, Ordering::SeqCst);
            let _guard = Guard(state.inflight.clone());

            let _ = axum::body::to_bytes(req.into_body(), 1024 * 1024)
                .await
                .unwrap_or_default();
            tokio::time::sleep(Duration::from_millis(50)).await;
            StatusCode::CREATED
        }

        let state = TestState::default();
        let app = Router::new()
            .route("/{*path}", any(handler))
            .with_state(state.clone());

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let base = Url::parse(&format!("http://{addr}/")).unwrap();
        let client = WebdavClient::new_with_limits(
            base.clone(),
            WebdavCredentials {
                username: "u".to_string(),
                password: "p".to_string(),
            },
            Some(WebdavRequestLimits {
                concurrency: 2,
                put_qps: None,
                head_qps: None,
                mkcol_qps: None,
                burst: None,
            }),
        )
        .unwrap();

        let mut set: tokio::task::JoinSet<Result<(), anyhow::Error>> = tokio::task::JoinSet::new();
        for i in 0..10usize {
            let client = client.clone();
            let url = base.join(&format!("file{i}")).unwrap();
            set.spawn(async move {
                client
                    .put_bytes(&url, vec![b'x'], "application/octet-stream")
                    .await
            });
        }
        while let Some(res) = set.join_next().await {
            res.unwrap().unwrap();
        }

        let peak = state.max_inflight.load(Ordering::SeqCst);
        assert!(peak <= 2, "expected peak concurrency <= 2, got {peak}");
    }

    #[tokio::test]
    async fn request_limiter_enforces_put_qps() {
        #[derive(Clone, Default)]
        struct TestState {
            puts: Arc<AtomicUsize>,
        }

        async fn handler(State(state): State<TestState>, req: Request<Body>) -> impl IntoResponse {
            if req.method() == Method::PUT {
                state.puts.fetch_add(1, Ordering::SeqCst);
            }
            let _ = axum::body::to_bytes(req.into_body(), 1024 * 1024)
                .await
                .unwrap_or_default();
            StatusCode::CREATED
        }

        let state = TestState::default();
        let app = Router::new()
            .route("/{*path}", any(handler))
            .with_state(state.clone());

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let base = Url::parse(&format!("http://{addr}/")).unwrap();
        let client = WebdavClient::new_with_limits(
            base.clone(),
            WebdavCredentials {
                username: "u".to_string(),
                password: "p".to_string(),
            },
            Some(WebdavRequestLimits {
                concurrency: 8,
                put_qps: Some(2),
                head_qps: None,
                mkcol_qps: None,
                burst: Some(1),
            }),
        )
        .unwrap();

        let start = tokio::time::Instant::now();
        for i in 0..5usize {
            let url = base.join(&format!("file{i}")).unwrap();
            client
                .put_bytes(&url, vec![b'x'], "application/octet-stream")
                .await
                .unwrap();
        }
        let elapsed = start.elapsed();

        // With qps=2 and burst=1, 5 sequential requests SHOULD take at least ~2s worth of waits.
        assert!(
            elapsed >= Duration::from_millis(1700),
            "expected elapsed >= 1.7s, got {elapsed:?}"
        );
        assert_eq!(state.puts.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn put_file_with_retries_retries_on_429_with_retry_after() {
        #[derive(Clone, Default)]
        struct TestState {
            puts: Arc<AtomicUsize>,
        }

        async fn handler(State(state): State<TestState>, req: Request<Body>) -> impl IntoResponse {
            let method = req.method().clone();
            let _ = axum::body::to_bytes(req.into_body(), 1024 * 1024)
                .await
                .unwrap_or_default();

            if method != Method::PUT {
                return StatusCode::CREATED.into_response();
            }

            let n = state.puts.fetch_add(1, Ordering::SeqCst) + 1;
            if n == 1 {
                return axum::http::Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .header("Retry-After", "0")
                    .body(Body::from("busy"))
                    .unwrap();
            }

            StatusCode::CREATED.into_response()
        }

        let state = TestState::default();
        let app = Router::new()
            .route("/{*path}", any(handler))
            .with_state(state.clone());

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let base = Url::parse(&format!("http://{addr}/")).unwrap();
        let client = WebdavClient::new(
            base.clone(),
            WebdavCredentials {
                username: "u".to_string(),
                password: "p".to_string(),
            },
        )
        .unwrap();

        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("payload.bin");
        std::fs::write(&path, b"hello").unwrap();

        let url = base.join("payload.bin").unwrap();
        client
            .put_file_with_retries(&url, &path, 5, 3)
            .await
            .unwrap();

        assert_eq!(state.puts.load(Ordering::SeqCst), 2);
    }
}
