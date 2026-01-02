use sqlx::Row;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::notifications_repo;

pub const SECRET_KIND_WECOM_BOT: &str = "wecom_bot";
pub const SECRET_KIND_SMTP: &str = "smtp";

#[derive(Debug, Clone)]
pub struct NotificationDestinationListItem {
    pub channel: String,
    pub name: String,
    pub enabled: bool,
    pub updated_at: i64,
}

pub fn secret_kind_for_channel(channel: &str) -> Option<&'static str> {
    match channel {
        notifications_repo::CHANNEL_WECOM_BOT => Some(SECRET_KIND_WECOM_BOT),
        notifications_repo::CHANNEL_EMAIL => Some(SECRET_KIND_SMTP),
        _ => None,
    }
}

pub fn channel_for_secret_kind(kind: &str) -> Option<&'static str> {
    match kind {
        SECRET_KIND_WECOM_BOT => Some(notifications_repo::CHANNEL_WECOM_BOT),
        SECRET_KIND_SMTP => Some(notifications_repo::CHANNEL_EMAIL),
        _ => None,
    }
}

pub async fn is_enabled(
    db: &SqlitePool,
    channel: &str,
    secret_name: &str,
) -> Result<bool, anyhow::Error> {
    let Some(secret_kind) = secret_kind_for_channel(channel) else {
        return Ok(false);
    };

    let row = sqlx::query(
        "SELECT enabled FROM notification_destinations WHERE secret_kind = ? AND secret_name = ? LIMIT 1",
    )
    .bind(secret_kind)
    .bind(secret_name)
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| r.get::<i64, _>("enabled") != 0).unwrap_or(true))
}

pub async fn set_enabled(
    db: &SqlitePool,
    channel: &str,
    secret_name: &str,
    enabled: bool,
) -> Result<(), anyhow::Error> {
    let Some(secret_kind) = secret_kind_for_channel(channel) else {
        anyhow::bail!("unsupported notification channel: {channel}");
    };

    let now = OffsetDateTime::now_utc().unix_timestamp();
    sqlx::query(
        r#"
        INSERT INTO notification_destinations (secret_kind, secret_name, enabled, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(secret_kind, secret_name) DO UPDATE SET enabled = excluded.enabled, updated_at = excluded.updated_at
        "#,
    )
    .bind(secret_kind)
    .bind(secret_name)
    .bind(if enabled { 1 } else { 0 })
    .bind(now)
    .bind(now)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn list_destinations(
    db: &SqlitePool,
) -> Result<Vec<NotificationDestinationListItem>, anyhow::Error> {
    let rows = sqlx::query(
        r#"
        SELECT 'wecom_bot' AS channel, s.name AS name, COALESCE(d.enabled, 1) AS enabled, s.updated_at AS updated_at
          FROM secrets s
          LEFT JOIN notification_destinations d ON d.secret_kind = s.kind AND d.secret_name = s.name
         WHERE s.kind = 'wecom_bot'
        UNION ALL
        SELECT 'email' AS channel, s.name AS name, COALESCE(d.enabled, 1) AS enabled, s.updated_at AS updated_at
          FROM secrets s
          LEFT JOIN notification_destinations d ON d.secret_kind = s.kind AND d.secret_name = s.name
         WHERE s.kind = 'smtp'
         ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(db)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(NotificationDestinationListItem {
            channel: row.get::<String, _>("channel"),
            name: row.get::<String, _>("name"),
            enabled: row.get::<i64, _>("enabled") != 0,
            updated_at: row.get::<i64, _>("updated_at"),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;
    use crate::notifications_repo::{CHANNEL_EMAIL, CHANNEL_WECOM_BOT};
    use crate::secrets::SecretsCrypto;
    use crate::secrets_repo;

    use super::{is_enabled, list_destinations, set_enabled};

    #[tokio::test]
    async fn destinations_default_enabled_and_toggleable() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");
        let crypto = SecretsCrypto::load_or_create(temp.path()).expect("crypto");

        // Seed one wecom and one smtp secret (destination).
        secrets_repo::upsert_secret(&pool, &crypto, "wecom_bot", "w1", b"{}")
            .await
            .expect("upsert wecom");
        secrets_repo::upsert_secret(&pool, &crypto, "smtp", "s1", b"{}")
            .await
            .expect("upsert smtp");

        assert!(is_enabled(&pool, CHANNEL_WECOM_BOT, "w1").await.unwrap());
        assert!(is_enabled(&pool, CHANNEL_EMAIL, "s1").await.unwrap());

        set_enabled(&pool, CHANNEL_WECOM_BOT, "w1", false)
            .await
            .expect("disable");
        assert!(!is_enabled(&pool, CHANNEL_WECOM_BOT, "w1").await.unwrap());

        let list = list_destinations(&pool).await.unwrap();
        assert_eq!(list.len(), 2);
        let w1 = list.iter().find(|x| x.channel == "wecom_bot" && x.name == "w1");
        assert_eq!(w1.map(|x| x.enabled), Some(false));
    }
}
