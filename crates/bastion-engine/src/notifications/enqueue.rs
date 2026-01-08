use std::collections::HashSet;

use sqlx::SqlitePool;

use bastion_core::job_spec;
use bastion_storage::notification_destinations_repo;
use bastion_storage::notifications_repo;
use bastion_storage::notifications_settings_repo;

pub async fn enqueue_for_run_spec(
    db: &SqlitePool,
    spec: &job_spec::JobSpecV1,
    run_id: &str,
) -> Result<bool, anyhow::Error> {
    let settings = notifications_settings_repo::get_or_default(db).await?;
    if !settings.enabled {
        return Ok(false);
    }

    let all = notification_destinations_repo::list_destinations(db).await?;

    let mut enabled_wecom = Vec::new();
    let mut enabled_email = Vec::new();
    for d in &all {
        if !d.enabled {
            continue;
        }
        match d.channel.as_str() {
            notifications_repo::CHANNEL_WECOM_BOT => enabled_wecom.push(d.name.clone()),
            notifications_repo::CHANNEL_EMAIL => enabled_email.push(d.name.clone()),
            _ => {}
        }
    }

    let mut selected_wecom: Vec<String> = Vec::new();
    let mut selected_email: Vec<String> = Vec::new();
    match spec.notifications().mode {
        job_spec::NotificationsModeV1::Inherit => {
            selected_wecom = enabled_wecom;
            selected_email = enabled_email;
        }
        job_spec::NotificationsModeV1::Custom => {
            let wecom_set: HashSet<&str> = enabled_wecom.iter().map(|s| s.as_str()).collect();
            let email_set: HashSet<&str> = enabled_email.iter().map(|s| s.as_str()).collect();

            for name in &spec.notifications().wecom_bot {
                if wecom_set.contains(name.as_str()) {
                    selected_wecom.push(name.clone());
                }
            }
            for name in &spec.notifications().email {
                if email_set.contains(name.as_str()) {
                    selected_email.push(name.clone());
                }
            }
        }
    }

    let mut inserted = 0_i64;
    if settings.channels.wecom_bot.enabled && !selected_wecom.is_empty() {
        inserted += notifications_repo::enqueue_for_run(
            db,
            run_id,
            notifications_repo::CHANNEL_WECOM_BOT,
            &selected_wecom,
        )
        .await?;
    }
    if settings.channels.email.enabled && !selected_email.is_empty() {
        inserted += notifications_repo::enqueue_for_run(
            db,
            run_id,
            notifications_repo::CHANNEL_EMAIL,
            &selected_email,
        )
        .await?;
    }

    Ok(inserted > 0)
}
