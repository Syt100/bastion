use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::settings_repo;

const KEY_HUB_RUNTIME_CONFIG: &str = "hub_runtime_config_v1";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HubRuntimeConfig {
    #[serde(default)]
    pub hub_timezone: Option<String>,
    #[serde(default)]
    pub run_retention_days: Option<i64>,
    #[serde(default)]
    pub incomplete_cleanup_days: Option<i64>,

    #[serde(default)]
    pub log_filter: Option<String>,
    #[serde(default)]
    pub log_file: Option<String>,
    #[serde(default)]
    pub log_rotation: Option<String>,
    #[serde(default)]
    pub log_keep_files: Option<usize>,
}

pub async fn get(db: &SqlitePool) -> Result<Option<HubRuntimeConfig>, anyhow::Error> {
    let json = settings_repo::get_value_json(db, KEY_HUB_RUNTIME_CONFIG).await?;
    let Some(json) = json else {
        return Ok(None);
    };
    let parsed = serde_json::from_str::<HubRuntimeConfig>(&json)?;
    Ok(Some(parsed))
}

pub async fn upsert(db: &SqlitePool, config: &HubRuntimeConfig) -> Result<(), anyhow::Error> {
    let json = serde_json::to_string(config)?;
    settings_repo::upsert_value_json(db, KEY_HUB_RUNTIME_CONFIG, &json).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::db;

    use super::{HubRuntimeConfig, get, upsert};

    #[tokio::test]
    async fn hub_runtime_config_round_trip() {
        let temp = TempDir::new().expect("tempdir");
        let pool = db::init(temp.path()).await.expect("db init");

        assert!(get(&pool).await.unwrap().is_none());

        let cfg = HubRuntimeConfig {
            hub_timezone: Some("Asia/Shanghai".to_string()),
            run_retention_days: Some(30),
            incomplete_cleanup_days: Some(7),
            log_filter: Some("info".to_string()),
            log_file: Some("/tmp/bastion.log".to_string()),
            log_rotation: Some("daily".to_string()),
            log_keep_files: Some(10),
        };
        upsert(&pool, &cfg).await.expect("upsert");

        let loaded = get(&pool).await.unwrap().expect("loaded");
        assert_eq!(loaded.hub_timezone.as_deref(), Some("Asia/Shanghai"));
        assert_eq!(loaded.run_retention_days, Some(30));
        assert_eq!(loaded.incomplete_cleanup_days, Some(7));
        assert_eq!(loaded.log_filter.as_deref(), Some("info"));
        assert_eq!(loaded.log_file.as_deref(), Some("/tmp/bastion.log"));
        assert_eq!(loaded.log_rotation.as_deref(), Some("daily"));
        assert_eq!(loaded.log_keep_files, Some(10));
    }
}

