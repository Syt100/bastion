use std::path::Path;

use tracing::{debug, info};
use url::Url;

use bastion_core::backup_format::{
    COMPLETE_NAME, ENTRIES_INDEX_NAME, LocalArtifact, LocalRunArtifacts, MANIFEST_NAME,
};
use bastion_core::manifest::{ArtifactFormatV1, ManifestV1};

use crate::StoreRunProgress;
use crate::webdav_client::{WebdavClient, WebdavCredentials, redact_url};

/// Upload `payload.part*` files as they are finalized, deleting the local part file after it has
/// been successfully uploaded (or skipped via resumability-by-size).
///
/// This is intended to be used with archive builders that emit part-finalized events so large runs
/// don't require staging all parts locally at once.
pub async fn store_run_parts_rolling(
    base_url: &str,
    credentials: WebdavCredentials,
    job_id: &str,
    run_id: &str,
    mut parts_rx: tokio::sync::mpsc::Receiver<LocalArtifact>,
) -> Result<Url, anyhow::Error> {
    let mut base_url = Url::parse(base_url)?;
    if !base_url.path().ends_with('/') {
        base_url.set_path(&format!("{}/", base_url.path()));
    }

    info!(
        job_id = %job_id,
        run_id = %run_id,
        base_url = %redact_url(&base_url),
        "starting rolling part upload to webdav"
    );

    let client = WebdavClient::new(base_url.clone(), credentials)?;
    let job_url = base_url.join(&format!("{job_id}/"))?;
    client.ensure_collection(&job_url).await?;

    let run_url = job_url.join(&format!("{run_id}/"))?;
    client.ensure_collection(&run_url).await?;

    while let Some(part) = parts_rx.recv().await {
        let url = run_url.join(&part.name)?;
        if let Some(existing) = client.head_size(&url).await?
            && existing == part.size
        {
            debug!(
                url = %redact_url(&url),
                size = part.size,
                "skipping existing webdav part (rolling upload)"
            );
        } else {
            debug!(
                url = %redact_url(&url),
                size = part.size,
                "uploading webdav part (rolling upload)"
            );
            client
                .put_file_with_retries(&url, &part.path, part.size, 3)
                .await?;
        }

        // Best-effort cleanup; failure here should not fail the run, because the target already has
        // the data (or we intentionally skipped due to resumability-by-size).
        let _ = tokio::fs::remove_file(&part.path).await;
    }

    Ok(run_url)
}

pub async fn store_run(
    base_url: &str,
    credentials: WebdavCredentials,
    job_id: &str,
    run_id: &str,
    artifacts: &LocalRunArtifacts,
    on_progress: Option<&(dyn Fn(StoreRunProgress) + Send + Sync)>,
) -> Result<Url, anyhow::Error> {
    let manifest_bytes = std::fs::read(&artifacts.manifest_path)?;
    let manifest: ManifestV1 = serde_json::from_slice(&manifest_bytes)?;
    let artifact_format = manifest.pipeline.format;

    let parts_count = artifacts.parts.len();
    let parts_bytes: u64 = artifacts.parts.iter().map(|p| p.size).sum();

    let entries_size = std::fs::metadata(&artifacts.entries_index_path)?.len();
    let manifest_size = std::fs::metadata(&artifacts.manifest_path)?.len();
    let complete_size = std::fs::metadata(&artifacts.complete_path)?.len();

    // Raw-tree includes many additional payload files under stage_dir/data; those are discovered
    // during upload and added to the running total as we traverse the directory.
    let mut bytes_total: u64 = parts_bytes
        .saturating_add(entries_size)
        .saturating_add(manifest_size)
        .saturating_add(complete_size);
    let mut bytes_done: u64 = 0;
    if let Some(cb) = on_progress {
        cb(StoreRunProgress {
            bytes_done,
            bytes_total: Some(bytes_total),
        });
    }

    let mut base_url = Url::parse(base_url)?;
    if !base_url.path().ends_with('/') {
        base_url.set_path(&format!("{}/", base_url.path()));
    }
    info!(
        job_id = %job_id,
        run_id = %run_id,
        base_url = %redact_url(&base_url),
        artifact_format = ?artifact_format,
        parts_count,
        parts_bytes,
        "storing run to webdav"
    );

    let client = WebdavClient::new(base_url.clone(), credentials)?;
    let job_url = base_url.join(&format!("{job_id}/"))?;
    client.ensure_collection(&job_url).await?;

    let run_url = job_url.join(&format!("{run_id}/"))?;
    client.ensure_collection(&run_url).await?;

    upload_artifacts(
        &client,
        &run_url,
        artifacts,
        artifact_format,
        &mut bytes_done,
        &mut bytes_total,
        on_progress,
    )
    .await?;

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
    artifact_format: ArtifactFormatV1,
    bytes_done: &mut u64,
    bytes_total: &mut u64,
    on_progress: Option<&(dyn Fn(StoreRunProgress) + Send + Sync)>,
) -> Result<(), anyhow::Error> {
    for part in &artifacts.parts {
        let url = run_url.join(&part.name)?;
        if let Some(existing) = client.head_size(&url).await?
            && existing == part.size
        {
            debug!(url = %redact_url(&url), size = part.size, "skipping existing webdav part");
            *bytes_done = bytes_done.saturating_add(part.size);
            if let Some(cb) = on_progress {
                cb(StoreRunProgress {
                    bytes_done: *bytes_done,
                    bytes_total: Some(*bytes_total),
                });
            }
            continue;
        }
        debug!(url = %redact_url(&url), size = part.size, "uploading webdav part");
        client
            .put_file_with_retries(&url, &part.path, part.size, 3)
            .await?;
        *bytes_done = bytes_done.saturating_add(part.size);
        if let Some(cb) = on_progress {
            cb(StoreRunProgress {
                bytes_done: *bytes_done,
                bytes_total: Some(*bytes_total),
            });
        }
    }

    upload_named_file(
        client,
        run_url,
        &artifacts.entries_index_path,
        ENTRIES_INDEX_NAME,
        true,
        bytes_done,
        bytes_total,
        on_progress,
    )
    .await?;

    upload_named_file(
        client,
        run_url,
        &artifacts.manifest_path,
        MANIFEST_NAME,
        false,
        bytes_done,
        bytes_total,
        on_progress,
    )
    .await?;

    if artifact_format == ArtifactFormatV1::RawTreeV1 {
        upload_raw_tree_data_dir(
            client,
            run_url,
            artifacts,
            bytes_done,
            bytes_total,
            on_progress,
        )
        .await?;
    }

    // Completion marker must be written last.
    upload_named_file(
        client,
        run_url,
        &artifacts.complete_path,
        COMPLETE_NAME,
        false,
        bytes_done,
        bytes_total,
        on_progress,
    )
    .await?;

    Ok(())
}

async fn upload_raw_tree_data_dir(
    client: &WebdavClient,
    run_url: &Url,
    artifacts: &LocalRunArtifacts,
    bytes_done: &mut u64,
    bytes_total: &mut u64,
    on_progress: Option<&(dyn Fn(StoreRunProgress) + Send + Sync)>,
) -> Result<(), anyhow::Error> {
    let stage_dir = artifacts
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("invalid manifest path"))?;
    let src = stage_dir.join("data");

    let meta = match tokio::fs::metadata(&src).await {
        Ok(m) => m,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(anyhow::Error::new(error)),
    };
    if !meta.is_dir() {
        anyhow::bail!("raw-tree data path is not a directory: {}", src.display());
    }

    let mut data_url = run_url.clone();
    {
        let mut segs = data_url
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("run_url cannot be a base"))?;
        segs.push("data");
    }
    if !data_url.path().ends_with('/') {
        data_url.set_path(&format!("{}/", data_url.path()));
    }
    client.ensure_collection(&data_url).await?;

    struct StackItem {
        dir_path: std::path::PathBuf,
        dir_url: Url,
    }

    let mut stack = vec![StackItem {
        dir_path: src,
        dir_url: data_url,
    }];

    while let Some(next) = stack.pop() {
        let mut rd = tokio::fs::read_dir(&next.dir_path).await?;
        while let Some(entry) = rd.next_entry().await? {
            let ty = entry.file_type().await?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            let path = entry.path();

            if ty.is_dir() {
                let mut url = next.dir_url.clone();
                {
                    let mut segs = url
                        .path_segments_mut()
                        .map_err(|_| anyhow::anyhow!("run_url cannot be a base"))?;
                    segs.push(&name_str);
                }
                if !url.path().ends_with('/') {
                    url.set_path(&format!("{}/", url.path()));
                }
                client.ensure_collection(&url).await?;
                stack.push(StackItem {
                    dir_path: path,
                    dir_url: url,
                });
                continue;
            }

            if ty.is_file() {
                let size = tokio::fs::metadata(&path).await?.len();
                *bytes_total = bytes_total.saturating_add(size);
                if let Some(cb) = on_progress {
                    cb(StoreRunProgress {
                        bytes_done: *bytes_done,
                        bytes_total: Some(*bytes_total),
                    });
                }
                let mut url = next.dir_url.clone();
                {
                    let mut segs = url
                        .path_segments_mut()
                        .map_err(|_| anyhow::anyhow!("run_url cannot be a base"))?;
                    segs.push(&name_str);
                }

                if let Some(existing) = client.head_size(&url).await?
                    && existing == size
                {
                    debug!(url = %redact_url(&url), size, "skipping existing webdav file");
                    *bytes_done = bytes_done.saturating_add(size);
                    if let Some(cb) = on_progress {
                        cb(StoreRunProgress {
                            bytes_done: *bytes_done,
                            bytes_total: Some(*bytes_total),
                        });
                    }
                    continue;
                }

                debug!(url = %redact_url(&url), size, "uploading webdav file");
                client.put_file_with_retries(&url, &path, size, 3).await?;
                *bytes_done = bytes_done.saturating_add(size);
                if let Some(cb) = on_progress {
                    cb(StoreRunProgress {
                        bytes_done: *bytes_done,
                        bytes_total: Some(*bytes_total),
                    });
                }
                continue;
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn upload_named_file(
    client: &WebdavClient,
    run_url: &Url,
    path: &Path,
    name: &str,
    allow_resume: bool,
    bytes_done: &mut u64,
    bytes_total: &mut u64,
    on_progress: Option<&(dyn Fn(StoreRunProgress) + Send + Sync)>,
) -> Result<(), anyhow::Error> {
    let size = tokio::fs::metadata(path).await?.len();
    let url = run_url.join(name)?;

    if allow_resume
        && let Some(existing) = client.head_size(&url).await?
        && existing == size
    {
        debug!(url = %redact_url(&url), size, "skipping existing webdav file");
        *bytes_done = bytes_done.saturating_add(size);
        if let Some(cb) = on_progress {
            cb(StoreRunProgress {
                bytes_done: *bytes_done,
                bytes_total: Some(*bytes_total),
            });
        }
        return Ok(());
    }

    debug!(url = %redact_url(&url), size, "uploading webdav file");
    client.put_file_with_retries(&url, path, size, 3).await?;
    *bytes_done = bytes_done.saturating_add(size);
    if let Some(cb) = on_progress {
        cb(StoreRunProgress {
            bytes_done: *bytes_done,
            bytes_total: Some(*bytes_total),
        });
    }
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

    use bastion_core::backup_format::{LocalArtifact, LocalRunArtifacts};
    use crate::WebdavCredentials;

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
        std::fs::write(
            &manifest_path,
            serde_json::to_vec(&serde_json::json!({
              "format_version": 1,
              "job_id": "00000000-0000-0000-0000-000000000000",
              "run_id": "00000000-0000-0000-0000-000000000000",
              "started_at": "2025-12-30T12:00:00Z",
              "ended_at": "2025-12-30T12:00:01Z",
              "pipeline": {
                "format": "archive_v1",
                "tar": "pax",
                "compression": "zstd",
                "encryption": "none",
                "split_bytes": 0
              },
              "artifacts": [],
              "entry_index": { "name": "entries.jsonl.zst", "count": 0 }
            }))
            .unwrap(),
        )
        .unwrap();

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
        let creds = crate::WebdavCredentials {
            username: "u".to_string(),
            password: "p".to_string(),
        };

        // First store: uploads all artifacts.
        let _ = super::store_run(&base_url, creds.clone(), "job1", "run1", &artifacts, None)
            .await
            .unwrap();

        // Second store: should skip payload parts and entries index, but re-upload manifest and complete.
        let _ = super::store_run(&base_url, creds, "job1", "run1", &artifacts, None)
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

    #[tokio::test]
    async fn store_run_parts_rolling_uploads_and_deletes_local_parts() {
        let (base_url, state) = start_dav().await;

        let temp = TempDir::new().expect("tempdir");
        let stage = temp.path().join("stage");
        std::fs::create_dir_all(&stage).unwrap();

        let part_path = stage.join("payload.part000001");
        std::fs::write(&part_path, b"hello").unwrap();

        let (tx, rx) = tokio::sync::mpsc::channel::<LocalArtifact>(1);
        tx.send(LocalArtifact {
            name: "payload.part000001".to_string(),
            path: part_path.clone(),
            size: 5,
            hash_alg: HashAlgorithm::Blake3,
            hash: "deadbeef".to_string(),
        })
        .await
        .unwrap();
        drop(tx);

        super::store_run_parts_rolling(
            &base_url,
            WebdavCredentials {
                username: "u".to_string(),
                password: "p".to_string(),
            },
            "job1",
            "run1",
            rx,
        )
        .await
        .unwrap();

        assert!(!part_path.exists());

        let expected_path = "/backup/job1/run1/payload.part000001".to_string();
        let files = state.files.lock().unwrap();
        assert_eq!(files.get(&expected_path).map(|b| b.len()).unwrap_or(0), 5);

        let counts = state.put_counts.lock().unwrap();
        assert_eq!(*counts.get(&expected_path).unwrap_or(&0), 1);
    }

    #[tokio::test]
    async fn store_run_parts_rolling_skips_existing_by_size_and_deletes_local_parts() {
        let (base_url, state) = start_dav().await;

        let existing_path = "/backup/job1/run1/payload.part000001".to_string();
        {
            let mut files = state.files.lock().unwrap();
            files.insert(existing_path.clone(), b"hello".to_vec());
        }

        let temp = TempDir::new().expect("tempdir");
        let stage = temp.path().join("stage");
        std::fs::create_dir_all(&stage).unwrap();

        let part_path = stage.join("payload.part000001");
        std::fs::write(&part_path, b"hello").unwrap();

        let (tx, rx) = tokio::sync::mpsc::channel::<LocalArtifact>(1);
        tx.send(LocalArtifact {
            name: "payload.part000001".to_string(),
            path: part_path.clone(),
            size: 5,
            hash_alg: HashAlgorithm::Blake3,
            hash: "deadbeef".to_string(),
        })
        .await
        .unwrap();
        drop(tx);

        super::store_run_parts_rolling(
            &base_url,
            WebdavCredentials {
                username: "u".to_string(),
                password: "p".to_string(),
            },
            "job1",
            "run1",
            rx,
        )
        .await
        .unwrap();

        assert!(!part_path.exists());

        let counts = state.put_counts.lock().unwrap();
        assert_eq!(*counts.get(&existing_path).unwrap_or(&0), 0);
    }
}
