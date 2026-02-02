use std::collections::HashMap;

use bastion_core::agent_protocol::JobSpecResolvedV1;

#[derive(Debug, Clone)]
pub(super) struct OfflineRunTask {
    pub(super) run_id: String,
    pub(super) job_id: String,
    pub(super) job_name: String,
    pub(super) spec: JobSpecResolvedV1,
}

#[derive(Debug, Default)]
pub(super) struct InFlightCounts {
    per_job: HashMap<String, usize>,
}

impl InFlightCounts {
    pub(super) fn inflight_for_job(&self, job_id: &str) -> usize {
        self.per_job.get(job_id).copied().unwrap_or(0)
    }

    pub(super) fn inc_job(&mut self, job_id: &str) {
        *self.per_job.entry(job_id.to_string()).or_insert(0) += 1;
    }

    pub(super) fn dec_job(&mut self, job_id: &str) {
        let Some(v) = self.per_job.get_mut(job_id) else {
            return;
        };
        *v = v.saturating_sub(1);
        if *v == 0 {
            self.per_job.remove(job_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InFlightCounts;

    #[test]
    fn inflight_counts_default_is_zero() {
        let counts = InFlightCounts::default();
        assert_eq!(counts.inflight_for_job("job1"), 0);
    }

    #[test]
    fn inflight_counts_increments_and_decrements_per_job() {
        let mut counts = InFlightCounts::default();

        counts.inc_job("job1");
        assert_eq!(counts.inflight_for_job("job1"), 1);
        assert_eq!(counts.inflight_for_job("job2"), 0);

        counts.inc_job("job1");
        counts.inc_job("job2");
        assert_eq!(counts.inflight_for_job("job1"), 2);
        assert_eq!(counts.inflight_for_job("job2"), 1);

        counts.dec_job("job1");
        assert_eq!(counts.inflight_for_job("job1"), 1);

        counts.dec_job("job1");
        assert_eq!(counts.inflight_for_job("job1"), 0);
        // Decrementing missing keys should be a no-op.
        counts.dec_job("job1");
        assert_eq!(counts.inflight_for_job("job1"), 0);
    }
}
