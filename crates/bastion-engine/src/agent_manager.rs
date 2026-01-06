use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::ws::Message;
use tokio::sync::{RwLock, mpsc};

#[derive(Debug, Clone)]
struct AgentConnection {
    sender: mpsc::UnboundedSender<Message>,
    last_config_snapshot_id: Option<String>,
}

#[derive(Clone, Default)]
pub struct AgentManager {
    inner: Arc<RwLock<HashMap<String, AgentConnection>>>,
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
}
