use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use tracing::warn;

use bastion_core::agent_protocol::OverlapPolicyV1;

use super::super::cron::cron_matches_minute_cached;
use super::super::storage::OfflineRunWriterHandle;
use super::types::{InFlightCounts, OfflineRunTask};

fn allow_due_for_local_minute(tz: chrono_tz::Tz, local_minute_start: chrono::DateTime<chrono_tz::Tz>) -> bool {
    use chrono::TimeZone as _;
    // DST fold: local wall time occurs twice. Run once by choosing the first occurrence (earlier offset).
    match tz.from_local_datetime(&local_minute_start.naive_local()) {
        chrono::LocalResult::Ambiguous(first, _) => local_minute_start == first,
        _ => true,
    }
}

pub(super) async fn offline_cron_loop(
    data_dir: PathBuf,
    agent_id: String,
    mut connected_rx: tokio::sync::watch::Receiver<bool>,
    tx: tokio::sync::mpsc::UnboundedSender<OfflineRunTask>,
    inflight: std::sync::Arc<tokio::sync::Mutex<InFlightCounts>>,
) {
    use chrono::{DateTime, Duration as ChronoDuration, TimeZone as _, Utc};
    use cron::Schedule;

    let mut schedule_cache: std::collections::HashMap<String, Schedule> =
        std::collections::HashMap::new();
    let mut last_minute = time::OffsetDateTime::now_utc().unix_timestamp() / 60 - 1;

    loop {
        if *connected_rx.borrow() {
            if connected_rx.changed().await.is_err() {
                break;
            }
            continue;
        }

        let now = time::OffsetDateTime::now_utc();
        let now_ts = now.unix_timestamp();
        let now_dt = match DateTime::<Utc>::from_timestamp(now_ts, now.nanosecond()) {
            Some(ts) => ts,
            None => {
                tokio::select! {
                    _ = connected_rx.changed() => {}
                    _ = tokio::time::sleep(Duration::from_secs(5)) => {}
                }
                continue;
            }
        };

        let minute = now_ts / 60;
        let minute_start = match DateTime::<Utc>::from_timestamp(minute * 60, 0) {
            Some(ts) => ts,
            None => {
                tokio::select! {
                    _ = connected_rx.changed() => {}
                    _ = tokio::time::sleep(Duration::from_secs(5)) => {}
                }
                continue;
            }
        };

        if minute != last_minute {
            last_minute = minute;
            match super::super::super::managed::load_managed_config_snapshot(&data_dir, &agent_id) {
                Ok(Some(snapshot)) => {
                    for job in snapshot.jobs {
                        let Some(expr) = job
                            .schedule
                            .as_deref()
                            .map(str::trim)
                            .filter(|v| !v.is_empty())
                        else {
                            continue;
                        };
                        let tz = job
                            .schedule_timezone
                            .as_deref()
                            .unwrap_or("UTC")
                            .parse::<chrono_tz::Tz>();
                        let tz = match tz {
                            Ok(v) => v,
                            Err(_) => {
                                warn!(agent_id = %agent_id, timezone = ?job.schedule_timezone, "invalid schedule timezone; skipping");
                                continue;
                            }
                        };
                        let local_minute_start = tz.from_utc_datetime(&minute_start.naive_utc());
                        if !allow_due_for_local_minute(tz, local_minute_start) {
                            continue;
                        }

                        match cron_matches_minute_cached(expr, local_minute_start, &mut schedule_cache) {
                            Ok(true) => {
                                let should_reject = {
                                    let state = inflight.lock().await;
                                    matches!(job.overlap_policy, OverlapPolicyV1::Reject)
                                        && state.inflight_for_job(&job.job_id) > 0
                                };

                                if should_reject {
                                    if let Err(error) = persist_offline_rejected_run(
                                        &data_dir,
                                        &job.job_id,
                                        &job.name,
                                    )
                                    .await
                                    {
                                        warn!(
                                            agent_id = %agent_id,
                                            job_id = %job.job_id,
                                            error = %error,
                                            "failed to persist offline rejected run"
                                        );
                                    }
                                    continue;
                                }

                                let run_id = uuid::Uuid::new_v4().to_string();
                                let task = OfflineRunTask {
                                    run_id,
                                    job_id: job.job_id,
                                    job_name: job.name,
                                    spec: job.spec,
                                };

                                {
                                    let mut state = inflight.lock().await;
                                    state.inc_job(&task.job_id);
                                }

                                if tx.send(task).is_err() {
                                    break;
                                }
                            }
                            Ok(false) => {}
                            Err(error) => {
                                warn!(agent_id = %agent_id, error = %error, "invalid cron schedule; skipping");
                            }
                        }
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    warn!(agent_id = %agent_id, error = %error, "failed to load managed config snapshot");
                }
            }
        }

        let next_minute = now_dt + ChronoDuration::seconds(60 - (now_dt.timestamp() % 60));
        let sleep_dur = match next_minute.signed_duration_since(now_dt).to_std() {
            Ok(v) => v,
            Err(_) => std::time::Duration::from_secs(1),
        };

        tokio::select! {
            _ = connected_rx.changed() => {}
            _ = tokio::time::sleep(std::time::Duration::from_secs(1).min(sleep_dur)) => {}
        }
    }
}

async fn persist_offline_rejected_run(
    data_dir: &Path,
    job_id: &str,
    job_name: &str,
) -> Result<(), anyhow::Error> {
    let run_id = uuid::Uuid::new_v4().to_string();
    let started_at = time::OffsetDateTime::now_utc().unix_timestamp();
    let writer =
        OfflineRunWriterHandle::start(data_dir, &run_id, job_id, job_name, started_at).await?;
    let _ = writer.append_event(
        "info",
        "rejected",
        "rejected",
        Some(serde_json::json!({ "source": "schedule", "executed_offline": true })),
    );
    writer.finish_rejected().await?;
    Ok(())
}
