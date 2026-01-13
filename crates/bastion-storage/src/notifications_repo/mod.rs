pub const CHANNEL_WECOM_BOT: &str = "wecom_bot";
pub const CHANNEL_EMAIL: &str = "email";

pub const STATUS_QUEUED: &str = "queued";
pub const STATUS_SENDING: &str = "sending";
pub const STATUS_SENT: &str = "sent";
pub const STATUS_FAILED: &str = "failed";
pub const STATUS_CANCELED: &str = "canceled";

#[derive(Debug, Clone)]
pub struct NotificationRow {
    pub id: String,
    pub run_id: String,
    pub channel: String,
    pub secret_name: String,
    pub attempts: i64,
}

#[derive(Debug, Clone)]
pub struct NotificationListItem {
    pub id: String,
    pub run_id: String,
    pub job_id: String,
    pub job_name: String,
    pub channel: String,
    pub secret_name: String,
    pub status: String,
    pub attempts: i64,
    pub next_attempt_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_error: Option<String>,
    pub destination_deleted: bool,
    pub destination_enabled: bool,
}

mod claim;
mod enqueue;
mod queries;
mod transitions;

pub use claim::{claim_next_due, next_due_at};
pub use enqueue::{enqueue_emails_for_run, enqueue_for_run, enqueue_wecom_bots_for_run};
pub use queries::{count_queue, get_notification, list_queue};
pub use transitions::{
    cancel_all_queued, cancel_queued_by_id, cancel_queued_for_channel,
    cancel_queued_for_destination, mark_canceled, mark_failed, mark_sent, reschedule,
    retry_now_by_id,
};

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;

    use super::enqueue_emails_for_run;
    use super::enqueue_wecom_bots_for_run;

    #[tokio::test]
    async fn enqueue_dedupes_per_run_per_bot() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        // Seed two wecom bots.
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        for name in ["a", "b"] {
            sqlx::query(
                "INSERT INTO secrets (id, node_id, kind, name, kid, nonce, ciphertext, created_at, updated_at) VALUES (?, 'hub', 'wecom_bot', ?, 1, X'00', X'00', ?, ?)",
            )
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(name)
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await
            .expect("insert secret");
        }

        // Seed a run row, as notifications has a FK.
        sqlx::query(
            "INSERT INTO jobs (id, name, schedule, overlap_policy, spec_json, created_at, updated_at) VALUES (?, ?, NULL, 'queue', ?, ?, ?)",
        )
        .bind("job1")
        .bind("job1")
        .bind(r#"{"v":1,"type":"filesystem","source":{"root":"/"},"target":{"type":"local_dir","base_dir":"/tmp"}}"#)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert job");

        sqlx::query(
            "INSERT INTO runs (id, job_id, status, started_at, ended_at) VALUES (?, ?, 'success', ?, ?)",
        )
        .bind("run1")
        .bind("job1")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert run");

        let inserted1 = enqueue_wecom_bots_for_run(&pool, "run1").await.unwrap();
        assert_eq!(inserted1, 2);

        let inserted2 = enqueue_wecom_bots_for_run(&pool, "run1").await.unwrap();
        assert_eq!(inserted2, 0);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM notifications")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn enqueue_email_dedupes_per_run_per_destination() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        // Seed two smtp destinations.
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        for name in ["a", "b"] {
            sqlx::query(
                "INSERT INTO secrets (id, node_id, kind, name, kid, nonce, ciphertext, created_at, updated_at) VALUES (?, 'hub', 'smtp', ?, 1, X'00', X'00', ?, ?)",
            )
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(name)
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await
            .expect("insert secret");
        }

        // Seed a run row, as notifications has a FK.
        sqlx::query(
            "INSERT INTO jobs (id, name, schedule, overlap_policy, spec_json, created_at, updated_at) VALUES (?, ?, NULL, 'queue', ?, ?, ?)",
        )
        .bind("job1")
        .bind("job1")
        .bind(r#"{"v":1,"type":"filesystem","source":{"root":"/"},"target":{"type":"local_dir","base_dir":"/tmp"}}"#)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert job");

        sqlx::query(
            "INSERT INTO runs (id, job_id, status, started_at, ended_at) VALUES (?, ?, 'success', ?, ?)",
        )
        .bind("run1")
        .bind("job1")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert run");

        let inserted1 = enqueue_emails_for_run(&pool, "run1").await.unwrap();
        assert_eq!(inserted1, 2);

        let inserted2 = enqueue_emails_for_run(&pool, "run1").await.unwrap();
        assert_eq!(inserted2, 0);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(1) FROM notifications")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn cancel_and_retry_now_change_status() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        // Seed smtp destination.
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        sqlx::query(
            "INSERT INTO secrets (id, node_id, kind, name, kid, nonce, ciphertext, created_at, updated_at) VALUES (?, 'hub', 'smtp', ?, 1, X'00', X'00', ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind("smtp1")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert secret");

        // Seed run + job.
        sqlx::query(
            "INSERT INTO jobs (id, name, schedule, overlap_policy, spec_json, created_at, updated_at) VALUES (?, ?, NULL, 'queue', ?, ?, ?)",
        )
        .bind("job1")
        .bind("job1")
        .bind(r#"{"v":1,"type":"filesystem","source":{"root":"/"},"target":{"type":"local_dir","base_dir":"/tmp"}}"#)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert job");

        sqlx::query(
            "INSERT INTO runs (id, job_id, status, started_at, ended_at) VALUES (?, ?, 'success', ?, ?)",
        )
        .bind("run1")
        .bind("job1")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert run");

        let inserted = enqueue_emails_for_run(&pool, "run1").await.unwrap();
        assert_eq!(inserted, 1);

        let id: String = sqlx::query_scalar("SELECT id FROM notifications LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        let canceled = super::cancel_queued_by_id(&pool, &id, "canceled", now)
            .await
            .unwrap();
        assert!(canceled);

        // Pretend it was failed, then retry-now should work.
        sqlx::query("UPDATE notifications SET status = 'failed' WHERE id = ?")
            .bind(&id)
            .execute(&pool)
            .await
            .unwrap();

        let retried = super::retry_now_by_id(&pool, &id, now).await.unwrap();
        assert!(retried);
        let status: String = sqlx::query_scalar("SELECT status FROM notifications WHERE id = ?")
            .bind(&id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, "queued");
    }

    #[tokio::test]
    async fn list_and_count_support_multi_value_filters() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        // Seed a wecom bot + smtp destination.
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        sqlx::query(
            "INSERT INTO secrets (id, node_id, kind, name, kid, nonce, ciphertext, created_at, updated_at) VALUES (?, 'hub', 'wecom_bot', ?, 1, X'00', X'00', ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind("bot1")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert wecom_bot");

        sqlx::query(
            "INSERT INTO secrets (id, node_id, kind, name, kid, nonce, ciphertext, created_at, updated_at) VALUES (?, 'hub', 'smtp', ?, 1, X'00', X'00', ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind("smtp1")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert smtp");

        // Seed job + run.
        sqlx::query(
            "INSERT INTO jobs (id, name, schedule, overlap_policy, spec_json, created_at, updated_at) VALUES (?, ?, NULL, 'queue', ?, ?, ?)",
        )
        .bind("job1")
        .bind("job1")
        .bind(r#"{"v":1,"type":"filesystem","source":{"root":"/"},"target":{"type":"local_dir","base_dir":"/tmp"}}"#)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert job");

        sqlx::query(
            "INSERT INTO runs (id, job_id, status, started_at, ended_at) VALUES (?, ?, 'success', ?, ?)",
        )
        .bind("run1")
        .bind("job1")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert run");

        let inserted_wecom = super::enqueue_wecom_bots_for_run(&pool, "run1")
            .await
            .expect("enqueue wecom");
        assert_eq!(inserted_wecom, 1);

        let inserted_email = super::enqueue_emails_for_run(&pool, "run1")
            .await
            .expect("enqueue email");
        assert_eq!(inserted_email, 1);

        // Put them in different statuses.
        sqlx::query("UPDATE notifications SET status = 'failed' WHERE channel = 'wecom_bot'")
            .execute(&pool)
            .await
            .expect("set failed");
        sqlx::query("UPDATE notifications SET status = 'sent' WHERE channel = 'email'")
            .execute(&pool)
            .await
            .expect("set sent");

        let total = super::count_queue(&pool, None, None).await.expect("count");
        assert_eq!(total, 2);

        let statuses = vec![
            super::STATUS_FAILED.to_string(),
            super::STATUS_SENT.to_string(),
        ];
        let channels = vec![super::CHANNEL_WECOM_BOT.to_string()];
        let total = super::count_queue(&pool, Some(statuses.as_slice()), Some(channels.as_slice()))
            .await
            .expect("count filtered");
        assert_eq!(total, 1);

        let channels = vec![
            super::CHANNEL_WECOM_BOT.to_string(),
            super::CHANNEL_EMAIL.to_string(),
        ];
        let rows = super::list_queue(
            &pool,
            Some(statuses.as_slice()),
            Some(channels.as_slice()),
            50,
            0,
        )
        .await
        .expect("list filtered");
        assert_eq!(rows.len(), 2);
    }
}
