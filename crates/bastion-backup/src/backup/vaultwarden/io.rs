use std::path::Path;

use serde::Serialize;

pub(super) fn write_json(path: &Path, value: &impl Serialize) -> Result<(), anyhow::Error> {
    let bytes = serde_json::to_vec_pretty(value)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_json;

    #[test]
    fn write_json_writes_pretty_json() -> Result<(), anyhow::Error> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("out.json");
        let value = serde_json::json!({
            "a": 1,
            "b": "x",
            "nested": { "c": true },
        });

        write_json(&path, &value)?;

        let bytes = std::fs::read(&path)?;
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains('\n'));

        let parsed: serde_json::Value = serde_json::from_slice(&bytes)?;
        assert_eq!(parsed, value);
        Ok(())
    }
}
