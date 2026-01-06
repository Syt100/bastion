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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverlapPolicyV1 {
    Reject,
    Queue,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JobConfigV1 {
    pub job_id: String,
    pub name: String,
    #[serde(default)]
    pub schedule: Option<String>,
    pub overlap_policy: OverlapPolicyV1,
    pub updated_at: i64,
    pub spec: JobSpecResolvedV1,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebdavSecretV1 {
    pub name: String,
    pub username: String,
    pub password: String,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HubToAgentMessageV1 {
    Task {
        v: u32,
        task_id: String,
        task: Box<BackupRunTaskV1>,
    },
    ConfigSnapshot {
        v: u32,
        node_id: String,
        snapshot_id: String,
        issued_at: i64,
        #[serde(default)]
        jobs: Vec<JobConfigV1>,
    },
    SecretsSnapshot {
        v: u32,
        node_id: String,
        issued_at: i64,
        #[serde(default)]
        webdav: Vec<WebdavSecretV1>,
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
    ConfigAck {
        v: u32,
        snapshot_id: String,
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
