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

#[cfg(test)]
mod tests {
    use super::write_json_pretty_atomic;

    #[test]
    fn write_json_pretty_atomic_writes_file_and_cleans_up_tmp() {
        let tmp = tempfile::tempdir().unwrap();

        let path = tmp.path().join("agent").join("managed").join("config.json");
        let value = serde_json::json!({
            "a": 1,
            "b": "x",
            "nested": { "c": true },
        });

        write_json_pretty_atomic(&path, &value).unwrap();

        assert!(path.exists());
        assert!(path.parent().unwrap().exists());

        let bytes = std::fs::read(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed, value);

        let tmp_path = path.with_extension("json.partial");
        assert!(!tmp_path.exists());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }
}
