# Change: Update source consistency warnings UX

## Why
We already detect (best-effort) when source files change during backup packaging and surface a warning count in the UI.

The remaining pain points are UX/operability:
- The user can see a count, but not *what changed* (breakdown + sample paths) without manual JSON inspection.
- The job run list only derives `consistency_changed_total` from `summary_json`, so for **running** runs the warning often appears late (after packaging completes).
- Notifications (Email/WeCom) do not explicitly call out "source changed during backup", so operators can miss the risk.

## What Changes
- Control-plane:
  - Make `GET /api/jobs/:id/runs` return a meaningful `consistency_changed_total` for running runs by falling back to the latest `source_consistency` run_event when `summary_json` is not present yet.
- Web UI:
  - In run detail, show a dedicated **Consistency** section with:
    - breakdown: changed / replaced / deleted / read_error
    - sample list (capped) with path + reason
    - a one-click action to jump to the filtered `source_consistency` event(s)
  - In job runs list, show the warning tag as soon as the run event is emitted (not only after completion).
- Notifications:
  - Include a short “source consistency warnings” line in notifications when present.

## Impact
- Affected specs: `control-plane`, `web-ui`, `notifications`
- Affected code (expected):
  - `crates/bastion-http/src/http/jobs/runs.rs`
  - `crates/bastion-storage/src/runs_repo/*` (if needed for efficient event lookup)
  - `ui/src/components/runs/RunDetailDetailsTabs.vue`
  - `ui/src/lib/run_summary.ts`
  - `ui/src/views/jobs/JobHistorySectionView.vue`
  - `crates/bastion-engine/src/notifications/*`

## Compatibility / Non-Goals
- This change is UX-focused; it does not change the underlying consistency detection algorithm.
- We do not add new snapshot/quiesce mechanisms here.

