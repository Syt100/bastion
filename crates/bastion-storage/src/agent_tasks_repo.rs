use serde::Serialize;
use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize)]
pub struct AgentTask {
    pub id: String,
    pub agent_id: String,
    pub run_id: String,
    pub status: String,
    pub payload: serde_json::Value,
    pub created_at: i64,
    pub updated_at: i64,
    pub acked_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub async fn upsert_task(
    db: &SqlitePool,
    id: &str,
    agent_id: &str,
    run_id: &str,
    status: &str,
    payload: &serde_json::Value,
) -> Result<(), anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let payload_json = serde_json::to_string(payload)?;

    sqlx::query(
        r#"
        INSERT INTO agent_tasks (id, agent_id, run_id, status, payload_json, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
          agent_id = excluded.agent_id,
          run_id = excluded.run_id,
          status = excluded.status,
          payload_json = excluded.payload_json,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(id)
    .bind(agent_id)
    .bind(run_id)
    .bind(status)
    .bind(payload_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn delete_task(db: &SqlitePool, id: &str) -> Result<bool, anyhow::Error> {
    let result = sqlx::query("DELETE FROM agent_tasks WHERE id = ?")
        .bind(id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn ack_task(db: &SqlitePool, id: &str) -> Result<bool, anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let result = sqlx::query(
        "UPDATE agent_tasks SET status = 'acked', acked_at = COALESCE(acked_at, ?), updated_at = ? WHERE id = ? AND completed_at IS NULL",
    )
    .bind(now)
    .bind(now)
    .bind(id)
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn complete_task(
    db: &SqlitePool,
    id: &str,
    result: Option<&serde_json::Value>,
    error: Option<&str>,
) -> Result<bool, anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let result_json = match result {
        Some(v) => Some(serde_json::to_string(v)?),
        None => None,
    };

    let result = sqlx::query(
        "UPDATE agent_tasks SET status = 'completed', completed_at = ?, updated_at = ?, result_json = ?, error = ? WHERE id = ? AND completed_at IS NULL",
    )
    .bind(now)
    .bind(now)
    .bind(result_json)
    .bind(error)
    .bind(id)
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_task(db: &SqlitePool, id: &str) -> Result<Option<AgentTask>, anyhow::Error> {
    let row = sqlx::query(
        "SELECT id, agent_id, run_id, status, payload_json, created_at, updated_at, acked_at, completed_at, result_json, error
         FROM agent_tasks
         WHERE id = ?
         LIMIT 1",
    )
    .bind(id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let payload_json = row.get::<String, _>("payload_json");
    let payload = serde_json::from_str::<serde_json::Value>(&payload_json)?;

    let result_json = row.get::<Option<String>, _>("result_json");
    let result = match result_json {
        Some(s) => Some(serde_json::from_str::<serde_json::Value>(&s)?),
        None => None,
    };

    Ok(Some(AgentTask {
        id: row.get::<String, _>("id"),
        agent_id: row.get::<String, _>("agent_id"),
        run_id: row.get::<String, _>("run_id"),
        status: row.get::<String, _>("status"),
        payload,
        created_at: row.get::<i64, _>("created_at"),
        updated_at: row.get::<i64, _>("updated_at"),
        acked_at: row.get::<Option<i64>, _>("acked_at"),
        completed_at: row.get::<Option<i64>, _>("completed_at"),
        result,
        error: row.get::<Option<String>, _>("error"),
    }))
}

pub async fn list_open_tasks_for_agent(
    db: &SqlitePool,
    agent_id: &str,
    limit: u32,
) -> Result<Vec<AgentTask>, anyhow::Error> {
    let rows = sqlx::query(
        r#"
        SELECT t.id, t.agent_id, t.run_id, t.status, t.payload_json, t.created_at, t.updated_at, t.acked_at, t.completed_at, t.result_json, t.error
        FROM agent_tasks t
        LEFT JOIN runs r ON r.id = t.run_id
        LEFT JOIN operations o ON o.id = t.id
        WHERE t.agent_id = ?
          AND t.completed_at IS NULL
          AND (
            r.status = 'running'
            OR o.status = 'running'
          )
        ORDER BY t.created_at ASC
        LIMIT ?
        "#,
    )
    .bind(agent_id)
    .bind(limit as i64)
    .fetch_all(db)
    .await?;

    let mut tasks = Vec::with_capacity(rows.len());
    for row in rows {
        let payload_json = row.get::<String, _>("payload_json");
        let payload = serde_json::from_str::<serde_json::Value>(&payload_json)?;

        let result_json = row.get::<Option<String>, _>("result_json");
        let result = match result_json {
            Some(s) => Some(serde_json::from_str::<serde_json::Value>(&s)?),
            None => None,
        };

        tasks.push(AgentTask {
            id: row.get::<String, _>("id"),
            agent_id: row.get::<String, _>("agent_id"),
            run_id: row.get::<String, _>("run_id"),
            status: row.get::<String, _>("status"),
            payload,
            created_at: row.get::<i64, _>("created_at"),
            updated_at: row.get::<i64, _>("updated_at"),
            acked_at: row.get::<Option<i64>, _>("acked_at"),
            completed_at: row.get::<Option<i64>, _>("completed_at"),
            result,
            error: row.get::<Option<String>, _>("error"),
        });
    }

    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;

    use super::{ack_task, complete_task, get_task, list_open_tasks_for_agent, upsert_task};

    #[tokio::test]
    async fn tasks_round_trip() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();

        // Create a dummy run referenced by the task, and mark it running.
        let job_id = "job1";
        sqlx::query("INSERT INTO jobs (id, name, agent_id, schedule, overlap_policy, spec_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(job_id)
            .bind("job")
            .bind("agent1")
            .bind(Option::<String>::None)
            .bind("queue")
            .bind(r#"{"v":1,"type":"filesystem"}"#)
            .bind(1i64)
            .bind(1i64)
            .execute(&pool)
            .await
            .unwrap();
        let run_id = "run1";
        sqlx::query(
            "INSERT INTO runs (id, job_id, status, started_at) VALUES (?, ?, 'running', ?)",
        )
        .bind(run_id)
        .bind(job_id)
        .bind(1i64)
        .execute(&pool)
        .await
        .unwrap();

        let payload = serde_json::json!({"v":1,"type":"task","task_id":"t1"});
        upsert_task(&pool, "t1", "agent1", run_id, "sent", &payload)
            .await
            .unwrap();

        let tasks = list_open_tasks_for_agent(&pool, "agent1", 10)
            .await
            .unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "t1");
        assert_eq!(get_task(&pool, "t1").await.unwrap().unwrap().id, "t1");

        assert!(ack_task(&pool, "t1").await.unwrap());
        assert!(
            complete_task(&pool, "t1", Some(&serde_json::json!({"ok": true})), None)
                .await
                .unwrap()
        );

        let tasks = list_open_tasks_for_agent(&pool, "agent1", 10)
            .await
            .unwrap();
        assert!(tasks.is_empty());
    }

    #[tokio::test]
    async fn list_open_tasks_includes_running_restore_operation_tasks() {
        let tmp = TempDir::new().unwrap();
        let pool = db::init(tmp.path()).await.unwrap();

        sqlx::query("INSERT INTO jobs (id, name, agent_id, schedule, overlap_policy, spec_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind("job1")
            .bind("job")
            .bind("agent1")
            .bind(Option::<String>::None)
            .bind("queue")
            .bind(r#"{"v":1,"type":"filesystem"}"#)
            .bind(1i64)
            .bind(1i64)
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO runs (id, job_id, status, started_at, ended_at) VALUES (?, ?, 'success', ?, ?)")
            .bind("run-success")
            .bind("job1")
            .bind(1i64)
            .bind(2i64)
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO operations (id, kind, status, created_at, started_at, subject_kind, subject_id)
             VALUES (?, 'restore', 'running', ?, ?, 'run', ?)",
        )
        .bind("op1")
        .bind(10i64)
        .bind(10i64)
        .bind("run-success")
        .execute(&pool)
        .await
        .unwrap();

        let payload = serde_json::json!({"v":1,"type":"restore_task","task_id":"op1","task":{"op_id":"op1","run_id":"run-success","destination_dir":"/tmp","conflict_policy":"overwrite"}});
        upsert_task(&pool, "op1", "agent1", "run-success", "sent", &payload)
            .await
            .unwrap();

        let tasks = list_open_tasks_for_agent(&pool, "agent1", 10)
            .await
            .unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "op1");
    }
}
