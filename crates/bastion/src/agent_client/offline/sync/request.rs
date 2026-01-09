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
