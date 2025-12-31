use std::path::Path;

use tracing::{debug, info};
use url::Url;

use crate::backup::{COMPLETE_NAME, ENTRIES_INDEX_NAME, LocalRunArtifacts, MANIFEST_NAME};
use crate::webdav::{WebdavClient, WebdavCredentials};

fn redact_url(url: &Url) -> String {
    let mut redacted = url.clone();
    let _ = redacted.set_username("");
    let _ = redacted.set_password(None);
    redacted.set_query(None);
    redacted.set_fragment(None);
    redacted.to_string()
}

pub async fn store_run(
    base_url: &str,
    credentials: WebdavCredentials,
    job_id: &str,
    run_id: &str,
    artifacts: &LocalRunArtifacts,
) -> Result<Url, anyhow::Error> {
    let parts_count = artifacts.parts.len();
    let parts_bytes: u64 = artifacts.parts.iter().map(|p| p.size).sum();
    let mut base_url = Url::parse(base_url)?;
    if !base_url.path().ends_with('/') {
        base_url.set_path(&format!("{}/", base_url.path()));
    }
    info!(
        job_id = %job_id,
        run_id = %run_id,
        base_url = %redact_url(&base_url),
        parts_count,
        parts_bytes,
        "storing run to webdav"
    );

    let client = WebdavClient::new(credentials)?;
    let job_url = base_url.join(&format!("{job_id}/"))?;
    client.ensure_collection(&job_url).await?;

    let run_url = job_url.join(&format!("{run_id}/"))?;
    client.ensure_collection(&run_url).await?;

    upload_artifacts(&client, &run_url, artifacts).await?;

    info!(
        job_id = %job_id,
        run_id = %run_id,
        run_url = %redact_url(&run_url),
        "stored run to webdav"
    );
    Ok(run_url)
}

async fn upload_artifacts(
    client: &WebdavClient,
    run_url: &Url,
    artifacts: &LocalRunArtifacts,
) -> Result<(), anyhow::Error> {
    for part in &artifacts.parts {
        let url = run_url.join(&part.name)?;
        if let Some(existing) = client.head_size(&url).await? {
            if existing == part.size {
                debug!(url = %redact_url(&url), size = part.size, "skipping existing webdav part");
                continue;
            }
        }
        debug!(url = %redact_url(&url), size = part.size, "uploading webdav part");
        client
            .put_file_with_retries(&url, &part.path, part.size, 3)
            .await?;
    }

    upload_named_file(
        client,
        run_url,
        &artifacts.entries_index_path,
        ENTRIES_INDEX_NAME,
        true,
    )
    .await?;

    upload_named_file(
        client,
        run_url,
        &artifacts.manifest_path,
        MANIFEST_NAME,
        false,
    )
    .await?;

    // Completion marker must be written last.
    upload_named_file(
        client,
        run_url,
        &artifacts.complete_path,
        COMPLETE_NAME,
        false,
    )
    .await?;

    Ok(())
}

async fn upload_named_file(
    client: &WebdavClient,
    run_url: &Url,
    path: &Path,
    name: &str,
    allow_resume: bool,
) -> Result<(), anyhow::Error> {
    let size = tokio::fs::metadata(path).await?.len();
    let url = run_url.join(name)?;

    if allow_resume {
        if let Some(existing) = client.head_size(&url).await? {
            if existing == size {
                debug!(url = %redact_url(&url), size, "skipping existing webdav file");
                return Ok(());
            }
        }
    }

    debug!(url = %redact_url(&url), size, "uploading webdav file");
    client.put_file_with_retries(&url, path, size, 3).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use axum::Router;
    use axum::body::Body;
    use axum::extract::State;
    use axum::http::header::CONTENT_LENGTH;
    use axum::http::{Method, Request, StatusCode};
    use axum::response::IntoResponse;
    use axum::routing::any;
    use bastion_core::manifest::HashAlgorithm;
    use tempfile::TempDir;
    use tokio::net::TcpListener;

    use crate::backup::{LocalArtifact, LocalRunArtifacts};

    #[derive(Clone, Default)]
    struct DavState {
        files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
        put_counts: Arc<Mutex<HashMap<String, u64>>>,
    }

    async fn dav_handler(State(state): State<DavState>, req: Request<Body>) -> impl IntoResponse {
        let method = req.method().clone();
        let path = req.uri().path().to_string();

        match method {
            Method::HEAD => {
                let files = state.files.lock().unwrap();
                if let Some(bytes) = files.get(&path) {
                    let mut resp = StatusCode::OK.into_response();
                    resp.headers_mut()
                        .insert(CONTENT_LENGTH, bytes.len().to_string().parse().unwrap());
                    resp
                } else {
                    StatusCode::NOT_FOUND.into_response()
                }
            }
            Method::PUT => {
                let body = axum::body::to_bytes(req.into_body(), 1024 * 1024)
                    .await
                    .unwrap_or_default();
                {
                    let mut files = state.files.lock().unwrap();
                    files.insert(path.clone(), body.to_vec());
                }
                {
                    let mut counts = state.put_counts.lock().unwrap();
                    *counts.entry(path).or_insert(0) += 1;
                }
                StatusCode::CREATED.into_response()
            }
            _ => {
                // MKCOL and any other methods.
                StatusCode::CREATED.into_response()
            }
        }
    }

    async fn start_dav() -> (String, DavState) {
        let state = DavState::default();
        let app = Router::new()
            .route("/{*path}", any(dav_handler))
            .with_state(state.clone());

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{addr}/backup"), state)
    }

    #[tokio::test]
    async fn store_run_skips_existing_parts_by_size() {
        let temp = TempDir::new().expect("tempdir");
        let stage = temp.path().join("stage");
        std::fs::create_dir_all(&stage).unwrap();

        let part_path = stage.join("payload.part000001");
        std::fs::write(&part_path, b"hello").unwrap();

        let entries_path = stage.join("entries.jsonl.zst");
        std::fs::write(&entries_path, b"entries").unwrap();

        let manifest_path = stage.join("manifest.json");
        std::fs::write(&manifest_path, b"{}").unwrap();

        let complete_path = stage.join("complete.json");
        std::fs::write(&complete_path, b"{}").unwrap();

        let artifacts = LocalRunArtifacts {
            run_dir: stage.clone(),
            parts: vec![LocalArtifact {
                name: "payload.part000001".to_string(),
                path: part_path,
                size: 5,
                hash_alg: HashAlgorithm::Blake3,
                hash: "deadbeef".to_string(),
            }],
            entries_index_path: entries_path,
            entries_count: 1,
            manifest_path,
            complete_path,
        };

        let (base_url, state) = start_dav().await;
        let creds = crate::webdav::WebdavCredentials {
            username: "u".to_string(),
            password: "p".to_string(),
        };

        // First store: uploads all artifacts.
        let _ = super::store_run(&base_url, creds.clone(), "job1", "run1", &artifacts)
            .await
            .unwrap();

        // Second store: should skip payload parts and entries index, but re-upload manifest and complete.
        let _ = super::store_run(&base_url, creds, "job1", "run1", &artifacts)
            .await
            .unwrap();

        let counts = state.put_counts.lock().unwrap();
        let part = "/backup/job1/run1/payload.part000001".to_string();
        let entries = "/backup/job1/run1/entries.jsonl.zst".to_string();
        let manifest = "/backup/job1/run1/manifest.json".to_string();
        let complete = "/backup/job1/run1/complete.json".to_string();

        assert_eq!(counts.get(&part).copied().unwrap_or(0), 1);
        assert_eq!(counts.get(&entries).copied().unwrap_or(0), 1);
        assert_eq!(counts.get(&manifest).copied().unwrap_or(0), 2);
        assert_eq!(counts.get(&complete).copied().unwrap_or(0), 2);
    }
}
