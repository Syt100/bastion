use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Queued,
    Running,
    Success,
    Failed,
    Rejected,
}

impl RunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Rejected => "rejected",
        }
    }
}

impl std::str::FromStr for RunStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "success" => Ok(Self::Success),
            "failed" => Ok(Self::Failed),
            "rejected" => Ok(Self::Rejected),
            _ => Err(anyhow::anyhow!("invalid run status")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Run {
    pub id: String,
    pub job_id: String,
    pub status: RunStatus,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub progress: Option<serde_json::Value>,
    pub summary: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunEvent {
    pub run_id: String,
    pub seq: i64,
    pub ts: i64,
    pub level: String,
    pub kind: String,
    pub message: String,
    pub fields: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct IncompleteCleanupRun {
    pub id: String,
    pub job_id: String,
    #[allow(dead_code)]
    pub status: RunStatus,
    #[allow(dead_code)]
    pub started_at: i64,
}
