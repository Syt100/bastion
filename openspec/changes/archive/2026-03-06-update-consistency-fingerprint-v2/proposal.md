# Change: Update source consistency fingerprint (v2)

## Why
The current consistency detector is intentionally heuristic (size/mtime/file_id) and warning-only.
However, in practice it can still miss or misclassify changes:
- `mtime` is recorded in seconds, which can miss "same-second" mutations.
- Post-read detection uses a path-based stat only; it can under-explain cases where the file changed via the open handle but the path later points elsewhere (or vice versa).
- Windows does not currently provide a strong `file_id`, increasing false negatives for replace-via-rename patterns.

We want to improve detection precision without changing the core contract (best-effort, bounded samples).

## What Changes
- Data model:
  - Introduce a v2 fingerprint with nanosecond timestamp support.
  - Record both post-read fingerprints:
    - `after_handle` (from the open handle metadata, when available)
    - `after_path` (from the path metadata)
  - Extend samples to include these fingerprints so operators can reason about “changed” vs “replaced”.
- Detection logic:
  - Prefer `file_id` when available to detect replacement.
  - Use nanosecond mtime comparisons when available.
  - Improve reason classification for changed/replaced/deleted/read_error.
- Platform support:
  - Implement Windows `file_id` (volume serial + file index) when feasible.
- Surfacing:
  - Run event and `summary_json` store the v2 report.
  - Web UI renders v2 fields.

## Impact
- Affected specs: `sources`, `backup-jobs`, `control-plane`, `web-ui`
- Affected code (expected):
  - `crates/bastion-backup/src/backup/source_consistency.rs`
  - `crates/bastion-backup/src/backup/filesystem/*`
  - `crates/bastion-backup/src/backup/vaultwarden/*`
  - `crates/bastion-engine/src/scheduler/worker/execute/*`
  - `crates/bastion/src/agent_client/tasks/*`
  - `ui/src/lib/run_summary.ts`
  - `ui/src/components/runs/*`

## Compatibility / Non-Goals
- We do not attempt to make backups consistent without snapshots; this remains warning-only.
- We keep samples capped and do not store unbounded payloads.

