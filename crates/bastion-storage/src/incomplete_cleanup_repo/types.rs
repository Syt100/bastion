use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupTaskStatus {
    Queued,
    Running,
    Retrying,
    Blocked,
    Done,
    Ignored,
    Abandoned,
}

impl CleanupTaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Retrying => "retrying",
            Self::Blocked => "blocked",
            Self::Done => "done",
            Self::Ignored => "ignored",
            Self::Abandoned => "abandoned",
        }
    }
}

impl std::str::FromStr for CleanupTaskStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "retrying" => Ok(Self::Retrying),
            "blocked" => Ok(Self::Blocked),
            "done" => Ok(Self::Done),
            "ignored" => Ok(Self::Ignored),
            "abandoned" => Ok(Self::Abandoned),
            _ => Err(anyhow::anyhow!("invalid cleanup task status")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupTargetType {
    Webdav,
    LocalDir,
}

impl CleanupTargetType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Webdav => "webdav",
            Self::LocalDir => "local_dir",
        }
    }
}

impl std::str::FromStr for CleanupTargetType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "webdav" => Ok(Self::Webdav),
            "local_dir" => Ok(Self::LocalDir),
            _ => Err(anyhow::anyhow!("invalid cleanup target type")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CleanupTaskRow {
    pub run_id: String,
    pub job_id: String,
    pub node_id: String,
    pub target_type: CleanupTargetType,
    pub target_snapshot: serde_json::Value,
    pub attempts: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanupTaskListItem {
    pub run_id: String,
    pub job_id: String,
    pub job_name: String,
    pub node_id: String,
    pub target_type: String,
    pub status: String,
    pub attempts: i64,
    pub last_attempt_at: Option<i64>,
    pub next_attempt_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_error_kind: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanupTaskDetail {
    pub run_id: String,
    pub job_id: String,
    pub job_name: String,
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
pub struct CleanupEvent {
    pub run_id: String,
    pub seq: i64,
    pub ts: i64,
    pub level: String,
    pub kind: String,
    pub message: String,
    pub fields: Option<serde_json::Value>,
}
