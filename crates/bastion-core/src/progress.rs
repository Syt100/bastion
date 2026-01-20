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
