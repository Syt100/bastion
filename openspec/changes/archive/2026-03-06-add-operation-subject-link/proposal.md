# Change: Link Operations To Their Subject (Run)

## Why
Restore/verify operations are currently stored independently from backup runs, which makes it hard to reliably surface restore work inside a run’s history view (and work appears to “disappear” when a modal is closed).

## What Changes
- Add a generic subject reference to `operations` so an operation can be linked to a domain entity.
- When starting restore/verify for a run, create the operation with `subject_kind = run` and `subject_id = <run_id>`.
- Add an API to list operations for a run.

## Impact
- Storage/DB: `operations` schema and indexes.
- Backend API: new run-scoped operations list endpoint.
- Web UI: consumed by the Run Detail page change (separate spec).
