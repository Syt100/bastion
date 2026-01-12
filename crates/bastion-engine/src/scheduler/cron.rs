use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use cron::Schedule;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

use bastion_storage::jobs_repo;

use crate::agent_manager::AgentManager;
use crate::run_events_bus::RunEventsBus;

use super::queue::enqueue_run;

fn normalize_cron(expr: &str) -> Result<String, anyhow::Error> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    match parts.len() {
        5 => Ok(format!("0 {}", parts.join(" "))),
        6 => {
            if parts[0] != "0" {
                anyhow::bail!("cron seconds must be 0 for minute-based scheduling");
            }
            Ok(parts.join(" "))
        }
        _ => Err(anyhow::anyhow!("invalid cron expression")),
    }
}

pub(super) fn validate_cron(expr: &str) -> Result<(), anyhow::Error> {
    let expr = normalize_cron(expr)?;
    let _ = Schedule::from_str(&expr)?;
    Ok(())
}

pub(super) async fn run_cron_loop(
    db: SqlitePool,
    run_events_bus: Arc<RunEventsBus>,
    run_queue_notify: Arc<Notify>,
    jobs_notify: Arc<Notify>,
    agent_manager: AgentManager,
    shutdown: CancellationToken,
) {
    let mut schedule_cache: HashMap<String, Schedule> = HashMap::new();
    let mut last_minute = OffsetDateTime::now_utc().unix_timestamp() / 60 - 1;
    let mut should_evaluate_due = true;

    enum WakeReason {
        Timer,
        Jobs,
    }

    loop {
        if shutdown.is_cancelled() {
            break;
        }

        let now = OffsetDateTime::now_utc();
        let now_ts = now.unix_timestamp();
        let now_dt = match DateTime::<Utc>::from_timestamp(now_ts, now.nanosecond()) {
            Some(ts) => ts,
            None => {
                warn!("invalid timestamp for scheduler now");
                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    _ = jobs_notify.notified() => {}
                    _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
                }
                continue;
            }
        };

        let minute = now_ts / 60;
        let minute_start = match DateTime::<Utc>::from_timestamp(minute * 60, 0) {
            Some(ts) => ts,
            None => {
                warn!("invalid timestamp for scheduler minute_start");
                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    _ = jobs_notify.notified() => {}
                    _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
                }
                continue;
            }
        };

        let jobs = match jobs_repo::list_jobs(&db).await {
            Ok(v) => v,
            Err(error) => {
                warn!(error = %error, "failed to list jobs for scheduler");
                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    _ = jobs_notify.notified() => {
                        should_evaluate_due = false;
                        continue;
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                        should_evaluate_due = true;
                        continue;
                    }
                }
            }
        };

        if should_evaluate_due {
            if minute != last_minute {
                last_minute = minute;
                for job in &jobs {
                    let Some(expr) = job.schedule.as_deref() else {
                        continue;
                    };

                    if let Some(agent_id) = job.agent_id.as_deref()
                        && !agent_manager.is_connected(agent_id).await
                    {
                        debug!(
                            job_id = %job.id,
                            agent_id = %agent_id,
                            "agent offline; skip hub scheduling"
                        );
                        continue;
                    }

                    match cron_matches_minute_cached(expr, minute_start, &mut schedule_cache) {
                        Ok(true) => {
                            debug!(job_id = %job.id, "cron due; enqueue run");
                            if let Err(error) = enqueue_run(
                                &db,
                                run_events_bus.as_ref(),
                                run_queue_notify.as_ref(),
                                job,
                                "schedule",
                            )
                            .await
                            {
                                warn!(
                                    job_id = %job.id,
                                    error = %error,
                                    "failed to enqueue scheduled run"
                                );
                            }
                        }
                        Ok(false) => {}
                        Err(error) => {
                            warn!(
                                job_id = %job.id,
                                error = %error,
                                "invalid cron schedule; skipping"
                            );
                        }
                    }
                }
            }
        } else {
            // A jobs change can arrive mid-minute; do not enqueue for a minute_start that already passed.
            last_minute = minute;
        }

        let next_due = next_cron_due_after_cached(&jobs, now_dt, &mut schedule_cache);

        let reason = match next_due {
            Some(next_due) => {
                let sleep_dur = match next_due.signed_duration_since(now_dt).to_std() {
                    Ok(v) => v,
                    Err(_) => std::time::Duration::from_secs(60),
                };
                let deadline = tokio::time::Instant::now() + sleep_dur;
                tokio::select! {
                    _ = shutdown.cancelled() => break,
                    _ = jobs_notify.notified() => WakeReason::Jobs,
                    _ = tokio::time::sleep_until(deadline) => WakeReason::Timer,
                }
            }
            None => tokio::select! {
                _ = shutdown.cancelled() => break,
                _ = jobs_notify.notified() => WakeReason::Jobs,
            },
        };

        should_evaluate_due = matches!(reason, WakeReason::Timer);
    }
}

fn cron_matches_minute_cached(
    expr: &str,
    minute_start: DateTime<Utc>,
    schedule_cache: &mut HashMap<String, Schedule>,
) -> Result<bool, anyhow::Error> {
    let schedule = parse_cron_cached(expr, schedule_cache)?;
    let prev = minute_start - Duration::seconds(1);
    let mut iter = schedule.after(&prev);
    let Some(next) = iter.next() else {
        return Ok(false);
    };
    Ok(next == minute_start)
}

fn next_cron_due_after_cached(
    jobs: &[jobs_repo::Job],
    now: DateTime<Utc>,
    schedule_cache: &mut HashMap<String, Schedule>,
) -> Option<DateTime<Utc>> {
    let mut next_due: Option<DateTime<Utc>> = None;
    for job in jobs {
        let Some(expr) = job.schedule.as_deref() else {
            continue;
        };

        let schedule = match parse_cron_cached(expr, schedule_cache) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let Some(next) = schedule.after(&now).next() else {
            continue;
        };
        next_due = Some(match next_due {
            Some(cur) => cur.min(next),
            None => next,
        });
    }
    next_due
}

fn parse_cron_cached<'a>(
    expr: &str,
    schedule_cache: &'a mut HashMap<String, Schedule>,
) -> Result<&'a Schedule, anyhow::Error> {
    let expr = normalize_cron(expr)?;
    if !schedule_cache.contains_key(&expr) {
        let schedule = Schedule::from_str(&expr)?;
        schedule_cache.insert(expr.clone(), schedule);
    }
    Ok(schedule_cache
        .get(&expr)
        .expect("schedule_cache contains key we just inserted"))
}
