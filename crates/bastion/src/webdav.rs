use std::path::Path;
use std::time::Duration;

use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use reqwest::{Method, StatusCode};
use serde::Deserialize;
use tokio_util::io::ReaderStream;
use url::Url;

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
                    tokio::time::sleep(backoff).await;
                    backoff = std::cmp::min(backoff * 2, Duration::from_secs(30));
                    attempt += 1;
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
    }
}
