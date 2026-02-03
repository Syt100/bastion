use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use tracing::warn;

use bastion_core::agent_protocol::OverlapPolicyV1;

use super::super::cron::cron_matches_minute_cached;
use super::super::storage::OfflineRunWriterHandle;
use super::types::{InFlightCounts, OfflineRunTask};

fn allow_due_for_local_minute(
    tz: chrono_tz::Tz,
    local_minute_start: chrono::DateTime<chrono_tz::Tz>,
) -> bool {
    use chrono::TimeZone as _;
    // DST fold: local wall time occurs twice. Run once by choosing the first occurrence (earlier offset).
    match tz.from_local_datetime(&local_minute_start.naive_local()) {
        chrono::LocalResult::Ambiguous(first, _) => local_minute_start == first,
        _ => true,
    }
}

#[derive(Debug)]
enum CronDecision {
    Queue {
        job_id: String,
        job_name: String,
        spec: Box<bastion_core::agent_protocol::JobSpecResolvedV1>,
    },
    Reject {
        job_id: String,
        job_name: String,
    },
}

fn decide_cron_minute_jobs(
    agent_id: &str,
    minute_start: chrono::DateTime<chrono::Utc>,
    jobs: Vec<bastion_core::agent_protocol::JobConfigV1>,
    schedule_cache: &mut std::collections::HashMap<String, cron::Schedule>,
    inflight_for_job: impl Fn(&str) -> usize,
) -> Vec<CronDecision> {
    use chrono::TimeZone as _;

    let mut decisions = Vec::new();
    for job in jobs {
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
                warn!(
                    agent_id = %agent_id,
                    timezone = ?job.schedule_timezone,
                    "invalid schedule timezone; skipping"
                );
                continue;
            }
        };
        let local_minute_start = tz.from_utc_datetime(&minute_start.naive_utc());
        if !allow_due_for_local_minute(tz, local_minute_start) {
            continue;
        }

        match cron_matches_minute_cached(expr, local_minute_start, schedule_cache) {
            Ok(true) => {
                let should_reject = matches!(job.overlap_policy, OverlapPolicyV1::Reject)
                    && inflight_for_job(&job.job_id) > 0;

                if should_reject {
                    decisions.push(CronDecision::Reject {
                        job_id: job.job_id,
                        job_name: job.name,
                    });
                } else {
                    decisions.push(CronDecision::Queue {
                        job_id: job.job_id,
                        job_name: job.name,
                        spec: Box::new(job.spec),
                    });
                }
            }
            Ok(false) => {}
            Err(error) => {
                warn!(
                    agent_id = %agent_id,
                    error = %error,
                    "invalid cron schedule; skipping"
                );
            }
        }
    }
    decisions
}

pub(super) async fn offline_cron_loop(
    data_dir: PathBuf,
    agent_id: String,
    mut connected_rx: tokio::sync::watch::Receiver<bool>,
    tx: tokio::sync::mpsc::UnboundedSender<OfflineRunTask>,
    inflight: std::sync::Arc<tokio::sync::Mutex<InFlightCounts>>,
) {
    use chrono::{DateTime, Duration as ChronoDuration, Utc};
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
                    let decisions = {
                        let state = inflight.lock().await;
                        decide_cron_minute_jobs(
                            &agent_id,
                            minute_start,
                            snapshot.jobs,
                            &mut schedule_cache,
                            |job_id| state.inflight_for_job(job_id),
                        )
                    };

                    for decision in decisions {
                        match decision {
                            CronDecision::Reject { job_id, job_name } => {
                                if let Err(error) =
                                    persist_offline_rejected_run(&data_dir, &job_id, &job_name)
                                        .await
                                {
                                    warn!(
                                        agent_id = %agent_id,
                                        job_id = %job_id,
                                        error = %error,
                                        "failed to persist offline rejected run"
                                    );
                                }
                            }
                            CronDecision::Queue {
                                job_id,
                                job_name,
                                spec,
                            } => {
                                let run_id = uuid::Uuid::new_v4().to_string();
                                let task = OfflineRunTask {
                                    run_id,
                                    job_id,
                                    job_name,
                                    spec: *spec,
                                };

                                {
                                    let mut state = inflight.lock().await;
                                    state.inc_job(&task.job_id);
                                }

                                if tx.send(task).is_err() {
                                    break;
                                }
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

#[cfg(test)]
mod tests {
    use chrono::TimeZone as _;

    use super::{
        allow_due_for_local_minute, decide_cron_minute_jobs, persist_offline_rejected_run,
    };

    #[test]
    fn allow_due_for_local_minute_runs_once_on_dst_fold() {
        let tz: chrono_tz::Tz = "America/New_York".parse().unwrap();
        let naive = chrono::NaiveDate::from_ymd_opt(2025, 11, 2)
            .unwrap()
            .and_hms_opt(1, 30, 0)
            .unwrap();

        match tz.from_local_datetime(&naive) {
            chrono::LocalResult::Ambiguous(first, second) => {
                assert!(allow_due_for_local_minute(tz, first));
                assert!(!allow_due_for_local_minute(tz, second));
            }
            other => panic!("expected DST fold ambiguity, got {other:?}"),
        }
    }

    #[test]
    fn allow_due_for_local_minute_allows_non_ambiguous_time() {
        let tz: chrono_tz::Tz = "America/New_York".parse().unwrap();
        let naive = chrono::NaiveDate::from_ymd_opt(2025, 11, 2)
            .unwrap()
            .and_hms_opt(3, 0, 0)
            .unwrap();

        let dt = tz.from_local_datetime(&naive).single().unwrap();
        assert!(allow_due_for_local_minute(tz, dt));
    }

    #[tokio::test]
    async fn persist_offline_rejected_run_writes_run_file_and_event() {
        use super::super::super::storage::{
            OfflineRunEventV1, OfflineRunFileV1, OfflineRunStatusV1,
        };

        let tmp = tempfile::tempdir().unwrap();
        persist_offline_rejected_run(tmp.path(), "job1", "job name")
            .await
            .unwrap();

        let offline_runs_dir = tmp.path().join("agent").join("offline_runs");
        let mut dirs = std::fs::read_dir(&offline_runs_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().ok().is_some_and(|t| t.is_dir()))
            .map(|e| e.path())
            .collect::<Vec<_>>();
        assert_eq!(dirs.len(), 1);
        let run_dir = dirs.pop().unwrap();

        let run_raw = std::fs::read(run_dir.join("run.json")).unwrap();
        let run: OfflineRunFileV1 = serde_json::from_slice(&run_raw).unwrap();
        assert_eq!(run.v, 1);
        assert_eq!(run.job_id, "job1");
        assert_eq!(run.job_name, "job name");
        assert_eq!(run.status, OfflineRunStatusV1::Rejected);
        assert!(run.started_at > 0);
        assert!(run.ended_at.is_some());
        assert_eq!(
            run.summary,
            Some(serde_json::json!({ "executed_offline": true }))
        );
        assert_eq!(run.error.as_deref(), Some("overlap_rejected"));

        let events_text = std::fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
        let events = events_text
            .lines()
            .map(|line| serde_json::from_str::<OfflineRunEventV1>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, "rejected");
        assert_eq!(events[0].message, "rejected");
        assert_eq!(
            events[0].fields,
            Some(serde_json::json!({ "source": "schedule", "executed_offline": true }))
        );
    }

    fn test_job(
        job_id: &str,
        schedule: Option<&str>,
        schedule_timezone: Option<&str>,
        overlap_policy: bastion_core::agent_protocol::OverlapPolicyV1,
    ) -> bastion_core::agent_protocol::JobConfigV1 {
        bastion_core::agent_protocol::JobConfigV1 {
            job_id: job_id.to_string(),
            name: format!("job-{job_id}"),
            schedule: schedule.map(|v| v.to_string()),
            schedule_timezone: schedule_timezone.map(|v| v.to_string()),
            overlap_policy,
            updated_at: 0,
            spec: bastion_core::agent_protocol::JobSpecResolvedV1::Sqlite {
                v: 1,
                pipeline: Default::default(),
                source: bastion_core::job_spec::SqliteSource {
                    path: "/db.sqlite".to_string(),
                    integrity_check: false,
                },
                target: bastion_core::agent_protocol::TargetResolvedV1::LocalDir {
                    base_dir: "/tmp".to_string(),
                    part_size_bytes: 1024,
                },
            },
        }
    }

    #[test]
    fn decide_cron_minute_jobs_queues_due_jobs_by_default() {
        let minute_start = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let jobs = vec![test_job(
            "job1",
            Some("* * * * *"),
            Some("UTC"),
            bastion_core::agent_protocol::OverlapPolicyV1::Queue,
        )];
        let mut cache = std::collections::HashMap::new();
        let decisions = decide_cron_minute_jobs("agent", minute_start, jobs, &mut cache, |_| 0);
        assert_eq!(decisions.len(), 1);
        match &decisions[0] {
            super::CronDecision::Queue { job_id, .. } => assert_eq!(job_id, "job1"),
            other => panic!("expected Queue, got {other:?}"),
        }
    }

    #[test]
    fn decide_cron_minute_jobs_rejects_when_overlap_reject_and_inflight() {
        let minute_start = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let jobs = vec![test_job(
            "job1",
            Some("* * * * *"),
            Some("UTC"),
            bastion_core::agent_protocol::OverlapPolicyV1::Reject,
        )];
        let mut cache = std::collections::HashMap::new();
        let decisions = decide_cron_minute_jobs("agent", minute_start, jobs, &mut cache, |_| 1);
        assert_eq!(decisions.len(), 1);
        match &decisions[0] {
            super::CronDecision::Reject { job_id, .. } => assert_eq!(job_id, "job1"),
            other => panic!("expected Reject, got {other:?}"),
        }
    }

    #[test]
    fn decide_cron_minute_jobs_skips_invalid_timezone_and_cron() {
        let minute_start = chrono::Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let jobs = vec![
            test_job(
                "job1",
                Some("* * * * *"),
                Some("Not/AZone"),
                bastion_core::agent_protocol::OverlapPolicyV1::Queue,
            ),
            test_job(
                "job2",
                Some("not cron"),
                Some("UTC"),
                bastion_core::agent_protocol::OverlapPolicyV1::Queue,
            ),
        ];
        let mut cache = std::collections::HashMap::new();
        let decisions = decide_cron_minute_jobs("agent", minute_start, jobs, &mut cache, |_| 0);
        assert!(decisions.is_empty());
    }

    #[test]
    fn decide_cron_minute_jobs_runs_once_on_dst_fold_by_skipping_second_occurrence() {
        // America/New_York DST ends on 2025-11-02. Local 01:30 happens twice:
        // - 05:30 UTC (EDT, first occurrence) -> allowed
        // - 06:30 UTC (EST, second occurrence) -> skipped
        let jobs = vec![test_job(
            "job1",
            Some("* * * * *"),
            Some("America/New_York"),
            bastion_core::agent_protocol::OverlapPolicyV1::Queue,
        )];
        let mut cache = std::collections::HashMap::new();

        let first = chrono::Utc.with_ymd_and_hms(2025, 11, 2, 5, 30, 0).unwrap();
        let decisions = decide_cron_minute_jobs("agent", first, jobs.clone(), &mut cache, |_| 0);
        assert_eq!(decisions.len(), 1);

        let second = chrono::Utc.with_ymd_and_hms(2025, 11, 2, 6, 30, 0).unwrap();
        let decisions = decide_cron_minute_jobs("agent", second, jobs, &mut cache, |_| 0);
        assert!(decisions.is_empty());
    }
}
