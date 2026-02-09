use sqlx::{Row, SqlitePool};

use bastion_core::agent_protocol::HubToAgentMessageV1;

pub(super) async fn authorize_agent_artifact_stream_open(
    db: &SqlitePool,
    agent_id: &str,
    req: &bastion_core::agent_protocol::ArtifactStreamOpenV1,
) -> Result<(), anyhow::Error> {
    let op_id = req.op_id.trim();
    if op_id.is_empty() {
        anyhow::bail!("op_id is required");
    }

    let run_id = req.run_id.trim();
    if run_id.is_empty() {
        anyhow::bail!("run_id is required");
    }

    let row = sqlx::query(
        "SELECT run_id, completed_at, payload_json FROM agent_tasks WHERE id = ? AND agent_id = ? LIMIT 1",
    )
    .bind(op_id)
    .bind(agent_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| anyhow::anyhow!("task not found for agent"))?;

    if row.get::<Option<i64>, _>("completed_at").is_some() {
        anyhow::bail!("task already completed");
    }

    let task_run_id = row.get::<String, _>("run_id");
    if task_run_id.trim() != run_id {
        anyhow::bail!("task run mismatch");
    }

    let payload_json = row.get::<String, _>("payload_json");
    let payload = serde_json::from_str::<HubToAgentMessageV1>(&payload_json)
        .map_err(|_| anyhow::anyhow!("invalid task payload"))?;

    match payload {
        HubToAgentMessageV1::RestoreTask { task_id, task, .. } => {
            if task_id != op_id {
                anyhow::bail!("task id mismatch");
            }
            if task.op_id.trim() != op_id {
                anyhow::bail!("task op_id mismatch");
            }
            if task.run_id.trim() != run_id {
                anyhow::bail!("task payload run mismatch");
            }
        }
        _ => anyhow::bail!("task payload does not allow artifact stream"),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use bastion_core::agent_protocol::{HubToAgentMessageV1, PROTOCOL_VERSION, RestoreTaskV1};
    use bastion_storage::db;

    use super::authorize_agent_artifact_stream_open;

    fn restore_task_payload(op_id: &str, run_id: &str) -> serde_json::Value {
        serde_json::to_value(HubToAgentMessageV1::RestoreTask {
            v: PROTOCOL_VERSION,
            task_id: op_id.to_string(),
            task: Box::new(RestoreTaskV1 {
                op_id: op_id.to_string(),
                run_id: run_id.to_string(),
                destination: None,
                destination_dir: String::new(),
                conflict_policy: "overwrite".to_string(),
                selection: None,
            }),
        })
        .expect("restore task payload")
    }

    #[tokio::test]
    async fn artifact_stream_open_denied_for_other_agent_task() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        sqlx::query(
            "INSERT INTO agent_tasks (id, agent_id, run_id, status, payload_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("op-1")
        .bind("agent-b")
        .bind("run-1")
        .bind("sent")
        .bind(serde_json::to_string(&restore_task_payload("op-1", "run-1")).expect("json"))
        .bind(1_i64)
        .bind(1_i64)
        .execute(&pool)
        .await
        .expect("insert task");

        let req = bastion_core::agent_protocol::ArtifactStreamOpenV1 {
            stream_id: uuid::Uuid::new_v4().to_string(),
            op_id: "op-1".to_string(),
            run_id: "run-1".to_string(),
            artifact: "manifest.json".to_string(),
            path: None,
        };

        let err = authorize_agent_artifact_stream_open(&pool, "agent-a", &req)
            .await
            .expect_err("must deny cross-agent task");
        assert!(err.to_string().contains("task not found"));
    }

    #[tokio::test]
    async fn artifact_stream_open_allowed_for_matching_restore_task() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        sqlx::query(
            "INSERT INTO agent_tasks (id, agent_id, run_id, status, payload_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("op-1")
        .bind("agent-a")
        .bind("run-1")
        .bind("sent")
        .bind(serde_json::to_string(&restore_task_payload("op-1", "run-1")).expect("json"))
        .bind(1_i64)
        .bind(1_i64)
        .execute(&pool)
        .await
        .expect("insert task");

        let req = bastion_core::agent_protocol::ArtifactStreamOpenV1 {
            stream_id: uuid::Uuid::new_v4().to_string(),
            op_id: "op-1".to_string(),
            run_id: "run-1".to_string(),
            artifact: "manifest.json".to_string(),
            path: None,
        };

        authorize_agent_artifact_stream_open(&pool, "agent-a", &req)
            .await
            .expect("must allow matching task");
    }
}
