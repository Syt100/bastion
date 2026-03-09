# Change: Add incomplete cleanup management (queue + UI)

## Why
The current incomplete run cleanup is best-effort and only observable via logs. When targets (e.g. WebDAV) are unreachable or misconfigured, the cleanup loop can repeatedly hit the same runs and emit noisy warnings without a clear operator workflow to resolve or silence them.

We want:
- A persistent cleanup queue with backoff/abandon semantics (no tight loops / no log spam)
- Full visibility into cleanup attempts and failures
- A UI for monitoring and operating cleanup tasks (retry/ignore)
- A user choice at “delete job” time: hard delete (cascade) vs archive (non-cascade)

## What Changes
- Persist a per-run target snapshot (`runs.target_snapshot_json`) so maintenance uses the run’s actual target configuration
- Add `jobs.archived_at` and treat “non-cascade delete” as archive/soft-delete (no new runs scheduled, history preserved)
- Add a persistent incomplete cleanup task queue + task events with:
  - per-target backoff
  - automatic abandon after thresholds
  - operator actions (retry now / ignore / unignore)
- Add authenticated maintenance HTTP endpoints and a mobile-friendly UI page to view and operate cleanup tasks

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code:
  - Storage schema + repos (`crates/bastion-storage`)
  - Engine cleanup loop (`crates/bastion-engine`)
  - HTTP API (`crates/bastion-http`)
  - UI routes/views/stores (`ui/`)

## Compatibility / Non-Goals
- Existing run execution semantics do not change.
- Hard delete remains available and continues to cascade by default.
- UI MUST redact sensitive URL components and never expose secrets.

