use tokio_tungstenite::tungstenite::Message;

use bastion_core::agent_protocol::AgentToHubMessageV1;

use super::super::storage::OfflineRunWriterHandle;

pub(super) struct OfflineSink {
    writer: OfflineRunWriterHandle,
    task_summary: Option<serde_json::Value>,
}

impl OfflineSink {
    pub(super) fn new(writer: OfflineRunWriterHandle) -> Self {
        Self {
            writer,
            task_summary: None,
        }
    }

    pub(super) fn writer(&self) -> &OfflineRunWriterHandle {
        &self.writer
    }

    pub(super) fn into_parts(self) -> (OfflineRunWriterHandle, Option<serde_json::Value>) {
        (self.writer, self.task_summary)
    }
}

impl futures_util::Sink<Message> for OfflineSink {
    type Error = tokio_tungstenite::tungstenite::Error;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn start_send(mut self: std::pin::Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        if let Message::Text(text) = item {
            let text = text.to_string();
            if let Ok(msg) = serde_json::from_str::<AgentToHubMessageV1>(&text) {
                match msg {
                    AgentToHubMessageV1::RunEvent {
                        level,
                        kind,
                        message,
                        fields,
                        ..
                    } => {
                        self.writer
                            .append_event(&level, &kind, &message, fields)
                            .map_err(to_ws_error)?;
                    }
                    AgentToHubMessageV1::TaskResult { summary, .. } => {
                        self.task_summary = summary;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

fn to_ws_error(error: anyhow::Error) -> tokio_tungstenite::tungstenite::Error {
    tokio_tungstenite::tungstenite::Error::Io(std::io::Error::other(error.to_string()))
}

pub(super) fn mark_summary_executed_offline(summary: &mut serde_json::Value) {
    if let Some(obj) = summary.as_object_mut() {
        obj.insert(
            "executed_offline".to_string(),
            serde_json::Value::Bool(true),
        );
    } else {
        *summary = serde_json::json!({ "executed_offline": true });
    }
}
