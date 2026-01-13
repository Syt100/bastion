use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, Duration, LocalResult, TimeZone as _, Utc};
use chrono_tz::Tz;
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

fn allow_due_for_local_minute(tz: Tz, local_minute_start: DateTime<Tz>) -> bool {
    // DST fold: local wall time occurs twice. Run once by choosing the first occurrence (earlier offset).
    match tz.from_local_datetime(&local_minute_start.naive_local()) {
        LocalResult::Ambiguous(first, _) => local_minute_start == first,
        _ => true,
    }
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

                    let tz = match job.schedule_timezone.parse::<Tz>() {
                        Ok(v) => v,
                        Err(_) => {
                            warn!(
                                job_id = %job.id,
                                schedule_timezone = %job.schedule_timezone,
                                "invalid schedule timezone; skipping"
                            );
                            continue;
                        }
                    };
                    let local_minute_start = tz.from_utc_datetime(&minute_start.naive_utc());
                    if !allow_due_for_local_minute(tz, local_minute_start) {
                        continue;
                    }

                    match cron_matches_minute_cached(expr, local_minute_start, &mut schedule_cache)
                    {
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

fn cron_matches_minute_cached<Tz1: chrono::TimeZone>(
    expr: &str,
    minute_start: DateTime<Tz1>,
    schedule_cache: &mut HashMap<String, Schedule>,
) -> Result<bool, anyhow::Error> {
    let schedule = parse_cron_cached(expr, schedule_cache)?;
    let prev = minute_start.clone() - Duration::seconds(1);
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

        let tz = match job.schedule_timezone.parse::<Tz>() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let now_local = tz.from_utc_datetime(&now.naive_utc());
        let iter = schedule.after(&now_local);
        for candidate in iter {
            if !allow_due_for_local_minute(tz, candidate) {
                continue;
            }
            let candidate_utc = candidate.with_timezone(&Utc);
            next_due = Some(match next_due {
                Some(cur) => cur.min(candidate_utc),
                None => candidate_utc,
            });
            break;
        }
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

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone as _};
    use chrono_tz::America::New_York;

    use super::{allow_due_for_local_minute, normalize_cron};

    #[test]
    fn normalize_cron_rejects_nonzero_seconds() {
        assert!(normalize_cron("10 * * * * *").is_err());
        assert!(normalize_cron("*/10 * * * * *").is_err());
        assert_eq!(normalize_cron("0 */5 * * * *").unwrap(), "0 */5 * * * *");
    }

    #[test]
    fn dst_fold_runs_once_by_selecting_first_occurrence() {
        // US DST ends on the first Sunday in November. In 2026, that's 2026-11-01.
        let naive = NaiveDate::from_ymd_opt(2026, 11, 1)
            .unwrap()
            .and_hms_opt(1, 30, 0)
            .unwrap();
        match New_York.from_local_datetime(&naive) {
            chrono::LocalResult::Ambiguous(first, second) => {
                assert!(allow_due_for_local_minute(New_York, first));
                assert!(!allow_due_for_local_minute(New_York, second));
            }
            other => panic!("expected ambiguous local time, got: {other:?}"),
        }
    }
}
