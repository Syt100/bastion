use serde::Serialize;

use super::OfflineRunEventV1;
use super::OfflineRunFileV1;

#[derive(Debug, Serialize)]
pub(super) struct AgentIngestRunRequestV1 {
    pub(super) run: AgentIngestRunV1,
}

#[derive(Debug, Serialize)]
pub(super) struct AgentIngestRunV1 {
    pub(super) id: String,
    pub(super) job_id: String,
    pub(super) status: String,
    pub(super) started_at: i64,
    pub(super) ended_at: i64,
    pub(super) summary: Option<serde_json::Value>,
    pub(super) error: Option<String>,
    pub(super) events: Vec<OfflineRunEventV1>,
}

impl AgentIngestRunRequestV1 {
    pub(super) fn from_offline_run(
        run: OfflineRunFileV1,
        ended_at: i64,
        status: &str,
        events: Vec<OfflineRunEventV1>,
    ) -> Self {
        Self {
            run: AgentIngestRunV1 {
                id: run.id,
                job_id: run.job_id,
                status: status.to_string(),
                started_at: run.started_at,
                ended_at,
                summary: run.summary,
                error: run.error,
                events,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentIngestRunRequestV1, OfflineRunEventV1, OfflineRunFileV1};
    use crate::agent_client::offline::storage::OfflineRunStatusV1;

    #[test]
    fn from_offline_run_maps_fields_and_overrides_status_and_ended_at() {
        let run = OfflineRunFileV1 {
            v: 1,
            id: "run1".to_string(),
            job_id: "job1".to_string(),
            job_name: "job name".to_string(),
            status: OfflineRunStatusV1::Success,
            started_at: 10,
            ended_at: Some(20),
            summary: Some(serde_json::json!({"k": "v"})),
            error: Some("err".to_string()),
        };
        let events = vec![OfflineRunEventV1 {
            seq: 1,
            ts: 10,
            level: "info".to_string(),
            kind: "start".to_string(),
            message: "a".to_string(),
            fields: None,
        }];

        let req = AgentIngestRunRequestV1::from_offline_run(run, 999, "failed", events);
        assert_eq!(req.run.id, "run1");
        assert_eq!(req.run.job_id, "job1");
        assert_eq!(req.run.status, "failed");
        assert_eq!(req.run.started_at, 10);
        assert_eq!(req.run.ended_at, 999);
        assert_eq!(req.run.summary, Some(serde_json::json!({"k": "v"})));
        assert_eq!(req.run.error.as_deref(), Some("err"));
        assert_eq!(req.run.events.len(), 1);
        assert_eq!(req.run.events[0].seq, 1);
    }
}
