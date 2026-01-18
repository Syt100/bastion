use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::Message;
use tokio::sync::{Mutex, RwLock, mpsc, oneshot};

use bastion_core::agent;
use bastion_core::agent_protocol::{FsDirEntryV1, HubToAgentMessageV1, PROTOCOL_VERSION};

type FsListKey = (String, String); // (agent_id, request_id)
type FsListResult = Result<FsListPage, String>;
type FsListSender = oneshot::Sender<FsListResult>;
type PendingFsList = HashMap<FsListKey, FsListSender>;

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
struct AgentConnection {
    sender: mpsc::UnboundedSender<Message>,
    last_config_snapshot_id: Option<String>,
}

#[derive(Clone, Default)]
pub struct AgentManager {
    inner: Arc<RwLock<HashMap<String, AgentConnection>>>,
    pending_fs_list: Arc<Mutex<PendingFsList>>,
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
        let keys = pending
            .keys()
            .filter(|(id, _)| id == agent_id)
            .cloned()
            .collect::<Vec<_>>();
        for key in keys {
            if let Some(tx) = pending.remove(&key) {
                let _ = tx.send(Err("agent disconnected".to_string()));
            }
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
}
