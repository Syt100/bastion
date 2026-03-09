# Change: Add Backup Snapshots Index + Read-Only UI Page

## Why
Today Bastion stores run history, but "successful backup output" is not a first-class, manageable resource:

- Users cannot list historical backup outputs ("snapshots") per job.
- Users cannot see snapshot-level metadata (format, sizes, counts, target at the time of the run).
- Later lifecycle features (delete / retention / pin) have no stable index to build upon.

This change introduces a snapshot index and a dedicated, independent snapshots page in the Web UI (read-only for now).

High-level design reference: `docs/dev/design/backup-snapshots.md`.

## What Changes
- Add a persistent snapshot index table (`run_artifacts` / "snapshots") in the Hub DB.
- Record a snapshot row when a run completes successfully (using the run-time `target_snapshot_json`).
- Expose read-only APIs to list/get snapshots per job (filtering + pagination).
- Add a new Web UI route/page to view snapshots as a first-class list (login required).

## Scope
- Indexing and read-only browsing only.
- Covers both Hub-executed and Agent-executed runs (index is stored in Hub DB).

## Out of Scope (Follow-ups)
- Async deletion queue and logs.
- Pin/protect.
- Retention policy (preview/apply + loop).
- Job archive cascade deletion.
- Reconciliation scan of external/manual deletions.

## Key Decisions
- **Snapshot identity**: `run_id` is the primary key (1 successful run → 1 snapshot record).
- **Stability**: snapshot location is derived from `runs.target_snapshot_json` captured at run start, not from the current job spec.
- **Source of truth**: Hub DB stores snapshot index and status; target storage remains the actual data.

## Risks
- Some older runs may have incomplete/partial metrics (sizes/counts). The UI must be resilient to missing fields.
- If target snapshot persistence fails for a run, indexing must fail gracefully (index can be absent, run still succeeds).

## Success Criteria
- A user can open a job and navigate to a dedicated "Snapshots" page.
- The page lists recent snapshots with key metadata and links to run details.
- APIs support filtering/pagination and remain stable for later deletion/retention work.
