use serde::{Deserialize, Serialize};

use crate::job_spec::{FilesystemSource, PipelineWebdavV1, SqliteSource, VaultwardenSource};
use crate::manifest::ArtifactFormatV1;

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FsDirEntryV1 {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub size: u64,
    #[serde(default)]
    pub mtime: Option<i64>,
}

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
    pub format: ArtifactFormatV1,
    #[serde(default)]
    pub encryption: EncryptionResolvedV1,
    #[serde(default)]
    pub webdav: PipelineWebdavV1,
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RestoreSelectionV1 {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub dirs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RestoreDestinationV1 {
    LocalFs {
        directory: String,
    },
    Webdav {
        base_url: String,
        secret_name: String,
        prefix: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RestoreTaskV1 {
    pub op_id: String,
    pub run_id: String,
    #[serde(default)]
    pub destination: Option<RestoreDestinationV1>,
    // Legacy field for older task payloads (local_fs only).
    #[serde(default)]
    pub destination_dir: String,
    pub conflict_policy: String,
    #[serde(default)]
    pub selection: Option<RestoreSelectionV1>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapshotDeleteTaskV1 {
    pub run_id: String,
    pub job_id: String,
    pub base_dir: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperationEventV1 {
    pub op_id: String,
    pub level: String,
    pub kind: String,
    pub message: String,
    #[serde(default)]
    pub fields: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperationResultV1 {
    pub op_id: String,
    pub status: String,
    #[serde(default)]
    pub summary: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtifactStreamOpenV1 {
    pub stream_id: String,
    pub op_id: String,
    pub run_id: String,
    pub artifact: String,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtifactStreamOpenResultV1 {
    pub stream_id: String,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtifactStreamPullV1 {
    pub stream_id: String,
    pub max_bytes: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtifactStreamCloseV1 {
    pub stream_id: String,
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
    #[serde(default)]
    pub schedule_timezone: Option<String>,
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
pub struct BackupAgeIdentitySecretV1 {
    pub name: String,
    pub identity: String,
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
    RestoreTask {
        v: u32,
        task_id: String,
        task: Box<RestoreTaskV1>,
    },
    SnapshotDeleteTask {
        v: u32,
        task: SnapshotDeleteTaskV1,
    },
    FsList {
        v: u32,
        request_id: String,
        path: String,
        #[serde(default)]
        cursor: Option<String>,
        #[serde(default)]
        limit: Option<u32>,
        #[serde(default)]
        q: Option<String>,
        #[serde(default)]
        kind: Option<String>,
        #[serde(default)]
        hide_dotfiles: Option<bool>,
        #[serde(default)]
        type_sort: Option<String>,
        #[serde(default)]
        sort_by: Option<String>,
        #[serde(default)]
        sort_dir: Option<String>,
        #[serde(default)]
        size_min_bytes: Option<u64>,
        #[serde(default)]
        size_max_bytes: Option<u64>,
    },
    WebdavList {
        v: u32,
        request_id: String,
        base_url: String,
        secret_name: String,
        path: String,
        #[serde(default)]
        cursor: Option<String>,
        #[serde(default)]
        limit: Option<u32>,
        #[serde(default)]
        q: Option<String>,
        #[serde(default)]
        kind: Option<String>,
        #[serde(default)]
        hide_dotfiles: Option<bool>,
        #[serde(default)]
        type_sort: Option<String>,
        #[serde(default)]
        sort_by: Option<String>,
        #[serde(default)]
        sort_dir: Option<String>,
        #[serde(default)]
        size_min_bytes: Option<u64>,
        #[serde(default)]
        size_max_bytes: Option<u64>,
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
        #[serde(default)]
        backup_age_identities: Vec<BackupAgeIdentitySecretV1>,
    },
    ArtifactStreamOpen {
        v: u32,
        req: ArtifactStreamOpenV1,
    },
    ArtifactStreamOpenResult {
        v: u32,
        res: ArtifactStreamOpenResultV1,
    },
    ArtifactStreamPull {
        v: u32,
        req: ArtifactStreamPullV1,
    },
    ArtifactStreamClose {
        v: u32,
        req: ArtifactStreamCloseV1,
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
    SnapshotDeleteEvent {
        v: u32,
        run_id: String,
        level: String,
        kind: String,
        message: String,
        #[serde(default)]
        fields: Option<serde_json::Value>,
    },
    SnapshotDeleteResult {
        v: u32,
        run_id: String,
        status: String,
        #[serde(default)]
        error_kind: Option<String>,
        #[serde(default)]
        error: Option<String>,
    },
    OperationEvent {
        v: u32,
        event: OperationEventV1,
    },
    OperationResult {
        v: u32,
        result: OperationResultV1,
    },
    FsListResult {
        v: u32,
        request_id: String,
        #[serde(default)]
        entries: Vec<FsDirEntryV1>,
        #[serde(default)]
        next_cursor: Option<String>,
        #[serde(default)]
        total: Option<u64>,
        #[serde(default)]
        error: Option<String>,
    },
    WebdavListResult {
        v: u32,
        request_id: String,
        #[serde(default)]
        entries: Vec<FsDirEntryV1>,
        #[serde(default)]
        next_cursor: Option<String>,
        #[serde(default)]
        total: Option<u64>,
        #[serde(default)]
        error_code: Option<String>,
        #[serde(default)]
        error: Option<String>,
    },
    ArtifactStreamOpen {
        v: u32,
        req: ArtifactStreamOpenV1,
    },
    ArtifactStreamOpenResult {
        v: u32,
        res: ArtifactStreamOpenResultV1,
    },
    ArtifactStreamPull {
        v: u32,
        req: ArtifactStreamPullV1,
    },
    ArtifactStreamClose {
        v: u32,
        req: ArtifactStreamCloseV1,
    },
}

#[cfg(test)]
mod tests {
    use super::{AgentToHubMessageV1, HubToAgentMessageV1, PROTOCOL_VERSION, SnapshotDeleteTaskV1};

    #[test]
    fn snapshot_delete_task_round_trip() {
        let msg = HubToAgentMessageV1::SnapshotDeleteTask {
            v: PROTOCOL_VERSION,
            task: SnapshotDeleteTaskV1 {
                run_id: "r1".to_string(),
                job_id: "j1".to_string(),
                base_dir: "/tmp".to_string(),
            },
        };

        let json = serde_json::to_string(&msg).expect("serialize");
        let decoded = serde_json::from_str::<HubToAgentMessageV1>(&json).expect("deserialize");
        match decoded {
            HubToAgentMessageV1::SnapshotDeleteTask { v, task } => {
                assert_eq!(v, PROTOCOL_VERSION);
                assert_eq!(task.run_id, "r1");
                assert_eq!(task.job_id, "j1");
                assert_eq!(task.base_dir, "/tmp");
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[test]
    fn snapshot_delete_result_round_trip() {
        let msg = AgentToHubMessageV1::SnapshotDeleteResult {
            v: PROTOCOL_VERSION,
            run_id: "r1".to_string(),
            status: "success".to_string(),
            error_kind: None,
            error: None,
        };

        let json = serde_json::to_string(&msg).expect("serialize");
        let decoded = serde_json::from_str::<AgentToHubMessageV1>(&json).expect("deserialize");
        match decoded {
            AgentToHubMessageV1::SnapshotDeleteResult {
                v, run_id, status, ..
            } => {
                assert_eq!(v, PROTOCOL_VERSION);
                assert_eq!(run_id, "r1");
                assert_eq!(status, "success");
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }
}
