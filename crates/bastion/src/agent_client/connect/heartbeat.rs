use std::time::Duration;

use tokio_tungstenite::tungstenite::Message;

use bastion_core::agent_protocol::{AgentToHubMessageV1, PROTOCOL_VERSION};

pub(super) fn pong_timed_out(last_pong: &tokio::time::Instant, pong_timeout: Duration) -> bool {
    last_pong.elapsed() > pong_timeout
}

pub(super) fn ping_message() -> Result<Message, anyhow::Error> {
    let ping = AgentToHubMessageV1::Ping {
        v: PROTOCOL_VERSION,
    };
    Ok(Message::Text(serde_json::to_string(&ping)?.into()))
}

pub(super) fn close_message() -> Message {
    Message::Close(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    use bastion_core::agent_protocol::AgentToHubMessageV1;

    #[test]
    fn pong_timeout_is_detected() {
        let last_pong = tokio::time::Instant::now()
            .checked_sub(Duration::from_secs(60))
            .expect("instant supports checked_sub");
        assert!(pong_timed_out(&last_pong, Duration::from_secs(10)));
        assert!(!pong_timed_out(
            &tokio::time::Instant::now(),
            Duration::from_secs(3600)
        ));
    }

    #[test]
    fn ping_message_serializes_as_agent_ping() -> Result<(), anyhow::Error> {
        let Message::Text(text) = ping_message()? else {
            anyhow::bail!("expected text message");
        };
        let msg: AgentToHubMessageV1 = serde_json::from_str(&text)?;
        match msg {
            AgentToHubMessageV1::Ping { v } => assert_eq!(v, PROTOCOL_VERSION),
            other => anyhow::bail!("unexpected message: {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn close_message_is_close_frame() {
        assert!(matches!(close_message(), Message::Close(None)));
    }
}
