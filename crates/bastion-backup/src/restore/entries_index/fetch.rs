use std::path::{Path, PathBuf};

use super::super::access::TargetAccess;

pub(in crate::restore) async fn fetch_entries_index(
    access: &TargetAccess,
    staging_dir: &Path,
) -> Result<PathBuf, anyhow::Error> {
    let dst = staging_dir.join(crate::backup::ENTRIES_INDEX_NAME);
    match access {
        TargetAccess::Webdav { client, run_url } => {
            let url = run_url.join(crate::backup::ENTRIES_INDEX_NAME)?;
            let expected = client.head_size(&url).await?;
            if let Some(size) = expected
                && let Ok(meta) = tokio::fs::metadata(&dst).await
                && meta.len() == size
            {
                return Ok(dst);
            }
            client.get_to_file(&url, &dst, expected, 3).await?;
            Ok(dst)
        }
        TargetAccess::LocalDir { run_dir } => Ok(run_dir.join(crate::backup::ENTRIES_INDEX_NAME)),
    }
}
