# Change: Update Run Retention to Preserve Runs With Existing Snapshots

## Why
The system currently prunes old `runs` records by age.

Once snapshots become a first-class resource, pruning runs too aggressively can cause:
- orphaned snapshot data (still exists in storage but the run history is gone)
- loss of traceability (no audit trail for an existing snapshot)

We need run retention to be snapshot-aware:
- runs with `run_artifacts.status=present` (or deleting/error) should not be pruned
- runs can be pruned after the associated snapshot is deleted/missing

High-level design reference: `docs/backup-snapshots.md` ("与现有 run retention 的关系").

## What Changes
- Update the run retention pruning query to skip runs that still have live snapshots.
- Add tests to prevent regressions.
- Keep existing behavior for runs without snapshots (or snapshots already deleted/missing).

## Scope
- Only the pruning criteria changes; schedule and cadence remain the same.

## Out of Scope
- Changing the default retention window.
- Exporting run history to external archives.

## Success Criteria
- Runs with existing snapshots are not pruned.
- After snapshots are deleted, run retention can prune old runs as before.

