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
