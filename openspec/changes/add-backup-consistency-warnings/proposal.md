# Change: Add best-effort source consistency warnings (no snapshots)

## Why
Filesystem backups without a snapshot mechanism cannot guarantee point-in-time consistency. Today, if files change during packaging, the resulting backup may be internally inconsistent, but the user has limited visibility into that risk.

Additionally, the current `archive_v1` packaging path can read file contents more than once (hashing + tar append), which can produce a mismatch between the recorded hash and the bytes actually written into the archive when a file changes mid-run.

We want to:
- Detect (best-effort) when source files change during backup packaging.
- Surface this as an explicit warning in the Web UI (run detail + job runs list).
- Ensure recorded per-file hashes match the archived bytes (single-read hashing).

## What Changes
- Backend (backup builders):
  - Add a best-effort consistency checker that captures file fingerprints before/after reading and records "changed during backup" warnings.
  - Refactor `archive_v1` tar writers to compute file hashes from the same bytes written into the archive (single read).
  - Apply this to filesystem backups (both `archive_v1` and `raw_tree_v1`) and Vaultwarden backups.
- Reporting:
  - Persist a structured `consistency` report in the run `summary_json` for successful runs.
  - Emit a structured `run_event` warning (`kind=source_consistency`) when changes are detected.
- Control-plane / API:
  - Extend the job run list endpoint (`GET /api/jobs/:id/runs`) to include `consistency_changed_total` so the UI can show warnings without fetching each run detail.
- Web UI:
  - Show a warning badge/tag in the job's runs list when a run has consistency warnings.
  - Show a warning badge/tag in run detail, with a link to the event details / sample.
- Tests:
  - Add regression tests to ensure hashing is single-read and consistent, and that the change detector reports modifications.

## Impact
- Affected specs: `sources`, `backup-jobs`, `control-plane`, `web-ui`
- Affected code (expected):
  - `crates/bastion-backup/src/backup/filesystem/tar/*`
  - `crates/bastion-backup/src/backup/filesystem/raw_tree.rs`
  - `crates/bastion-backup/src/backup/vaultwarden/*`
  - `crates/bastion-engine/src/scheduler/worker/execute/*`
  - `crates/bastion/src/agent_client/tasks/*`
  - `crates/bastion-http/src/http/jobs/runs.rs`
  - `ui/src/views/jobs/JobHistorySectionView.vue`
  - `ui/src/components/runs/*`
  - `ui/src/lib/run_summary.ts`
  - `ui/src/stores/jobs.ts`

## Compatibility / Non-Goals
- No attempt to "make backups consistent" without snapshots; this change is warning-only.
- No unbounded logs: sample size MUST be capped.
- No new job types in this change (snapshot/quiesce hooks are future work).

