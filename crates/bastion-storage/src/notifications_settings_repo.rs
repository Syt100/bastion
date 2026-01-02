use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::settings_repo;

const KEY_NOTIFICATIONS: &str = "notifications";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsChannelSettings {
    pub enabled: bool,
}

impl Default for NotificationsChannelSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationsChannels {
    #[serde(default)]
    pub wecom_bot: NotificationsChannelSettings,
    #[serde(default)]
    pub email: NotificationsChannelSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsTemplates {
    pub wecom_markdown: String,
    pub email_subject: String,
    pub email_body: String,
}

impl Default for NotificationsTemplates {
    fn default() -> Self {
        Self {
            wecom_markdown: r#"**{{title}}**
> Job: {{job_name}}
> Run: {{run_id}}
> Started: {{started_at}}
> Ended: {{ended_at}}
{{target_line_wecom}}{{error_line_wecom}}"#
                .to_string(),
            email_subject: "Bastion {{status_text}} - {{job_name}}".to_string(),
            email_body: r#"Bastion backup

Job: {{job_name}}
Run: {{run_id}}
Status: {{status}}
Started: {{started_at}}
Ended: {{ended_at}}
{{target_line_email}}{{error_line_email}}"#
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsSettings {
    pub enabled: bool,
    #[serde(default)]
    pub channels: NotificationsChannels,
    #[serde(default)]
    pub templates: NotificationsTemplates,
}

impl Default for NotificationsSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            channels: NotificationsChannels::default(),
            templates: NotificationsTemplates::default(),
        }
    }
}

pub async fn get_or_default(db: &SqlitePool) -> Result<NotificationsSettings, anyhow::Error> {
    let json = settings_repo::get_value_json(db, KEY_NOTIFICATIONS).await?;
    if let Some(json) = json {
        let parsed = serde_json::from_str::<NotificationsSettings>(&json)?;
        return Ok(parsed);
    }

    let defaults = NotificationsSettings::default();
    let json = serde_json::to_string(&defaults)?;
    settings_repo::upsert_value_json(db, KEY_NOTIFICATIONS, &json).await?;
    Ok(defaults)
}

pub async fn upsert(
    db: &SqlitePool,
    settings: &NotificationsSettings,
) -> Result<(), anyhow::Error> {
    let json = serde_json::to_string(settings)?;
    settings_repo::upsert_value_json(db, KEY_NOTIFICATIONS, &json).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;

    use super::{get_or_default, upsert};

    #[tokio::test]
    async fn default_is_inserted_and_loadable() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        let s1 = get_or_default(&pool).await.expect("get default");
        assert!(s1.enabled);
        assert!(s1.channels.wecom_bot.enabled);
        assert!(s1.channels.email.enabled);

        let mut s2 = s1.clone();
        s2.enabled = false;
        s2.channels.email.enabled = false;
        upsert(&pool, &s2).await.expect("upsert");

        let s3 = get_or_default(&pool).await.expect("load");
        assert!(!s3.enabled);
        assert!(!s3.channels.email.enabled);
    }
}
