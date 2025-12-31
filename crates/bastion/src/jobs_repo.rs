use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlapPolicy {
    Reject,
    Queue,
}

impl OverlapPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reject => "reject",
            Self::Queue => "queue",
        }
    }
}

impl std::str::FromStr for OverlapPolicy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "reject" => Ok(Self::Reject),
            "queue" => Ok(Self::Queue),
            _ => Err(anyhow::anyhow!("invalid overlap_policy")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub agent_id: Option<String>,
    pub schedule: Option<String>,
    pub overlap_policy: OverlapPolicy,
    pub spec: serde_json::Value,
    pub created_at: i64,
    pub updated_at: i64,
}

pub async fn create_job(
    db: &SqlitePool,
    name: &str,
    agent_id: Option<&str>,
    schedule: Option<&str>,
    overlap_policy: OverlapPolicy,
    spec: serde_json::Value,
) -> Result<Job, anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let id = Uuid::new_v4().to_string();
    let spec_json = serde_json::to_string(&spec)?;

    sqlx::query(
        r#"
        INSERT INTO jobs (id, name, agent_id, schedule, overlap_policy, spec_json, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(name)
    .bind(agent_id)
    .bind(schedule)
    .bind(overlap_policy.as_str())
    .bind(spec_json)
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    Ok(Job {
        id,
        name: name.to_string(),
        agent_id: agent_id.map(|s| s.to_string()),
        schedule: schedule.map(|s| s.to_string()),
        overlap_policy,
        spec,
        created_at: now,
        updated_at: now,
    })
}

pub async fn get_job(db: &SqlitePool, job_id: &str) -> Result<Option<Job>, anyhow::Error> {
    let row = sqlx::query(
        "SELECT id, name, agent_id, schedule, overlap_policy, spec_json, created_at, updated_at FROM jobs WHERE id = ? LIMIT 1",
    )
    .bind(job_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let overlap_policy = row
        .get::<String, _>("overlap_policy")
        .parse::<OverlapPolicy>()?;
    let spec_json = row.get::<String, _>("spec_json");
    let spec = serde_json::from_str::<serde_json::Value>(&spec_json)?;

    Ok(Some(Job {
        id: row.get::<String, _>("id"),
        name: row.get::<String, _>("name"),
        agent_id: row.get::<Option<String>, _>("agent_id"),
        schedule: row.get::<Option<String>, _>("schedule"),
        overlap_policy,
        spec,
        created_at: row.get::<i64, _>("created_at"),
        updated_at: row.get::<i64, _>("updated_at"),
    }))
}

pub async fn list_jobs(db: &SqlitePool) -> Result<Vec<Job>, anyhow::Error> {
    let rows = sqlx::query(
        "SELECT id, name, agent_id, schedule, overlap_policy, spec_json, created_at, updated_at FROM jobs ORDER BY created_at DESC",
    )
    .fetch_all(db)
    .await?;

    let mut jobs = Vec::with_capacity(rows.len());
    for row in rows {
        let overlap_policy = row
            .get::<String, _>("overlap_policy")
            .parse::<OverlapPolicy>()?;
        let spec_json = row.get::<String, _>("spec_json");
        let spec = serde_json::from_str::<serde_json::Value>(&spec_json)?;

        jobs.push(Job {
            id: row.get::<String, _>("id"),
            name: row.get::<String, _>("name"),
            agent_id: row.get::<Option<String>, _>("agent_id"),
            schedule: row.get::<Option<String>, _>("schedule"),
            overlap_policy,
            spec,
            created_at: row.get::<i64, _>("created_at"),
            updated_at: row.get::<i64, _>("updated_at"),
        });
    }

    Ok(jobs)
}

pub async fn update_job(
    db: &SqlitePool,
    job_id: &str,
    name: &str,
    agent_id: Option<&str>,
    schedule: Option<&str>,
    overlap_policy: OverlapPolicy,
    spec: serde_json::Value,
) -> Result<bool, anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let spec_json = serde_json::to_string(&spec)?;

    let result = sqlx::query(
        r#"
        UPDATE jobs
        SET name = ?, agent_id = ?, schedule = ?, overlap_policy = ?, spec_json = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(agent_id)
    .bind(schedule)
    .bind(overlap_policy.as_str())
    .bind(spec_json)
    .bind(now)
    .bind(job_id)
    .execute(db)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_job(db: &SqlitePool, job_id: &str) -> Result<bool, anyhow::Error> {
    let result = sqlx::query("DELETE FROM jobs WHERE id = ?")
        .bind(job_id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;

    use super::{OverlapPolicy, create_job, get_job, list_jobs, update_job};

    #[tokio::test]
    async fn jobs_crud_round_trip() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        let spec = serde_json::json!({ "v": 1, "type": "filesystem" });
        let job = create_job(
            &pool,
            "job1",
            None,
            Some("0 */6 * * *"),
            OverlapPolicy::Queue,
            spec,
        )
        .await
        .expect("create");

        let fetched = get_job(&pool, &job.id)
            .await
            .expect("get")
            .expect("present");
        assert_eq!(fetched.name, "job1");
        assert_eq!(fetched.overlap_policy, OverlapPolicy::Queue);

        let listed = list_jobs(&pool).await.expect("list");
        assert_eq!(listed.len(), 1);

        let updated_spec = serde_json::json!({ "v": 1, "type": "sqlite" });
        let updated = update_job(
            &pool,
            &job.id,
            "job2",
            Some("agent-1"),
            None,
            OverlapPolicy::Reject,
            updated_spec,
        )
        .await
        .expect("update");
        assert!(updated);

        let fetched = get_job(&pool, &job.id)
            .await
            .expect("get2")
            .expect("present2");
        assert_eq!(fetched.name, "job2");
        assert_eq!(fetched.agent_id.as_deref(), Some("agent-1"));
        assert_eq!(fetched.overlap_policy, OverlapPolicy::Reject);
        assert!(fetched.schedule.is_none());
    }
}
