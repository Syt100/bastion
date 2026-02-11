use std::path::{Path, PathBuf};

use super::super::access::TargetAccess;

pub(in crate::restore) async fn fetch_entries_index(
    access: &TargetAccess,
    staging_dir: &Path,
) -> Result<PathBuf, anyhow::Error> {
    if let Some(run_dir) = access.local_run_dir() {
        return Ok(run_dir.join(crate::backup::ENTRIES_INDEX_NAME));
    }

    let reader = access.reader();
    let dst = staging_dir.join(crate::backup::ENTRIES_INDEX_NAME);
    let expected = reader
        .head_size(crate::backup::ENTRIES_INDEX_NAME.to_string())
        .await?;

    if let Some(size) = expected
        && let Ok(meta) = tokio::fs::metadata(&dst).await
        && meta.len() == size
    {
        return Ok(dst);
    }

    reader
        .get_to_file(
            crate::backup::ENTRIES_INDEX_NAME.to_string(),
            dst.clone(),
            expected,
            3,
        )
        .await?;
    Ok(dst)
}
