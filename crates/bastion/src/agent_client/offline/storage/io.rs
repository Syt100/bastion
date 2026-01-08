use std::path::Path;

use tokio::io::AsyncWriteExt as _;

use super::types::{OfflineRunEventV1, OfflineRunFileV1};

pub(super) async fn write_offline_run_file_atomic(
    path: &Path,
    doc: OfflineRunFileV1,
) -> Result<(), anyhow::Error> {
    let bytes = serde_json::to_vec_pretty(&doc)?;
    let tmp = path.with_extension("json.partial");
    let _ = tokio::fs::remove_file(&tmp).await;
    tokio::fs::write(&tmp, bytes).await?;
    tokio::fs::rename(&tmp, path).await?;
    Ok(())
}

pub(super) async fn append_offline_event_line(
    path: &Path,
    event: &OfflineRunEventV1,
) -> Result<(), anyhow::Error> {
    let line = serde_json::to_vec(event)?;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;
    file.write_all(&line).await?;
    file.write_all(b"\n").await?;
    Ok(())
}
