use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteTargetType {
    Webdav,
    LocalDir,
}

impl DeleteTargetType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Webdav => "webdav",
            Self::LocalDir => "local_dir",
        }
    }
}

impl std::str::FromStr for DeleteTargetType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "webdav" => Ok(Self::Webdav),
            "local_dir" => Ok(Self::LocalDir),
            _ => Err(anyhow::anyhow!("invalid delete target type")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArtifactDeleteTaskRow {
    pub run_id: String,
    pub job_id: String,
    pub node_id: String,
    pub target_type: DeleteTargetType,
    pub target_snapshot: serde_json::Value,
    pub attempts: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactDeleteTaskDetail {
    pub run_id: String,
    pub job_id: String,
    pub node_id: String,
    pub target_type: String,
    pub target_snapshot: serde_json::Value,
    pub status: String,
    pub attempts: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_attempt_at: Option<i64>,
    pub next_attempt_at: i64,
    pub last_error_kind: Option<String>,
    pub last_error: Option<String>,
    pub ignored_at: Option<i64>,
    pub ignored_by_user_id: Option<i64>,
    pub ignore_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactDeleteTaskSummary {
    pub run_id: String,
    pub status: String,
    pub attempts: i64,
    pub last_attempt_at: Option<i64>,
    pub next_attempt_at: i64,
    pub last_error_kind: Option<String>,
    pub last_error: Option<String>,
    pub ignored_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactDeleteEvent {
    pub run_id: String,
    pub seq: i64,
    pub ts: i64,
    pub level: String,
    pub kind: String,
    pub message: String,
    pub fields: Option<serde_json::Value>,
}

