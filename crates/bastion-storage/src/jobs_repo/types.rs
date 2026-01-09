use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlapPolicy {
    Reject,
    Queue,
}

impl OverlapPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reject => "reject",
            Self::Queue => "queue",
        }
    }
}

impl std::str::FromStr for OverlapPolicy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "reject" => Ok(Self::Reject),
            "queue" => Ok(Self::Queue),
            _ => Err(anyhow::anyhow!("invalid overlap_policy")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub agent_id: Option<String>,
    pub schedule: Option<String>,
    pub overlap_policy: OverlapPolicy,
    pub spec: serde_json::Value,
    pub created_at: i64,
    pub updated_at: i64,
}
