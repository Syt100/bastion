use std::path::Path;

use url::Url;

use crate::backup::{COMPLETE_NAME, ENTRIES_INDEX_NAME, LocalRunArtifacts, MANIFEST_NAME};
use crate::webdav::{WebdavClient, WebdavCredentials};

pub async fn store_run(
    base_url: &str,
    credentials: WebdavCredentials,
    job_id: &str,
    run_id: &str,
    artifacts: &LocalRunArtifacts,
) -> Result<Url, anyhow::Error> {
    let mut base_url = Url::parse(base_url)?;
    if !base_url.path().ends_with('/') {
        base_url.set_path(&format!("{}/", base_url.path()));
    }

    let client = WebdavClient::new(base_url.clone(), credentials)?;
    let job_url = base_url.join(&format!("{job_id}/"))?;
    client.ensure_collection(&job_url).await?;

    let run_url = job_url.join(&format!("{run_id}/"))?;
    client.ensure_collection(&run_url).await?;

    upload_artifacts(&client, &run_url, artifacts).await?;
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
                continue;
            }
        }
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
                return Ok(());
            }
        }
    }

    client.put_file_with_retries(&url, path, size, 3).await?;
    Ok(())
}
