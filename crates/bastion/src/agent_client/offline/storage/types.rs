use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(in super::super) enum OfflineRunStatusV1 {
    Running,
    Success,
    Failed,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
pub(in super::super) struct OfflineRunFileV1 {
    pub(in super::super) v: u32,
    pub(in super::super) id: String,
    pub(in super::super) job_id: String,
    pub(in super::super) job_name: String,
    pub(in super::super) status: OfflineRunStatusV1,
    pub(in super::super) started_at: i64,
    pub(in super::super) ended_at: Option<i64>,
    pub(in super::super) summary: Option<serde_json::Value>,
    pub(in super::super) error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(in super::super) struct OfflineRunEventV1 {
    pub(in super::super) seq: i64,
    pub(in super::super) ts: i64,
    pub(in super::super) level: String,
    pub(in super::super) kind: String,
    pub(in super::super) message: String,
    pub(in super::super) fields: Option<serde_json::Value>,
}
