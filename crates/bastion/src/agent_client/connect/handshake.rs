use futures_util::{Sink, SinkExt};
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;

use bastion_core::agent_protocol::{AgentToHubMessageV1, PROTOCOL_VERSION};

use super::super::identity::AgentIdentityV1;

pub(super) struct ConnectedGuard(tokio::sync::watch::Sender<bool>);

impl ConnectedGuard {
    pub(super) fn new(sender: tokio::sync::watch::Sender<bool>) -> Self {
        let _ = sender.send(true);
        Self(sender)
    }
}

impl Drop for ConnectedGuard {
    fn drop(&mut self) {
        let _ = self.0.send(false);
    }
}

pub(super) async fn send_hello<S>(
    tx: &mut S,
    identity: &AgentIdentityV1,
) -> Result<(), anyhow::Error>
where
    S: Sink<Message, Error = tungstenite::Error> + Unpin,
{
    let hello = AgentToHubMessageV1::Hello {
        v: PROTOCOL_VERSION,
        agent_id: identity.agent_id.clone(),
        name: identity.name.clone(),
        info: serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        }),
        capabilities: serde_json::json!({
            "backup": ["filesystem", "sqlite", "vaultwarden"],
            "control": ["fs_list"],
        }),
    };
    tx.send(Message::Text(serde_json::to_string(&hello)?.into()))
        .await?;
    Ok(())
}
