use std::path::Path;

use serde::Serialize;

pub(super) fn write_json_pretty_atomic(
    path: &Path,
    value: &impl Serialize,
) -> Result<(), anyhow::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let bytes = serde_json::to_vec_pretty(value)?;
    let tmp = path.with_extension("json.partial");
    let _ = std::fs::remove_file(&tmp);
    std::fs::write(&tmp, bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }

    std::fs::rename(&tmp, path)?;
    Ok(())
}
