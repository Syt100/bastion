use sqlx::SqlitePool;
use time::OffsetDateTime;
use tracing::{info, warn};

const LOGIN_THROTTLE_RETENTION_DAYS: i64 = 30;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DbPruneStats {
    pub sessions_deleted: u64,
    pub enrollment_tokens_deleted: u64,
    pub login_throttle_deleted: u64,
}

pub fn spawn(db: SqlitePool) {
    tokio::spawn(run_loop(db));
}

async fn run_loop(db: SqlitePool) {
    loop {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        match prune_with_now(&db, now).await {
            Ok(stats) => {
                let total = stats
                    .sessions_deleted
                    .saturating_add(stats.enrollment_tokens_deleted)
                    .saturating_add(stats.login_throttle_deleted);
                if total > 0 {
                    info!(
                        sessions_deleted = stats.sessions_deleted,
                        enrollment_tokens_deleted = stats.enrollment_tokens_deleted,
                        login_throttle_deleted = stats.login_throttle_deleted,
                        "database maintenance pruned rows"
                    );
                }
            }
            Err(error) => {
                warn!(error = %error, "database maintenance failed");
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await;
    }
}

async fn prune_with_now(db: &SqlitePool, now: i64) -> Result<DbPruneStats, anyhow::Error> {
    let mut stats = DbPruneStats::default();

    let result = sqlx::query("DELETE FROM sessions WHERE expires_at < ?")
        .bind(now)
        .execute(db)
        .await?;
    stats.sessions_deleted = result.rows_affected();

    let result = sqlx::query("DELETE FROM enrollment_tokens WHERE expires_at < ?")
        .bind(now)
        .execute(db)
        .await?;
    stats.enrollment_tokens_deleted = result.rows_affected();

    let cutoff = now.saturating_sub(LOGIN_THROTTLE_RETENTION_DAYS.saturating_mul(24 * 60 * 60));
    let result = sqlx::query("DELETE FROM login_throttle WHERE last_failed_at < ?")
        .bind(cutoff)
        .execute(db)
        .await?;
    stats.login_throttle_deleted = result.rows_affected();

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use sqlx::Row;

    use crate::auth;
    use crate::db;

    use super::prune_with_now;

    #[tokio::test]
    async fn db_maintenance_prunes_expired_rows() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        let now = 10_000_000_i64;

        auth::create_user(&pool, "admin", "pw")
            .await
            .expect("create user");
        let user = auth::find_user_by_username(&pool, "admin")
            .await
            .expect("find user")
            .expect("user exists");
        let session = auth::create_session(&pool, user.id)
            .await
            .expect("create session");

        // Expire the session.
        sqlx::query("UPDATE sessions SET expires_at = ? WHERE id = ?")
            .bind(now - 1)
            .bind(&session.id)
            .execute(&pool)
            .await
            .expect("expire session");

        // Expired enrollment token.
        sqlx::query(
            "INSERT INTO enrollment_tokens (token_hash, created_at, expires_at, remaining_uses) VALUES (?, ?, ?, ?)",
        )
        .bind(vec![1_u8, 2, 3])
        .bind(now - 10)
        .bind(now - 1)
        .bind(None::<i64>)
        .execute(&pool)
        .await
        .expect("insert token");

        let old = now
            .saturating_sub(super::LOGIN_THROTTLE_RETENTION_DAYS.saturating_mul(24 * 60 * 60))
            .saturating_sub(1);

        // Old login throttle row.
        sqlx::query(
            "INSERT INTO login_throttle (ip, failures, first_failed_at, last_failed_at, locked_until) VALUES (?, ?, ?, ?, ?)",
        )
        .bind("1.2.3.4")
        .bind(1_i64)
        .bind(old)
        .bind(old)
        .bind(None::<i64>)
        .execute(&pool)
        .await
        .expect("insert login throttle");

        let stats = prune_with_now(&pool, now).await.expect("prune");
        assert_eq!(stats.sessions_deleted, 1);
        assert_eq!(stats.enrollment_tokens_deleted, 1);
        assert_eq!(stats.login_throttle_deleted, 1);
    }

    #[tokio::test]
    async fn runs_indexes_exist_after_migrations() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        let rows = sqlx::query("PRAGMA index_list('runs')")
            .fetch_all(&pool)
            .await
            .expect("index_list");
        let names: std::collections::HashSet<String> =
            rows.iter().map(|r| r.get::<String, _>("name")).collect();

        assert!(names.contains("idx_runs_status_started_at"));
        assert!(names.contains("idx_runs_ended_at"));
    }
}
