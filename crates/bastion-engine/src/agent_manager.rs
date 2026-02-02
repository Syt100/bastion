use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::Message;
use tokio::sync::{Mutex, RwLock, mpsc, oneshot};

use bastion_core::agent;
use bastion_core::agent_protocol::{
    ArtifactStreamOpenResultV1, ArtifactStreamOpenV1, ArtifactStreamPullV1, FsDirEntryV1,
    HubToAgentMessageV1, PROTOCOL_VERSION,
};
use uuid::Uuid;

type FsListKey = (String, String); // (agent_id, request_id)
type FsListResult = Result<FsListPage, String>;
type FsListSender = oneshot::Sender<FsListResult>;
type PendingFsList = HashMap<FsListKey, FsListSender>;

type WebdavListKey = (String, String); // (agent_id, request_id)
type WebdavListResult = Result<WebdavListPage, WebdavListRemoteError>;
type WebdavListSender = oneshot::Sender<WebdavListResult>;
type PendingWebdavList = HashMap<WebdavListKey, WebdavListSender>;

type ArtifactStreamKey = (String, Uuid); // (agent_id, stream_id)
type PendingArtifactOpen = HashMap<ArtifactStreamKey, oneshot::Sender<ArtifactStreamOpenResultV1>>;
type PendingArtifactChunk =
    HashMap<ArtifactStreamKey, oneshot::Sender<Result<ArtifactChunk, String>>>;

#[derive(Debug, Clone)]
pub struct ArtifactChunk {
    pub eof: bool,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct FsListOptions {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub q: Option<String>,
    pub kind: Option<String>,
    pub hide_dotfiles: bool,
    pub type_sort: Option<String>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub size_min_bytes: Option<u64>,
    pub size_max_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct FsListPage {
    pub entries: Vec<FsDirEntryV1>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct WebdavListOptions {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub q: Option<String>,
    pub kind: Option<String>,
    pub hide_dotfiles: bool,
    pub type_sort: Option<String>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub size_min_bytes: Option<u64>,
    pub size_max_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct WebdavListPage {
    pub entries: Vec<FsDirEntryV1>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct WebdavListRemoteError {
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for WebdavListRemoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for WebdavListRemoteError {}

#[derive(Debug, Clone)]
struct AgentConnection {
    sender: mpsc::UnboundedSender<Message>,
    last_config_snapshot_id: Option<String>,
}

#[derive(Clone, Default)]
pub struct AgentManager {
    inner: Arc<RwLock<HashMap<String, AgentConnection>>>,
    pending_fs_list: Arc<Mutex<PendingFsList>>,
    pending_webdav_list: Arc<Mutex<PendingWebdavList>>,
    pending_artifact_open: Arc<Mutex<PendingArtifactOpen>>,
    pending_artifact_chunk: Arc<Mutex<PendingArtifactChunk>>,
}

impl AgentManager {
    pub async fn register(&self, agent_id: String, sender: mpsc::UnboundedSender<Message>) {
        self.inner.write().await.insert(
            agent_id,
            AgentConnection {
                sender,
                last_config_snapshot_id: None,
            },
        );
    }

    pub async fn unregister(&self, agent_id: &str) {
        self.inner.write().await.remove(agent_id);

        let mut pending = self.pending_fs_list.lock().await;
        for (_key, tx) in pending.extract_if(|key, _| key.0.as_str() == agent_id) {
            let _ = tx.send(Err("agent disconnected".to_string()));
        }

        let mut pending_webdav = self.pending_webdav_list.lock().await;
        for (_key, tx) in pending_webdav.extract_if(|key, _| key.0.as_str() == agent_id) {
            let _ = tx.send(Err(WebdavListRemoteError {
                code: "agent_offline".to_string(),
                message: "agent disconnected".to_string(),
            }));
        }

        let mut pending_open = self.pending_artifact_open.lock().await;
        for ((_id, stream_id), tx) in pending_open.extract_if(|key, _| key.0.as_str() == agent_id) {
            let _ = tx.send(ArtifactStreamOpenResultV1 {
                stream_id: stream_id.to_string(),
                size: None,
                error: Some("agent disconnected".to_string()),
            });
        }

        let mut pending_chunk = self.pending_artifact_chunk.lock().await;
        for (_key, tx) in pending_chunk.extract_if(|key, _| key.0.as_str() == agent_id) {
            let _ = tx.send(Err("agent disconnected".to_string()));
        }
    }

    pub async fn is_connected(&self, agent_id: &str) -> bool {
        self.inner.read().await.contains_key(agent_id)
    }

    pub async fn send(&self, agent_id: &str, msg: Message) -> Result<(), anyhow::Error> {
        let sender = self
            .inner
            .read()
            .await
            .get(agent_id)
            .map(|v| v.sender.clone())
            .ok_or_else(|| anyhow::anyhow!("agent not connected"))?;

        sender
            .send(msg)
            .map_err(|_| anyhow::anyhow!("agent send failed"))?;
        Ok(())
    }

    pub async fn send_json<T: serde::Serialize>(
        &self,
        agent_id: &str,
        value: &T,
    ) -> Result<(), anyhow::Error> {
        let text = serde_json::to_string(value)?;
        self.send(agent_id, Message::Text(text.into())).await
    }

    pub async fn send_config_snapshot_json<T: serde::Serialize>(
        &self,
        agent_id: &str,
        snapshot_id: &str,
        value: &T,
    ) -> Result<bool, anyhow::Error> {
        let text = serde_json::to_string(value)?;

        let mut guard = self.inner.write().await;
        let conn = guard
            .get_mut(agent_id)
            .ok_or_else(|| anyhow::anyhow!("agent not connected"))?;

        if conn.last_config_snapshot_id.as_deref() == Some(snapshot_id) {
            return Ok(false);
        }

        conn.sender
            .send(Message::Text(text.into()))
            .map_err(|_| anyhow::anyhow!("agent send failed"))?;
        conn.last_config_snapshot_id = Some(snapshot_id.to_string());
        Ok(true)
    }

    pub async fn fs_list_page(
        &self,
        agent_id: &str,
        path: String,
        opts: FsListOptions,
        timeout: std::time::Duration,
    ) -> Result<FsListPage, anyhow::Error> {
        let request_id = agent::generate_token_b64_urlsafe(16);
        let (tx, rx) = oneshot::channel::<FsListResult>();
        self.pending_fs_list
            .lock()
            .await
            .insert((agent_id.to_string(), request_id.clone()), tx);

        let msg = HubToAgentMessageV1::FsList {
            v: PROTOCOL_VERSION,
            request_id: request_id.clone(),
            path,
            cursor: opts.cursor,
            limit: opts.limit,
            q: opts.q,
            kind: opts.kind,
            hide_dotfiles: if opts.hide_dotfiles { Some(true) } else { None },
            type_sort: opts.type_sort,
            sort_by: opts.sort_by,
            sort_dir: opts.sort_dir,
            size_min_bytes: opts.size_min_bytes,
            size_max_bytes: opts.size_max_bytes,
        };
        if let Err(error) = self.send_json(agent_id, &msg).await {
            let _ = self
                .pending_fs_list
                .lock()
                .await
                .remove(&(agent_id.to_string(), request_id));
            return Err(error);
        }

        let result = tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| anyhow::anyhow!("agent fs list timeout"))?
            .map_err(|_| anyhow::anyhow!("agent fs list channel closed"))?;

        // Remove in case the response arrived after a timeout and the slot is still present.
        let _ = self
            .pending_fs_list
            .lock()
            .await
            .remove(&(agent_id.to_string(), request_id));

        result.map_err(anyhow::Error::msg)
    }

    pub async fn fs_list(
        &self,
        agent_id: &str,
        path: String,
        timeout: std::time::Duration,
    ) -> Result<Vec<FsDirEntryV1>, anyhow::Error> {
        let page = self
            .fs_list_page(
                agent_id,
                path,
                FsListOptions {
                    cursor: None,
                    limit: None,
                    q: None,
                    kind: None,
                    hide_dotfiles: false,
                    type_sort: None,
                    sort_by: None,
                    sort_dir: None,
                    size_min_bytes: None,
                    size_max_bytes: None,
                },
                timeout,
            )
            .await?;
        Ok(page.entries)
    }

    pub async fn complete_fs_list(&self, agent_id: &str, request_id: &str, result: FsListResult) {
        let key = (agent_id.to_string(), request_id.to_string());
        let tx = self.pending_fs_list.lock().await.remove(&key);
        if let Some(tx) = tx {
            let _ = tx.send(result);
        }
    }

    pub async fn webdav_list_page(
        &self,
        agent_id: &str,
        base_url: String,
        secret_name: String,
        path: String,
        opts: WebdavListOptions,
        timeout: std::time::Duration,
    ) -> Result<WebdavListPage, anyhow::Error> {
        let request_id = agent::generate_token_b64_urlsafe(16);
        let (tx, rx) = oneshot::channel::<WebdavListResult>();
        self.pending_webdav_list
            .lock()
            .await
            .insert((agent_id.to_string(), request_id.clone()), tx);

        let msg = HubToAgentMessageV1::WebdavList {
            v: PROTOCOL_VERSION,
            request_id: request_id.clone(),
            base_url,
            secret_name,
            path,
            cursor: opts.cursor,
            limit: opts.limit,
            q: opts.q,
            kind: opts.kind,
            hide_dotfiles: if opts.hide_dotfiles { Some(true) } else { None },
            type_sort: opts.type_sort,
            sort_by: opts.sort_by,
            sort_dir: opts.sort_dir,
            size_min_bytes: opts.size_min_bytes,
            size_max_bytes: opts.size_max_bytes,
        };
        if let Err(error) = self.send_json(agent_id, &msg).await {
            let _ = self
                .pending_webdav_list
                .lock()
                .await
                .remove(&(agent_id.to_string(), request_id));
            return Err(error);
        }

        let result = tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| anyhow::anyhow!("agent webdav list timeout"))?
            .map_err(|_| anyhow::anyhow!("agent webdav list channel closed"))?;

        // Remove in case the response arrived after a timeout and the slot is still present.
        let _ = self
            .pending_webdav_list
            .lock()
            .await
            .remove(&(agent_id.to_string(), request_id));

        result.map_err(anyhow::Error::new)
    }

    pub async fn complete_webdav_list(
        &self,
        agent_id: &str,
        request_id: &str,
        result: WebdavListResult,
    ) {
        let key = (agent_id.to_string(), request_id.to_string());
        let tx = self.pending_webdav_list.lock().await.remove(&key);
        if let Some(tx) = tx {
            let _ = tx.send(result);
        }
    }

    pub async fn artifact_stream_open(
        &self,
        agent_id: &str,
        req: ArtifactStreamOpenV1,
        timeout: std::time::Duration,
    ) -> Result<ArtifactStreamOpenResultV1, anyhow::Error> {
        let stream_id = Uuid::parse_str(req.stream_id.trim())?;
        let key = (agent_id.to_string(), stream_id);
        let (tx, rx) = oneshot::channel::<ArtifactStreamOpenResultV1>();
        self.pending_artifact_open
            .lock()
            .await
            .insert(key.clone(), tx);

        let msg = HubToAgentMessageV1::ArtifactStreamOpen {
            v: PROTOCOL_VERSION,
            req,
        };
        if let Err(error) = self.send_json(agent_id, &msg).await {
            let _ = self.pending_artifact_open.lock().await.remove(&key);
            return Err(error);
        }

        let res = tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| anyhow::anyhow!("agent artifact stream open timeout"))?
            .map_err(|_| anyhow::anyhow!("agent artifact stream open channel closed"))?;

        let _ = self.pending_artifact_open.lock().await.remove(&key);
        Ok(res)
    }

    pub async fn complete_artifact_stream_open(
        &self,
        agent_id: &str,
        res: ArtifactStreamOpenResultV1,
    ) {
        let Ok(stream_id) = Uuid::parse_str(res.stream_id.trim()) else {
            return;
        };
        let key = (agent_id.to_string(), stream_id);
        let tx = self.pending_artifact_open.lock().await.remove(&key);
        if let Some(tx) = tx {
            let _ = tx.send(res);
        }
    }

    pub async fn artifact_stream_pull(
        &self,
        agent_id: &str,
        req: ArtifactStreamPullV1,
        timeout: std::time::Duration,
    ) -> Result<ArtifactChunk, anyhow::Error> {
        let stream_id = Uuid::parse_str(req.stream_id.trim())?;
        let key = (agent_id.to_string(), stream_id);
        let (tx, rx) = oneshot::channel::<Result<ArtifactChunk, String>>();
        self.pending_artifact_chunk
            .lock()
            .await
            .insert(key.clone(), tx);

        let msg = HubToAgentMessageV1::ArtifactStreamPull {
            v: PROTOCOL_VERSION,
            req,
        };
        if let Err(error) = self.send_json(agent_id, &msg).await {
            let _ = self.pending_artifact_chunk.lock().await.remove(&key);
            return Err(error);
        }

        let res = tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| anyhow::anyhow!("agent artifact stream pull timeout"))?
            .map_err(|_| anyhow::anyhow!("agent artifact stream pull channel closed"))?;

        let _ = self.pending_artifact_chunk.lock().await.remove(&key);
        res.map_err(anyhow::Error::msg)
    }

    pub async fn complete_artifact_stream_chunk(
        &self,
        agent_id: &str,
        stream_id: Uuid,
        chunk: ArtifactChunk,
    ) {
        let key = (agent_id.to_string(), stream_id);
        let tx = self.pending_artifact_chunk.lock().await.remove(&key);
        if let Some(tx) = tx {
            let _ = tx.send(Ok(chunk));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::mpsc;
    use uuid::Uuid;

    use super::{AgentManager, ArtifactStreamOpenV1, FsListOptions};

    #[tokio::test]
    async fn pending_fs_list_page_fails_fast_on_disconnect() {
        let manager = AgentManager::default();
        let (sender, mut receiver) = mpsc::unbounded_channel();
        manager.register("agent1".to_string(), sender).await;

        let manager_task = manager.clone();
        let task = tokio::spawn(async move {
            manager_task
                .fs_list_page(
                    "agent1",
                    "/".to_string(),
                    FsListOptions {
                        cursor: None,
                        limit: None,
                        q: None,
                        kind: None,
                        hide_dotfiles: false,
                        type_sort: None,
                        sort_by: None,
                        sort_dir: None,
                        size_min_bytes: None,
                        size_max_bytes: None,
                    },
                    Duration::from_secs(30),
                )
                .await
        });

        // Ensure the request has been sent before we unregister (so the pending map is populated).
        let _ = receiver.recv().await.expect("sent request");
        manager.unregister("agent1").await;

        let err = task.await.expect("task join").expect_err("should error");
        assert_eq!(err.to_string(), "agent disconnected");
    }

    #[tokio::test]
    async fn pending_artifact_stream_open_returns_structured_error_on_disconnect() {
        let manager = AgentManager::default();
        let (sender, mut receiver) = mpsc::unbounded_channel();
        manager.register("agent1".to_string(), sender).await;

        let stream_id = Uuid::new_v4().to_string();
        let req = ArtifactStreamOpenV1 {
            stream_id: stream_id.clone(),
            op_id: "op1".to_string(),
            run_id: "run1".to_string(),
            artifact: "entries_index".to_string(),
            path: None,
        };

        let manager_task = manager.clone();
        let task = tokio::spawn(async move {
            manager_task
                .artifact_stream_open("agent1", req, Duration::from_secs(30))
                .await
        });

        // Ensure the request has been sent before we unregister.
        let _ = receiver.recv().await.expect("sent request");
        manager.unregister("agent1").await;

        let res = task.await.expect("task join").expect("open result");
        assert_eq!(res.stream_id, stream_id);
        assert_eq!(res.error.as_deref(), Some("agent disconnected"));
    }
}
