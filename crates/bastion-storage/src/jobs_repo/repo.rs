use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;
use uuid::Uuid;

use super::types::{Job, OverlapPolicy};

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

pub async fn list_jobs_for_agent(
    db: &SqlitePool,
    agent_id: &str,
) -> Result<Vec<Job>, anyhow::Error> {
    let rows = sqlx::query(
        "SELECT id, name, agent_id, schedule, overlap_policy, spec_json, created_at, updated_at FROM jobs WHERE agent_id = ? ORDER BY created_at DESC",
    )
    .bind(agent_id)
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
