use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::Message;
use tokio::sync::{Mutex, RwLock, mpsc, oneshot};

use bastion_core::agent;
use bastion_core::agent_protocol::{FsDirEntryV1, HubToAgentMessageV1, PROTOCOL_VERSION};

#[derive(Debug, Clone)]
struct AgentConnection {
    sender: mpsc::UnboundedSender<Message>,
    last_config_snapshot_id: Option<String>,
}

#[derive(Clone, Default)]
pub struct AgentManager {
    inner: Arc<RwLock<HashMap<String, AgentConnection>>>,
    pending_fs_list: Arc<Mutex<HashMap<(String, String), oneshot::Sender<Result<Vec<FsDirEntryV1>, String>>>>>,
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

    pub async fn fs_list(
        &self,
        agent_id: &str,
        path: String,
        timeout: std::time::Duration,
    ) -> Result<Vec<FsDirEntryV1>, anyhow::Error> {
        let request_id = agent::generate_token_b64_urlsafe(16);
        let (tx, rx) = oneshot::channel::<Result<Vec<FsDirEntryV1>, String>>();
        self.pending_fs_list
            .lock()
            .await
            .insert((agent_id.to_string(), request_id.clone()), tx);

        let msg = HubToAgentMessageV1::FsList {
            v: PROTOCOL_VERSION,
            request_id: request_id.clone(),
            path,
        };
        if let Err(error) = self.send_json(agent_id, &msg).await {
            let _ = self
                .pending_fs_list
                .lock()
                .await
                .remove(&(agent_id.to_string(), request_id));
            return Err(error);
        }

        let result =
            tokio::time::timeout(timeout, rx)
                .await
                .map_err(|_| anyhow::anyhow!("agent fs list timeout"))?
                .map_err(|_| anyhow::anyhow!("agent fs list channel closed"))?;

        // Remove in case the response arrived after a timeout and the slot is still present.
        let _ = self
            .pending_fs_list
            .lock()
            .await
            .remove(&(agent_id.to_string(), request_id));

        match result {
            Ok(entries) => Ok(entries),
            Err(msg) => Err(anyhow::anyhow!(msg)),
        }
    }

    pub async fn complete_fs_list(
        &self,
        agent_id: &str,
        request_id: &str,
        result: Result<Vec<FsDirEntryV1>, String>,
    ) {
        let key = (agent_id.to_string(), request_id.to_string());
        let tx = self.pending_fs_list.lock().await.remove(&key);
        if let Some(tx) = tx {
            let _ = tx.send(result);
        }
    }
}
