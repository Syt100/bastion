use std::collections::HashSet;

use crate::job_spec::RetentionPolicyV1;

#[derive(Debug, Clone)]
pub struct RetentionSnapshot {
    pub run_id: String,
    pub ended_at: i64,
    pub pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionDecision {
    pub run_id: String,
    pub ended_at: i64,
    pub keep: bool,
    pub reasons: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionSelection {
    pub keep: Vec<RetentionDecision>,
    pub delete: Vec<RetentionDecision>,
}

pub fn select_retention(policy: &RetentionPolicyV1, now: i64, snapshots: &[RetentionSnapshot]) -> RetentionSelection {
    if !policy.enabled {
        let mut keep = snapshots
            .iter()
            .map(|s| RetentionDecision {
                run_id: s.run_id.clone(),
                ended_at: s.ended_at,
                keep: true,
                reasons: vec!["retention_disabled"],
            })
            .collect::<Vec<_>>();
        keep.sort_by(|a, b| (b.ended_at, &b.run_id).cmp(&(a.ended_at, &a.run_id)));
        return RetentionSelection { keep, delete: Vec::new() };
    }

    let keep_last = policy.keep_last.unwrap_or(0);
    let keep_days = policy.keep_days.unwrap_or(0);

    let mut keep_ids = HashSet::<&str>::new();

    // pinned is always kept.
    for s in snapshots {
        if s.pinned {
            keep_ids.insert(&s.run_id);
        }
    }

    // keep_last-N: newest by ended_at DESC (tie-breaker: run_id DESC).
    if keep_last > 0 {
        let mut ordered = snapshots.iter().collect::<Vec<_>>();
        ordered.sort_by(|a, b| (b.ended_at, &b.run_id).cmp(&(a.ended_at, &a.run_id)));
        for s in ordered.into_iter().take(keep_last as usize) {
            keep_ids.insert(&s.run_id);
        }
    }

    // keep_days: ended_at within window.
    if keep_days > 0 {
        let cutoff = now.saturating_sub((keep_days as i64).saturating_mul(24 * 60 * 60));
        for s in snapshots {
            if s.ended_at >= cutoff {
                keep_ids.insert(&s.run_id);
            }
        }
    }

    let mut keep = Vec::new();
    let mut delete = Vec::new();

    // Precompute reason sets for stable explanations.
    let mut keep_last_ids = HashSet::<&str>::new();
    if keep_last > 0 {
        let mut ordered = snapshots.iter().collect::<Vec<_>>();
        ordered.sort_by(|a, b| (b.ended_at, &b.run_id).cmp(&(a.ended_at, &a.run_id)));
        for s in ordered.into_iter().take(keep_last as usize) {
            keep_last_ids.insert(&s.run_id);
        }
    }

    let mut keep_days_ids = HashSet::<&str>::new();
    if keep_days > 0 {
        let cutoff = now.saturating_sub((keep_days as i64).saturating_mul(24 * 60 * 60));
        for s in snapshots {
            if s.ended_at >= cutoff {
                keep_days_ids.insert(&s.run_id);
            }
        }
    }

    for s in snapshots {
        let is_keep = keep_ids.contains(s.run_id.as_str());
        let mut reasons = Vec::new();
        if s.pinned {
            reasons.push("pinned");
        }
        if keep_last_ids.contains(s.run_id.as_str()) {
            reasons.push("keep_last");
        }
        if keep_days_ids.contains(s.run_id.as_str()) {
            reasons.push("keep_days");
        }

        if is_keep {
            keep.push(RetentionDecision {
                run_id: s.run_id.clone(),
                ended_at: s.ended_at,
                keep: true,
                reasons,
            });
        } else {
            delete.push(RetentionDecision {
                run_id: s.run_id.clone(),
                ended_at: s.ended_at,
                keep: false,
                reasons: vec!["delete"],
            });
        }
    }

    keep.sort_by(|a, b| (b.ended_at, &b.run_id).cmp(&(a.ended_at, &a.run_id)));
    delete.sort_by(|a, b| (b.ended_at, &b.run_id).cmp(&(a.ended_at, &a.run_id)));

    RetentionSelection { keep, delete }
}

#[cfg(test)]
mod tests {
    use super::{RetentionSnapshot, select_retention};
    use crate::job_spec::RetentionPolicyV1;

    #[test]
    fn keep_union_of_keep_last_and_keep_days_excluding_delete() {
        let now = 1_000_000;
        let snaps = vec![
            RetentionSnapshot { run_id: "a".to_string(), ended_at: now - 10, pinned: false },
            RetentionSnapshot { run_id: "b".to_string(), ended_at: now - 20, pinned: false },
            RetentionSnapshot { run_id: "c".to_string(), ended_at: now - 400_000, pinned: false },
        ];

        let policy = RetentionPolicyV1 { enabled: true, keep_last: Some(1), keep_days: Some(1), max_delete_per_tick: 50, max_delete_per_day: 200 };
        let sel = select_retention(&policy, now, &snaps);

        // keep_last keeps 'a'. keep_days(1) keeps 'a' and 'b' (both within ~1 day window), so union keeps a+b.
        assert_eq!(sel.keep.iter().map(|d| d.run_id.as_str()).collect::<Vec<_>>(), vec!["a", "b"]);
        assert_eq!(sel.delete.iter().map(|d| d.run_id.as_str()).collect::<Vec<_>>(), vec!["c"]);
    }

    #[test]
    fn pinned_is_never_deleted() {
        let now = 1_000_000;
        let snaps = vec![
            RetentionSnapshot { run_id: "a".to_string(), ended_at: now - 10, pinned: false },
            RetentionSnapshot { run_id: "b".to_string(), ended_at: now - 20, pinned: true },
            RetentionSnapshot { run_id: "c".to_string(), ended_at: now - 30, pinned: false },
        ];

        let policy = RetentionPolicyV1 { enabled: true, keep_last: Some(1), keep_days: None, max_delete_per_tick: 50, max_delete_per_day: 200 };
        let sel = select_retention(&policy, now, &snaps);

        assert!(sel.keep.iter().any(|d| d.run_id == "b" && d.reasons.contains(&"pinned")));
        assert!(!sel.delete.iter().any(|d| d.run_id == "b"));
    }
}
