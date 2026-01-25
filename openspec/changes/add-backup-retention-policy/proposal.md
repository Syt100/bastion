# Change: Add Backup Retention Policy (Per Job + Preview + Enforcement Loop)

## Why
Without retention, storage grows unbounded and operators must manually delete snapshots.

We need a long-term, best-practice retention system that is:
- server-enforced (not just UI)
- configurable per job (with a global default)
- safe (preview/simulate, rate limits)
- compatible with multi-node storage ownership (Hub/Agent)

High-level design reference: `docs/backup-snapshots.md`.

## What Changes
- Add a retention configuration to job specs (and a global default in Hub settings).
- Add APIs:
  - read/update retention config
  - preview (simulate) retention selection
  - apply now (enqueue deletions immediately)
- Add a background retention loop that periodically enqueues snapshot delete tasks.
- Ensure retention:
  - uses snapshot index (`run_artifacts`) as input
  - excludes pinned snapshots
  - uses safety limits (max deletes per day / per tick)

## Scope
- Core policy: keep-last-N + keep-days (union) + pinned exclusion.
- Server loop enforcement + preview/apply endpoints.
- UI support in the job editor for configuring retention and previewing planned deletes.

## Out of Scope (Future)
- GFS schedules (grandfather-father-son) or custom calendars.
- Storage tiering or archiving to cold storage.

## Key Decisions
- Policy is evaluated against `run_artifacts` (snapshots), not raw runs.
- Preview is purely computed; apply enqueues deletions via the delete queue.
- Defaults: retention is opt-in, with a configurable global default for new jobs.

## Risks
- Misconfiguration can cause unexpected deletions; preview + safety limits mitigate this.
- Large jobs with many snapshots require efficient queries and bounded work per tick.

## Success Criteria
- Users can configure retention per job and preview which snapshots would be deleted.
- The system automatically enqueues deletions and keeps storage bounded over time.

