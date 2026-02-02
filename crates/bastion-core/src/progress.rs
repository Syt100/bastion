use serde::{Deserialize, Serialize};

pub const PROGRESS_SNAPSHOT_EVENT_KIND_V1: &str = "progress_snapshot";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressKindV1 {
    Backup,
    Restore,
    Verify,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressUnitsV1 {
    pub files: u64,
    pub dirs: u64,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressSnapshotV1 {
    pub v: u32,
    pub kind: ProgressKindV1,
    pub stage: String,
    pub ts: i64,
    pub done: ProgressUnitsV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<ProgressUnitsV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_bps: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eta_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_kind_serializes_as_snake_case() -> Result<(), anyhow::Error> {
        assert_eq!(
            serde_json::to_value(ProgressKindV1::Backup)?,
            serde_json::Value::String("backup".to_string())
        );
        assert_eq!(
            serde_json::to_value(ProgressKindV1::Restore)?,
            serde_json::Value::String("restore".to_string())
        );
        Ok(())
    }

    #[test]
    fn progress_snapshot_omits_none_fields() -> Result<(), anyhow::Error> {
        let snap = ProgressSnapshotV1 {
            v: 1,
            kind: ProgressKindV1::Backup,
            stage: "packaging".to_string(),
            ts: 1,
            done: ProgressUnitsV1::default(),
            total: None,
            rate_bps: None,
            eta_seconds: None,
            detail: None,
        };

        let v = serde_json::to_value(&snap)?;
        let obj = v.as_object().expect("object");
        assert!(!obj.contains_key("total"));
        assert!(!obj.contains_key("rate_bps"));
        assert!(!obj.contains_key("eta_seconds"));
        assert!(!obj.contains_key("detail"));
        Ok(())
    }
}
