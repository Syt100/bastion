use serde::Serialize;
use sqlx::{QueryBuilder, Row, SqlitePool};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BulkOperationStatus {
    Queued,
    Running,
    Done,
    Canceled,
}

impl BulkOperationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Canceled => "canceled",
        }
    }
}

impl std::str::FromStr for BulkOperationStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "done" => Ok(Self::Done),
            "canceled" => Ok(Self::Canceled),
            _ => Err(anyhow::anyhow!("invalid bulk operation status")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BulkOperationItemStatus {
    Queued,
    Running,
    Success,
    Failed,
    Canceled,
}

impl BulkOperationItemStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
        }
    }
}

impl std::str::FromStr for BulkOperationItemStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "success" => Ok(Self::Success),
            "failed" => Ok(Self::Failed),
            "canceled" => Ok(Self::Canceled),
            _ => Err(anyhow::anyhow!("invalid bulk operation item status")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkOperationListItem {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub canceled_at: Option<i64>,
    pub total: i64,
    pub queued: i64,
    pub running: i64,
    pub success: i64,
    pub failed: i64,
    pub canceled: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkOperationDetail {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub created_by_user_id: Option<i64>,
    pub selector: serde_json::Value,
    pub payload: serde_json::Value,
    pub created_at: i64,
    pub updated_at: i64,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub canceled_at: Option<i64>,
    pub total: i64,
    pub queued: i64,
    pub running: i64,
    pub success: i64,
    pub failed: i64,
    pub canceled: i64,
    pub items: Vec<BulkOperationItemDetail>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkOperationItemDetail {
    pub op_id: String,
    pub agent_id: String,
    pub agent_name: Option<String>,
    pub status: String,
    pub attempts: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub last_error_kind: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClaimedBulkOperationItem {
    pub op_id: String,
    pub kind: String,
    pub agent_id: String,
    pub payload_json: String,
}

pub async fn create_operation(
    db: &SqlitePool,
    created_by_user_id: i64,
    kind: &str,
    selector: &serde_json::Value,
    payload: &serde_json::Value,
    agent_ids: &[String],
) -> Result<String, anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let id = Uuid::new_v4().to_string();

    let mut tx = db.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO bulk_operations (
          id, kind, status, created_by_user_id, selector_json, payload_json,
          created_at, updated_at, started_at, ended_at, canceled_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL)
        "#,
    )
    .bind(&id)
    .bind(kind)
    .bind(BulkOperationStatus::Queued.as_str())
    .bind(created_by_user_id)
    .bind(selector.to_string())
    .bind(payload.to_string())
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    for agent_id in agent_ids {
        sqlx::query(
            r#"
            INSERT INTO bulk_operation_items (
              op_id, agent_id, status, attempts,
              created_at, updated_at, started_at, ended_at, last_error_kind, last_error
            )
            VALUES (?, ?, ?, 0, ?, ?, NULL, NULL, NULL, NULL)
            "#,
        )
        .bind(&id)
        .bind(agent_id)
        .bind(BulkOperationItemStatus::Queued.as_str())
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(id)
}

pub async fn list_operations(
    db: &SqlitePool,
    limit: i64,
) -> Result<Vec<BulkOperationListItem>, anyhow::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
          o.id, o.kind, o.status, o.created_at, o.updated_at, o.started_at, o.ended_at, o.canceled_at,
          COUNT(i.agent_id) AS total,
          SUM(CASE WHEN i.status = 'queued' THEN 1 ELSE 0 END) AS queued,
          SUM(CASE WHEN i.status = 'running' THEN 1 ELSE 0 END) AS running,
          SUM(CASE WHEN i.status = 'success' THEN 1 ELSE 0 END) AS success,
          SUM(CASE WHEN i.status = 'failed' THEN 1 ELSE 0 END) AS failed,
          SUM(CASE WHEN i.status = 'canceled' THEN 1 ELSE 0 END) AS canceled
        FROM bulk_operations o
        LEFT JOIN bulk_operation_items i ON i.op_id = o.id
        GROUP BY o.id
        ORDER BY o.created_at DESC
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| BulkOperationListItem {
            id: r.get::<String, _>("id"),
            kind: r.get::<String, _>("kind"),
            status: r.get::<String, _>("status"),
            created_at: r.get::<i64, _>("created_at"),
            updated_at: r.get::<i64, _>("updated_at"),
            started_at: r.get::<Option<i64>, _>("started_at"),
            ended_at: r.get::<Option<i64>, _>("ended_at"),
            canceled_at: r.get::<Option<i64>, _>("canceled_at"),
            total: r.get::<i64, _>("total"),
            queued: r.get::<i64, _>("queued"),
            running: r.get::<i64, _>("running"),
            success: r.get::<i64, _>("success"),
            failed: r.get::<i64, _>("failed"),
            canceled: r.get::<i64, _>("canceled"),
        })
        .collect())
}

pub async fn get_operation(
    db: &SqlitePool,
    op_id: &str,
) -> Result<Option<BulkOperationDetail>, anyhow::Error> {
    let row = sqlx::query(
        r#"
        SELECT
          o.id, o.kind, o.status, o.created_by_user_id, o.selector_json, o.payload_json,
          o.created_at, o.updated_at, o.started_at, o.ended_at, o.canceled_at,
          COUNT(i.agent_id) AS total,
          SUM(CASE WHEN i.status = 'queued' THEN 1 ELSE 0 END) AS queued,
          SUM(CASE WHEN i.status = 'running' THEN 1 ELSE 0 END) AS running,
          SUM(CASE WHEN i.status = 'success' THEN 1 ELSE 0 END) AS success,
          SUM(CASE WHEN i.status = 'failed' THEN 1 ELSE 0 END) AS failed,
          SUM(CASE WHEN i.status = 'canceled' THEN 1 ELSE 0 END) AS canceled
        FROM bulk_operations o
        LEFT JOIN bulk_operation_items i ON i.op_id = o.id
        WHERE o.id = ?
        GROUP BY o.id
        LIMIT 1
        "#,
    )
    .bind(op_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let selector_json = row.get::<String, _>("selector_json");
    let payload_json = row.get::<String, _>("payload_json");
    let selector = serde_json::from_str(&selector_json).unwrap_or(serde_json::Value::Null);
    let payload = serde_json::from_str(&payload_json).unwrap_or(serde_json::Value::Null);

    let items_rows = sqlx::query(
        r#"
        SELECT
          i.op_id, i.agent_id, a.name AS agent_name, i.status, i.attempts,
          i.created_at, i.updated_at, i.started_at, i.ended_at, i.last_error_kind, i.last_error
        FROM bulk_operation_items i
        LEFT JOIN agents a ON a.id = i.agent_id
        WHERE i.op_id = ?
        ORDER BY i.agent_id ASC
        "#,
    )
    .bind(op_id)
    .fetch_all(db)
    .await?;

    let items = items_rows
        .into_iter()
        .map(|r| BulkOperationItemDetail {
            op_id: r.get::<String, _>("op_id"),
            agent_id: r.get::<String, _>("agent_id"),
            agent_name: r.get::<Option<String>, _>("agent_name"),
            status: r.get::<String, _>("status"),
            attempts: r.get::<i64, _>("attempts"),
            created_at: r.get::<i64, _>("created_at"),
            updated_at: r.get::<i64, _>("updated_at"),
            started_at: r.get::<Option<i64>, _>("started_at"),
            ended_at: r.get::<Option<i64>, _>("ended_at"),
            last_error_kind: r.get::<Option<String>, _>("last_error_kind"),
            last_error: r.get::<Option<String>, _>("last_error"),
        })
        .collect();

    Ok(Some(BulkOperationDetail {
        id: row.get::<String, _>("id"),
        kind: row.get::<String, _>("kind"),
        status: row.get::<String, _>("status"),
        created_by_user_id: row.get::<Option<i64>, _>("created_by_user_id"),
        selector,
        payload,
        created_at: row.get::<i64, _>("created_at"),
        updated_at: row.get::<i64, _>("updated_at"),
        started_at: row.get::<Option<i64>, _>("started_at"),
        ended_at: row.get::<Option<i64>, _>("ended_at"),
        canceled_at: row.get::<Option<i64>, _>("canceled_at"),
        total: row.get::<i64, _>("total"),
        queued: row.get::<i64, _>("queued"),
        running: row.get::<i64, _>("running"),
        success: row.get::<i64, _>("success"),
        failed: row.get::<i64, _>("failed"),
        canceled: row.get::<i64, _>("canceled"),
        items,
    }))
}

pub async fn cancel_operation(db: &SqlitePool, op_id: &str) -> Result<(), anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let mut tx = db.begin().await?;

    sqlx::query(
        "UPDATE bulk_operations SET status = 'canceled', canceled_at = COALESCE(canceled_at, ?), updated_at = ? WHERE id = ?",
    )
    .bind(now)
    .bind(now)
    .bind(op_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE bulk_operation_items SET status = 'canceled', ended_at = COALESCE(ended_at, ?), updated_at = ? WHERE op_id = ? AND status = 'queued'",
    )
    .bind(now)
    .bind(now)
    .bind(op_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn retry_failed(db: &SqlitePool, op_id: &str) -> Result<i64, anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let mut tx = db.begin().await?;

    let row = sqlx::query("SELECT status FROM bulk_operations WHERE id = ? LIMIT 1")
        .bind(op_id)
        .fetch_optional(&mut *tx)
        .await?;
    let Some(row) = row else {
        return Ok(0);
    };
    let status = row.get::<String, _>("status");
    if status == BulkOperationStatus::Canceled.as_str() {
        return Err(anyhow::anyhow!("bulk operation is canceled"));
    }

    let result = sqlx::query(
        r#"
        UPDATE bulk_operation_items
        SET status = 'queued',
            updated_at = ?,
            started_at = NULL,
            ended_at = NULL,
            last_error_kind = NULL,
            last_error = NULL
        WHERE op_id = ? AND status = 'failed'
        "#,
    )
    .bind(now)
    .bind(op_id)
    .execute(&mut *tx)
    .await?;
    let changed = result.rows_affected() as i64;

    if changed > 0 {
        sqlx::query(
            "UPDATE bulk_operations SET status = 'queued', ended_at = NULL, updated_at = ? WHERE id = ? AND status != 'canceled'",
        )
        .bind(now)
        .bind(op_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(changed)
}

pub async fn claim_next_items(
    db: &SqlitePool,
    limit: i64,
) -> Result<Vec<ClaimedBulkOperationItem>, anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let mut tx = db.begin().await?;

    let candidates = sqlx::query(
        r#"
        SELECT i.op_id, i.agent_id, o.kind, o.payload_json
        FROM bulk_operation_items i
        JOIN bulk_operations o ON o.id = i.op_id
        WHERE i.status = 'queued' AND o.status != 'canceled'
        ORDER BY o.created_at ASC, i.updated_at ASC
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(&mut *tx)
    .await?;

    let mut claimed = Vec::new();
    for row in candidates {
        let op_id = row.get::<String, _>("op_id");
        let agent_id = row.get::<String, _>("agent_id");
        let kind = row.get::<String, _>("kind");
        let payload_json = row.get::<String, _>("payload_json");

        let result = sqlx::query(
            r#"
            UPDATE bulk_operation_items
            SET status = 'running',
                attempts = attempts + 1,
                started_at = COALESCE(started_at, ?),
                updated_at = ?
            WHERE op_id = ? AND agent_id = ? AND status = 'queued'
            "#,
        )
        .bind(now)
        .bind(now)
        .bind(&op_id)
        .bind(&agent_id)
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            continue;
        }

        sqlx::query(
            r#"
            UPDATE bulk_operations
            SET status = 'running',
                started_at = COALESCE(started_at, ?),
                updated_at = ?
            WHERE id = ? AND status != 'canceled'
            "#,
        )
        .bind(now)
        .bind(now)
        .bind(&op_id)
        .execute(&mut *tx)
        .await?;

        claimed.push(ClaimedBulkOperationItem {
            op_id,
            kind,
            agent_id,
            payload_json,
        });
    }

    tx.commit().await?;
    Ok(claimed)
}

pub async fn mark_item_succeeded(
    db: &SqlitePool,
    op_id: &str,
    agent_id: &str,
) -> Result<(), anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let mut tx = db.begin().await?;

    sqlx::query(
        r#"
        UPDATE bulk_operation_items
        SET status = 'success',
            ended_at = COALESCE(ended_at, ?),
            updated_at = ?,
            last_error_kind = NULL,
            last_error = NULL
        WHERE op_id = ? AND agent_id = ?
        "#,
    )
    .bind(now)
    .bind(now)
    .bind(op_id)
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE bulk_operations SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(op_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    finalize_operation_if_done(db, op_id).await?;
    Ok(())
}

pub async fn mark_item_succeeded_with_note(
    db: &SqlitePool,
    op_id: &str,
    agent_id: &str,
    note_kind: &str,
    note: &str,
) -> Result<(), anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let mut tx = db.begin().await?;

    sqlx::query(
        r#"
        UPDATE bulk_operation_items
        SET status = 'success',
            ended_at = COALESCE(ended_at, ?),
            updated_at = ?,
            last_error_kind = ?,
            last_error = ?
        WHERE op_id = ? AND agent_id = ?
        "#,
    )
    .bind(now)
    .bind(now)
    .bind(note_kind)
    .bind(note)
    .bind(op_id)
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE bulk_operations SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(op_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    finalize_operation_if_done(db, op_id).await?;
    Ok(())
}

pub async fn mark_item_failed(
    db: &SqlitePool,
    op_id: &str,
    agent_id: &str,
    error_kind: &str,
    error: &str,
) -> Result<(), anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let mut tx = db.begin().await?;

    sqlx::query(
        r#"
        UPDATE bulk_operation_items
        SET status = 'failed',
            ended_at = COALESCE(ended_at, ?),
            updated_at = ?,
            last_error_kind = ?,
            last_error = ?
        WHERE op_id = ? AND agent_id = ?
        "#,
    )
    .bind(now)
    .bind(now)
    .bind(error_kind)
    .bind(error)
    .bind(op_id)
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE bulk_operations SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(op_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    finalize_operation_if_done(db, op_id).await?;
    Ok(())
}

pub async fn finalize_operation_if_done(db: &SqlitePool, op_id: &str) -> Result<(), anyhow::Error> {
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let row = sqlx::query(
        r#"
        SELECT
          SUM(CASE WHEN status IN ('queued', 'running') THEN 1 ELSE 0 END) AS active
        FROM bulk_operation_items
        WHERE op_id = ?
        "#,
    )
    .bind(op_id)
    .fetch_one(db)
    .await?;
    let active = row.get::<Option<i64>, _>("active").unwrap_or(0);
    if active != 0 {
        return Ok(());
    }

    let row = sqlx::query("SELECT status FROM bulk_operations WHERE id = ? LIMIT 1")
        .bind(op_id)
        .fetch_optional(db)
        .await?;
    let Some(row) = row else {
        return Ok(());
    };
    let status = row.get::<String, _>("status");
    if status == BulkOperationStatus::Canceled.as_str() {
        sqlx::query(
            "UPDATE bulk_operations SET ended_at = COALESCE(ended_at, ?), updated_at = ? WHERE id = ?",
        )
        .bind(now)
        .bind(now)
        .bind(op_id)
        .execute(db)
        .await?;
        return Ok(());
    }

    sqlx::query(
        "UPDATE bulk_operations SET status = 'done', ended_at = COALESCE(ended_at, ?), updated_at = ? WHERE id = ? AND status != 'canceled'",
    )
    .bind(now)
    .bind(now)
    .bind(op_id)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn resolve_agent_ids_by_selector_labels(
    db: &SqlitePool,
    labels: &[String],
    mode: &str,
) -> Result<Vec<String>, anyhow::Error> {
    if labels.is_empty() {
        return Ok(Vec::new());
    }

    let mut qb: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
        "SELECT DISTINCT al.agent_id FROM agent_labels al WHERE al.agent_id IN (",
    );

    match mode {
        "and" => {
            qb.push("SELECT al2.agent_id FROM agent_labels al2 WHERE al2.label IN (");
            let mut separated = qb.separated(", ");
            for label in labels {
                separated.push_bind(label);
            }
            separated.push_unseparated(")");
            qb.push(" GROUP BY al2.agent_id HAVING COUNT(DISTINCT al2.label) = ");
            qb.push_bind(labels.len() as i64);
        }
        _ => {
            qb.push("SELECT al2.agent_id FROM agent_labels al2 WHERE al2.label IN (");
            let mut separated = qb.separated(", ");
            for label in labels {
                separated.push_bind(label);
            }
            separated.push_unseparated(")");
        }
    }

    qb.push(")");

    let rows = qb.build().fetch_all(db).await?;
    Ok(rows
        .into_iter()
        .map(|r| r.get::<String, _>("agent_id"))
        .collect())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::{auth, db};

    use super::{
        BulkOperationItemStatus, BulkOperationStatus, cancel_operation, create_operation,
        get_operation, retry_failed,
    };

    #[tokio::test]
    async fn retry_failed_only_requeues_failed_items() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        auth::create_user(&pool, "admin", "pw")
            .await
            .expect("create user");
        let user = auth::find_user_by_username(&pool, "admin")
            .await
            .expect("find user")
            .expect("user exists");

        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        for id in ["a", "b"] {
            sqlx::query(
                "INSERT INTO agents (id, name, key_hash, created_at) VALUES (?, NULL, ?, ?)",
            )
            .bind(id)
            .bind(vec![0u8; 32])
            .bind(now)
            .execute(&pool)
            .await
            .expect("insert agent");
        }

        let agent_ids = vec!["a".to_string(), "b".to_string()];
        let op_id = create_operation(
            &pool,
            user.id,
            "agent_labels_add",
            &serde_json::json!({ "node_ids": ["a", "b"] }),
            &serde_json::json!({ "labels": ["prod"] }),
            &agent_ids,
        )
        .await
        .expect("create op");

        sqlx::query("UPDATE bulk_operation_items SET status = ? WHERE op_id = ? AND agent_id = ?")
            .bind(BulkOperationItemStatus::Success.as_str())
            .bind(&op_id)
            .bind("a")
            .execute(&pool)
            .await
            .expect("mark success");
        sqlx::query("UPDATE bulk_operation_items SET status = ? WHERE op_id = ? AND agent_id = ?")
            .bind(BulkOperationItemStatus::Failed.as_str())
            .bind(&op_id)
            .bind("b")
            .execute(&pool)
            .await
            .expect("mark failed");
        sqlx::query("UPDATE bulk_operations SET status = ? WHERE id = ?")
            .bind(BulkOperationStatus::Done.as_str())
            .bind(&op_id)
            .execute(&pool)
            .await
            .expect("mark done");

        let changed = retry_failed(&pool, &op_id).await.expect("retry_failed");
        assert_eq!(changed, 1);

        let op = get_operation(&pool, &op_id).await.unwrap().unwrap();
        let mut by_agent: std::collections::HashMap<String, String> = op
            .items
            .into_iter()
            .map(|i| (i.agent_id, i.status))
            .collect();
        assert_eq!(
            by_agent.remove("a").unwrap(),
            BulkOperationItemStatus::Success.as_str()
        );
        assert_eq!(
            by_agent.remove("b").unwrap(),
            BulkOperationItemStatus::Queued.as_str()
        );
        assert_eq!(op.status, BulkOperationStatus::Queued.as_str());
    }

    #[tokio::test]
    async fn cancel_marks_queued_items_canceled() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        auth::create_user(&pool, "admin", "pw")
            .await
            .expect("create user");
        let user = auth::find_user_by_username(&pool, "admin")
            .await
            .expect("find user")
            .expect("user exists");

        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        for id in ["a", "b"] {
            sqlx::query(
                "INSERT INTO agents (id, name, key_hash, created_at) VALUES (?, NULL, ?, ?)",
            )
            .bind(id)
            .bind(vec![0u8; 32])
            .bind(now)
            .execute(&pool)
            .await
            .expect("insert agent");
        }

        let agent_ids = vec!["a".to_string(), "b".to_string()];
        let op_id = create_operation(
            &pool,
            user.id,
            "agent_labels_add",
            &serde_json::json!({ "node_ids": ["a", "b"] }),
            &serde_json::json!({ "labels": ["prod"] }),
            &agent_ids,
        )
        .await
        .expect("create op");

        sqlx::query(
            "UPDATE bulk_operation_items SET status = 'running' WHERE op_id = ? AND agent_id = ?",
        )
        .bind(&op_id)
        .bind("a")
        .execute(&pool)
        .await
        .expect("mark running");

        cancel_operation(&pool, &op_id).await.expect("cancel");

        let op = get_operation(&pool, &op_id).await.unwrap().unwrap();
        let mut by_agent: std::collections::HashMap<String, String> = op
            .items
            .into_iter()
            .map(|i| (i.agent_id, i.status))
            .collect();
        assert_eq!(by_agent.remove("a").unwrap(), "running");
        assert_eq!(by_agent.remove("b").unwrap(), "canceled");
        assert_eq!(op.status, BulkOperationStatus::Canceled.as_str());
    }
}
