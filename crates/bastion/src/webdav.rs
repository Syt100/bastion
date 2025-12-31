use std::path::Path;
use std::time::Duration;

use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use reqwest::{Method, StatusCode};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use url::Url;

fn redact_url(url: &Url) -> String {
    let mut redacted = url.clone();
    let _ = redacted.set_username("");
    let _ = redacted.set_password(None);
    redacted.set_query(None);
    redacted.set_fragment(None);
    redacted.to_string()
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
    base_url: Url,
    credentials: WebdavCredentials,
}

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

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub async fn ensure_collection(&self, url: &Url) -> Result<(), anyhow::Error> {
        tracing::debug!(url = %redact_url(url), "webdav mkcol");
        let res = self
            .http
            .request(Method::from_bytes(b"MKCOL")?, url.clone())
            .basic_auth(
                self.credentials.username.clone(),
                Some(self.credentials.password.clone()),
            )
            .send()
            .await?;

        match res.status() {
            StatusCode::CREATED | StatusCode::METHOD_NOT_ALLOWED => Ok(()),
            s => Err(anyhow::anyhow!("MKCOL failed: HTTP {s}")),
        }
    }

    pub async fn head_size(&self, url: &Url) -> Result<Option<u64>, anyhow::Error> {
        tracing::debug!(url = %redact_url(url), "webdav head");
        let res = self
            .http
            .head(url.clone())
            .basic_auth(
                self.credentials.username.clone(),
                Some(self.credentials.password.clone()),
            )
            .send()
            .await?;

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
            .http
            .put(url.clone())
            .basic_auth(
                self.credentials.username.clone(),
                Some(self.credentials.password.clone()),
            )
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
        let res = self
            .http
            .get(url.clone())
            .basic_auth(
                self.credentials.username.clone(),
                Some(self.credentials.password.clone()),
            )
            .send()
            .await?;

        match res.status() {
            StatusCode::OK => Ok(res.bytes().await?.to_vec()),
            StatusCode::NOT_FOUND => Err(anyhow::anyhow!("GET failed: HTTP 404")),
            s => Err(anyhow::anyhow!("GET failed: HTTP {s}")),
        }
    }

    pub async fn delete(&self, url: &Url) -> Result<bool, anyhow::Error> {
        tracing::debug!(url = %redact_url(url), "webdav delete");
        let res = self
            .http
            .delete(url.clone())
            .basic_auth(
                self.credentials.username.clone(),
                Some(self.credentials.password.clone()),
            )
            .send()
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
            .http
            .get(url.clone())
            .basic_auth(
                self.credentials.username.clone(),
                Some(self.credentials.password.clone()),
            )
            .send()
            .await?;

        if res.status() != StatusCode::OK {
            anyhow::bail!("GET failed: HTTP {}", res.status());
        }

        if let Some(expected) = expected_size {
            if let Some(len) = res
                .headers()
                .get(CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
            {
                if len != expected {
                    anyhow::bail!("Content-Length mismatch: expected {expected}, got {len}");
                }
            }
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

        if let Some(expected) = expected_size {
            if written != expected {
                anyhow::bail!("download size mismatch: expected {expected}, got {written}");
            }
        }

        let _ = tokio::fs::remove_file(dest).await;
        tokio::fs::rename(&tmp, dest).await?;
        Ok(written)
    }
}
