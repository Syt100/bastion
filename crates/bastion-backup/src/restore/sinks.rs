use std::io::Read;
use std::path::{Path, PathBuf};

use super::ConflictPolicy;

pub(super) trait RestoreSink {
    fn prepare(&mut self) -> Result<(), anyhow::Error>;

    fn apply_entry<R: Read>(
        &mut self,
        entry: &mut tar::Entry<R>,
        rel_path: &Path,
    ) -> Result<(), anyhow::Error>;
}

pub(super) struct LocalFsSink {
    base_dir: PathBuf,
    conflict: ConflictPolicy,
}

impl LocalFsSink {
    pub(super) fn new(base_dir: PathBuf, conflict: ConflictPolicy) -> Self {
        Self { base_dir, conflict }
    }
}

impl RestoreSink for LocalFsSink {
    fn prepare(&mut self) -> Result<(), anyhow::Error> {
        std::fs::create_dir_all(&self.base_dir)?;
        Ok(())
    }

    fn apply_entry<R: Read>(
        &mut self,
        entry: &mut tar::Entry<R>,
        rel_path: &Path,
    ) -> Result<(), anyhow::Error> {
        let dest_path = self.base_dir.join(rel_path);

        match self.conflict {
            ConflictPolicy::Overwrite => {
                if dest_path.exists() {
                    remove_existing_path(&dest_path)?;
                }
            }
            ConflictPolicy::Skip => {
                if dest_path.exists() {
                    return Ok(());
                }
            }
            ConflictPolicy::Fail => {
                if dest_path.exists() {
                    anyhow::bail!("restore conflict: {} exists", dest_path.display());
                }
            }
        }

        // `unpack_in` also provides its own path traversal checks. We validate paths separately
        // (engine stage) and treat "skipped" as an error to preserve existing behavior.
        let unpacked = entry.unpack_in(&self.base_dir)?;
        if !unpacked {
            anyhow::bail!("invalid tar entry path: {}", rel_path.display());
        }
        Ok(())
    }
}

fn remove_existing_path(path: &Path) -> Result<(), anyhow::Error> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.is_dir() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
