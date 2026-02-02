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

#[cfg(test)]
mod tests {
    use futures_util::SinkExt as _;
    use tokio_tungstenite::tungstenite::Message;

    use bastion_core::agent_protocol::{AgentToHubMessageV1, PROTOCOL_VERSION};

    use super::super::super::storage::OfflineRunEventV1;
    use super::super::super::storage::OfflineRunWriterHandle;
    use super::{OfflineSink, mark_summary_executed_offline};

    #[test]
    fn mark_summary_executed_offline_inserts_flag_for_objects() {
        let mut summary = serde_json::json!({ "k": "v" });
        mark_summary_executed_offline(&mut summary);
        assert_eq!(
            summary.get("k"),
            Some(&serde_json::Value::String("v".to_string()))
        );
        assert_eq!(
            summary.get("executed_offline"),
            Some(&serde_json::Value::Bool(true))
        );
    }

    #[test]
    fn mark_summary_executed_offline_replaces_non_objects() {
        let mut summary = serde_json::json!(["not", "an", "object"]);
        mark_summary_executed_offline(&mut summary);
        assert_eq!(summary, serde_json::json!({ "executed_offline": true }));
    }

    #[tokio::test]
    async fn offline_sink_appends_run_events_and_captures_task_summary() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path();

        let writer = OfflineRunWriterHandle::start(data_dir, "run1", "job1", "job name", 1)
            .await
            .unwrap();
        let mut sink = OfflineSink::new(writer);

        let run_event = AgentToHubMessageV1::RunEvent {
            v: PROTOCOL_VERSION,
            run_id: "run1".to_string(),
            level: "info".to_string(),
            kind: "start".to_string(),
            message: "hello".to_string(),
            fields: Some(serde_json::json!({ "x": 1 })),
        };
        sink.send(Message::Text(
            serde_json::to_string(&run_event).unwrap().into(),
        ))
        .await
        .unwrap();

        let task_result = AgentToHubMessageV1::TaskResult {
            v: PROTOCOL_VERSION,
            task_id: "task1".to_string(),
            run_id: "run1".to_string(),
            status: "success".to_string(),
            summary: Some(serde_json::json!({ "k": "v" })),
            error: None,
        };
        sink.send(Message::Text(
            serde_json::to_string(&task_result).unwrap().into(),
        ))
        .await
        .unwrap();

        let (writer, summary) = sink.into_parts();
        assert_eq!(summary, Some(serde_json::json!({ "k": "v" })));

        writer
            .finish_success(serde_json::json!({ "done": true }))
            .await
            .unwrap();

        let events_path = data_dir
            .join("agent")
            .join("offline_runs")
            .join("run1")
            .join("events.jsonl");
        let text = std::fs::read_to_string(&events_path).unwrap();
        let events = text
            .lines()
            .map(|line| serde_json::from_str::<OfflineRunEventV1>(line).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, "info");
        assert_eq!(events[0].kind, "start");
        assert_eq!(events[0].message, "hello");
        assert_eq!(events[0].fields, Some(serde_json::json!({ "x": 1 })));
    }
}
