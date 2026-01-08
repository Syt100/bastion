use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn temp_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("master.key");
    path.with_file_name(format!("{file_name}.tmp"))
}

pub(super) fn write_file_atomic(path: &Path, bytes: &[u8]) -> Result<(), anyhow::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp = temp_path(path);
    fs::write(&tmp, bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))?;
    }

    fs::rename(tmp, path)?;
    Ok(())
}
