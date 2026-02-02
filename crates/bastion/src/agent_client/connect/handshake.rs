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

#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use futures_util::Sink;

    use super::*;

    #[derive(Default)]
    struct VecSink {
        sent: Vec<Message>,
    }

    impl Sink<Message> for VecSink {
        type Error = tungstenite::Error;

        fn poll_ready(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
            self.get_mut().sent.push(item);
            Ok(())
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    fn identity() -> AgentIdentityV1 {
        AgentIdentityV1 {
            v: 1,
            hub_url: "http://localhost:9876/".to_string(),
            agent_id: "agent1".to_string(),
            agent_key: "k".to_string(),
            name: Some("n".to_string()),
            enrolled_at: 1,
        }
    }

    #[test]
    fn connected_guard_sets_connected_true_then_false_on_drop() {
        let (tx, rx) = tokio::sync::watch::channel(false);
        assert!(!*rx.borrow());

        {
            let _guard = ConnectedGuard::new(tx);
            assert!(*rx.borrow());
        }

        assert!(!*rx.borrow());
    }

    #[tokio::test]
    async fn send_hello_sends_expected_message() -> Result<(), anyhow::Error> {
        let id = identity();
        let mut sink = VecSink::default();

        send_hello(&mut sink, &id).await?;

        assert_eq!(sink.sent.len(), 1);
        let Message::Text(text) = &sink.sent[0] else {
            anyhow::bail!("expected text message");
        };

        let msg: AgentToHubMessageV1 = serde_json::from_str(text)?;
        match msg {
            AgentToHubMessageV1::Hello {
                v,
                agent_id,
                name,
                info,
                capabilities,
            } => {
                assert_eq!(v, PROTOCOL_VERSION);
                assert_eq!(agent_id, "agent1");
                assert_eq!(name.as_deref(), Some("n"));
                assert_eq!(
                    info.get("version").and_then(|v| v.as_str()),
                    Some(env!("CARGO_PKG_VERSION"))
                );
                assert_eq!(
                    info.get("os").and_then(|v| v.as_str()),
                    Some(std::env::consts::OS)
                );
                assert_eq!(
                    info.get("arch").and_then(|v| v.as_str()),
                    Some(std::env::consts::ARCH)
                );
                assert!(capabilities.get("backup").is_some());
                assert!(capabilities.get("control").is_some());
            }
            other => anyhow::bail!("unexpected message: {other:?}"),
        }

        Ok(())
    }
}
