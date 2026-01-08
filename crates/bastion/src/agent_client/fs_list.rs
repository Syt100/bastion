use std::path::PathBuf;

use bastion_core::agent_protocol::FsDirEntryV1;

pub(super) fn fs_list_dir_entries(path: &str) -> Result<Vec<FsDirEntryV1>, String> {
    use std::time::UNIX_EPOCH;

    let path = path.trim();
    if path.is_empty() {
        return Err("path is required".to_string());
    }

    let dir = PathBuf::from(path);
    let meta = std::fs::metadata(&dir).map_err(|e| format!("stat failed: {e}"))?;
    if !meta.is_dir() {
        return Err("path is not a directory".to_string());
    }

    let mut out = Vec::<FsDirEntryV1>::new();
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("read_dir failed: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read_dir entry failed: {e}"))?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.trim().is_empty() {
            continue;
        }

        let path = entry.path().to_string_lossy().to_string();
        let ft = entry
            .file_type()
            .map_err(|e| format!("file_type failed: {e}"))?;
        let kind = if ft.is_dir() {
            "dir"
        } else if ft.is_file() {
            "file"
        } else if ft.is_symlink() {
            "symlink"
        } else {
            "other"
        };

        let meta = entry.metadata().ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let mtime = meta.and_then(|m| m.modified().ok()).and_then(|t| {
            t.duration_since(UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs() as i64)
        });

        out.push(FsDirEntryV1 {
            name,
            path,
            kind: kind.to_string(),
            size,
            mtime,
        });
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}
