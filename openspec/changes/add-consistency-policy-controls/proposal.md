# Change: Add consistency policy controls

## Why
Consistency warnings are currently “warn-only”. That is correct as a default, but different workloads have different tolerances:
- Some backups are best-effort: keep going and warn.
- Some backups must be point-in-time: if the source changes, the run should fail (or at least not be retained).

We want to make this behavior configurable per job/source, so operators can choose the right trade-off.

## What Changes
- Job spec:
  - Add `consistency_policy` with values:
    - `warn` (default): keep run successful, warn in UI
    - `fail`: mark run failed when warnings exceed threshold
    - `ignore`: do not emit warnings (or do not surface them)
  - Add optional `consistency_fail_threshold` (integer, default `0` meaning “any warning fails” when policy is `fail`).
  - (Optional) Add `upload_on_consistency_failure` (boolean) to control whether artifacts are uploaded/stored when failing for consistency.
- Executors:
  - After packaging completes and report is available, enforce the policy consistently for Hub and Agent execution paths.
  - Always persist the report in `summary_json` (so failures remain diagnosable).
- Web UI:
  - Expose controls in the Job editor with clear copy about trade-offs and snapshots.
- Notifications:
  - When policy triggers a failure, include explicit messaging.

## Impact
- Affected specs: `backup-jobs`, `control-plane`, `web-ui`, `notifications`
- Affected code (expected):
  - `crates/bastion-core/src/job_spec/*`
  - `crates/bastion-engine/src/scheduler/worker/execute/*`
  - `crates/bastion/src/agent_client/tasks/*`
  - `ui/src/components/jobs/editor/*`
  - `crates/bastion-engine/src/notifications/*`

## Compatibility / Non-Goals
- This does not add snapshots; it only changes policy enforcement for existing best-effort detection.
- Thresholding is about totals, not per-path matching.

