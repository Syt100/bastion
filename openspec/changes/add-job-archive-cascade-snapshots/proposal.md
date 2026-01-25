# Change: Job Archive With Optional Cascade Snapshot Deletion

## Why
When a job is archived/removed from active scheduling, its historical snapshots often become unwanted.

Operators should be able to choose:
- archive the job only (keep snapshots)
- archive the job and cascade delete its snapshots (via async delete queue)

This reduces manual cleanup work and makes lifecycle management predictable.

High-level design reference: `docs/backup-snapshots.md` (milestone "Job deletion cascade").

## What Changes
- Extend the job archive flow to optionally enqueue deletion for all snapshots of that job.
- Add UI confirmation semantics:
  - a "cascade delete snapshots" option
  - pinned snapshots require force or are excluded (see pinning spec)
- Ensure cascade deletion is asynchronous and observable (delete tasks + events).

## Scope
- Applies to "archive job" (soft-delete) workflows.

## Out of Scope
- Hard-delete jobs and all history.
- Cross-job snapshot sharing/dedup.

## Risks
- Cascading deletions are destructive; UI must clearly show what will be deleted and require confirmation.

## Success Criteria
- Users can archive a job with an explicit cascade option.
- If cascade is selected, snapshot delete tasks are enqueued and visible in the snapshots UI.

