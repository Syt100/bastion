use sqlx::Row;
use sqlx::SqlitePool;

const LOGIN_WINDOW_SECONDS: i64 = 10 * 60;
const LOGIN_MAX_FAILURES: i64 = 10;
const LOGIN_LOCK_SECONDS: i64 = 15 * 60;

pub async fn login_throttle_retry_after_seconds(
    db: &SqlitePool,
    ip: &str,
    now: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let row = sqlx::query("SELECT locked_until FROM login_throttle WHERE ip = ? LIMIT 1")
        .bind(ip)
        .fetch_optional(db)
        .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let locked_until = row.get::<Option<i64>, _>("locked_until");
    let Some(locked_until) = locked_until else {
        return Ok(None);
    };

    if locked_until > now {
        return Ok(Some(locked_until.saturating_sub(now)));
    }

    Ok(None)
}

pub async fn clear_login_throttle(db: &SqlitePool, ip: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM login_throttle WHERE ip = ?")
        .bind(ip)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn record_login_failure(db: &SqlitePool, ip: &str, now: i64) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    let row = sqlx::query(
        "SELECT failures, first_failed_at, locked_until FROM login_throttle WHERE ip = ? LIMIT 1",
    )
    .bind(ip)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(row) = row {
        let failures = row.get::<i64, _>("failures");
        let first_failed_at = row.get::<i64, _>("first_failed_at");
        let locked_until = row.get::<Option<i64>, _>("locked_until");

        if locked_until.is_some_and(|t| t > now) {
            sqlx::query("UPDATE login_throttle SET last_failed_at = ? WHERE ip = ?")
                .bind(now)
                .bind(ip)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            return Ok(());
        }

        let window_expired = now.saturating_sub(first_failed_at) > LOGIN_WINDOW_SECONDS;
        if window_expired {
            sqlx::query(
                "UPDATE login_throttle SET failures = 1, first_failed_at = ?, last_failed_at = ?, locked_until = NULL WHERE ip = ?",
            )
            .bind(now)
            .bind(now)
            .bind(ip)
            .execute(&mut *tx)
            .await?;
        } else {
            let new_failures = failures.saturating_add(1);
            let locked_until = if new_failures >= LOGIN_MAX_FAILURES {
                Some(now.saturating_add(LOGIN_LOCK_SECONDS))
            } else {
                None
            };

            sqlx::query(
                "UPDATE login_throttle SET failures = ?, last_failed_at = ?, locked_until = ? WHERE ip = ?",
            )
            .bind(new_failures)
            .bind(now)
            .bind(locked_until)
            .bind(ip)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        return Ok(());
    }

    sqlx::query(
        "INSERT INTO login_throttle (ip, failures, first_failed_at, last_failed_at, locked_until) VALUES (?, ?, ?, ?, NULL)",
    )
    .bind(ip)
    .bind(1_i64)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;

    use super::{
        LOGIN_LOCK_SECONDS, LOGIN_MAX_FAILURES, clear_login_throttle,
        login_throttle_retry_after_seconds, record_login_failure,
    };

    #[tokio::test]
    async fn login_throttle_locks_after_too_many_failures() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        let ip = "203.0.113.10";
        let now = 1000;

        for _ in 0..LOGIN_MAX_FAILURES {
            record_login_failure(&pool, ip, now).await.expect("record");
        }

        let retry = login_throttle_retry_after_seconds(&pool, ip, now)
            .await
            .expect("retry");
        assert!(retry.is_some());

        let retry = login_throttle_retry_after_seconds(&pool, ip, now + LOGIN_LOCK_SECONDS)
            .await
            .expect("retry2");
        assert!(retry.is_none());

        clear_login_throttle(&pool, ip).await.expect("clear");
        let retry = login_throttle_retry_after_seconds(&pool, ip, now)
            .await
            .expect("retry3");
        assert!(retry.is_none());
    }
}
