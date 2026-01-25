# Change: Add Backup Snapshots Delete Queue (Async, Observable)

## Why
Users need to delete historical backup outputs to reclaim storage and manage lifecycle.

Deletion must be:
- asynchronous (avoid HTTP timeouts)
- idempotent (safe to retry or re-submit)
- observable (status, attempts, error classification, event log)
- resilient (retry/backoff, operator controls like retry/ignore)

We already have a proven pattern in the codebase: `incomplete_cleanup_tasks` + events + UI.
This change applies the same pattern to **successful snapshot deletion**.

High-level design reference: `docs/backup-snapshots.md`.

## What Changes
- Add `artifact_delete_tasks` + `artifact_delete_events` tables (modeled after incomplete cleanup).
- Add a background worker loop to process delete tasks on the Hub.
- Implement deletion adapters for:
  - `webdav` snapshots (Hub executes HTTP DELETE)
  - `local_dir` snapshots **when they belong to Hub node only** (Hub local filesystem)
- Add APIs:
  - enqueue snapshot deletion (single + bulk)
  - view deletion task state and events
  - operator actions: retry now / ignore
- Update the Snapshots UI page to support delete actions and show deletion status/logs.

## Scope
- Hub-executed deletion for `webdav` and Hub-local `local_dir`.
- Reuses existing error classification and retry/backoff conventions.

## Out of Scope (Follow-ups)
- Agent-executed deletion for Agent-local `local_dir` snapshots (separate change).
- Retention policy automation.
- Pin/protect semantics and forced deletion.

## Key Decisions
- **Task identity**: one delete task per `run_id` (same as snapshot identity).
- **Idempotency**: "not found" is treated as success; repeated enqueue is safe.
- **Observability**: write structured events for each attempt and operator action.

## Risks
- WebDAV servers differ in how they implement collection deletion. We must handle trailing-slash and 404 cases consistently.
- Deletion is destructive; UI must require explicit confirmation and show what will be deleted.

## Success Criteria
- Users can delete snapshots from the UI without blocking the page.
- If the target is unavailable, the task retries with backoff and the UI shows actionable status/errors.
- Operators can retry/ignore and view the event log.

