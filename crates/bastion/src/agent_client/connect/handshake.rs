use futures_util::{Sink, SinkExt};
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::Message;

use bastion_core::agent_protocol::{AgentToHubMessageV1, PROTOCOL_VERSION};
use bastion_driver_registry::builtins;

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

fn source_driver_entries() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({ "kind": "filesystem", "version": 1 }),
        serde_json::json!({ "kind": "sqlite", "version": 1 }),
        serde_json::json!({ "kind": "vaultwarden", "version": 1 }),
    ]
}

fn target_driver_entries() -> Vec<serde_json::Value> {
    let registry = builtins::target_registry();
    let mut out = Vec::new();

    for id in [
        builtins::local_dir_driver_id(),
        builtins::webdav_driver_id(),
    ] {
        let capabilities = registry
            .target_capabilities(&id)
            .ok()
            .map(|caps| {
                serde_json::json!({
                    "supports_archive_rolling_upload": caps.supports_archive_rolling_upload,
                    "supports_raw_tree_direct_upload": caps.supports_raw_tree_direct_upload,
                    "supports_cleanup_run": caps.supports_cleanup_run,
                    "supports_restore_reader": caps.supports_restore_reader,
                })
            })
            .unwrap_or_else(|| serde_json::json!({}));

        out.push(serde_json::json!({
            "kind": id.kind,
            "version": id.version,
            "capabilities": capabilities,
        }));
    }

    out
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
            "drivers": {
                "source": source_driver_entries(),
                "target": target_driver_entries(),
            }
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

                let source = capabilities
                    .get("drivers")
                    .and_then(|v| v.get("source"))
                    .and_then(|v| v.as_array())
                    .expect("source drivers");
                assert!(source.iter().any(|entry| {
                    entry.get("kind").and_then(|v| v.as_str()) == Some("filesystem")
                        && entry.get("version").and_then(|v| v.as_u64()) == Some(1)
                }));

                let target = capabilities
                    .get("drivers")
                    .and_then(|v| v.get("target"))
                    .and_then(|v| v.as_array())
                    .expect("target drivers");
                assert!(target.iter().any(|entry| {
                    entry.get("kind").and_then(|v| v.as_str()) == Some("webdav")
                        && entry.get("version").and_then(|v| v.as_u64()) == Some(1)
                        && entry
                            .get("capabilities")
                            .and_then(|v| v.get("supports_archive_rolling_upload"))
                            .and_then(|v| v.as_bool())
                            == Some(true)
                }));
            }
            other => anyhow::bail!("unexpected message: {other:?}"),
        }

        Ok(())
    }
}
