use serde::{Deserialize, Serialize};

use crate::job_spec::{FilesystemSource, SqliteSource, VaultwardenSource};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EncryptionResolvedV1 {
    #[default]
    None,
    AgeX25519 {
        recipient: String,
        key_name: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PipelineResolvedV1 {
    #[serde(default)]
    pub encryption: EncryptionResolvedV1,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TargetResolvedV1 {
    Webdav {
        base_url: String,
        username: String,
        password: String,
        part_size_bytes: u64,
    },
    LocalDir {
        base_dir: String,
        part_size_bytes: u64,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JobSpecResolvedV1 {
    Filesystem {
        v: u32,
        #[serde(default)]
        pipeline: PipelineResolvedV1,
        source: FilesystemSource,
        target: TargetResolvedV1,
    },
    Sqlite {
        v: u32,
        #[serde(default)]
        pipeline: PipelineResolvedV1,
        source: SqliteSource,
        target: TargetResolvedV1,
    },
    Vaultwarden {
        v: u32,
        #[serde(default)]
        pipeline: PipelineResolvedV1,
        source: VaultwardenSource,
        target: TargetResolvedV1,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupRunTaskV1 {
    pub run_id: String,
    pub job_id: String,
    pub started_at: i64,
    pub spec: JobSpecResolvedV1,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HubToAgentMessageV1 {
    Task {
        v: u32,
        task_id: String,
        task: Box<BackupRunTaskV1>,
    },
    Pong {
        v: u32,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentToHubMessageV1 {
    Hello {
        v: u32,
        agent_id: String,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        info: serde_json::Value,
        #[serde(default)]
        capabilities: serde_json::Value,
    },
    Ping {
        v: u32,
    },
    Ack {
        v: u32,
        task_id: String,
    },
    RunEvent {
        v: u32,
        run_id: String,
        level: String,
        kind: String,
        message: String,
        #[serde(default)]
        fields: Option<serde_json::Value>,
    },
    TaskResult {
        v: u32,
        task_id: String,
        run_id: String,
        status: String,
        #[serde(default)]
        summary: Option<serde_json::Value>,
        #[serde(default)]
        error: Option<String>,
    },
}
