use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Restore,
    Verify,
}

impl OperationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Restore => "restore",
            Self::Verify => "verify",
        }
    }
}

impl std::str::FromStr for OperationKind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "restore" => Ok(Self::Restore),
            "verify" => Ok(Self::Verify),
            _ => Err(anyhow::anyhow!("invalid operation kind")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    Running,
    Success,
    Failed,
}

impl OperationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Success => "success",
            Self::Failed => "failed",
        }
    }
}

impl std::str::FromStr for OperationStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "running" => Ok(Self::Running),
            "success" => Ok(Self::Success),
            "failed" => Ok(Self::Failed),
            _ => Err(anyhow::anyhow!("invalid operation status")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Operation {
    pub id: String,
    pub kind: OperationKind,
    pub status: OperationStatus,
    pub created_at: i64,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub summary: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperationEvent {
    pub op_id: String,
    pub seq: i64,
    pub ts: i64,
    pub level: String,
    pub kind: String,
    pub message: String,
    pub fields: Option<serde_json::Value>,
}
