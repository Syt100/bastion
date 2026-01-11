# Design: Incomplete Cleanup Management (Queue + UI)

## Overview

We introduce a persistent “incomplete cleanup” queue to track and operate the cleanup of stale runs that never produced a completion marker (e.g. missing `complete.json`).

This replaces the current “log-only” behavior with:
- deterministic scheduling (no tight loops)
- stateful retries with exponential backoff and jitter
- automatic abandon for long-term failures
- operator tooling in the UI (retry/ignore)

## Key Decisions (Confirmed)

1. **Run target snapshot**: persist `runs.target_snapshot_json` at run start and use it for cleanup.
2. **Delete semantics**: “non-cascade delete” is implemented as **archive/soft-delete** (job remains, history remains).
3. **Abandon**: cleanup automatically stops retrying after thresholds, but can be manually retried.
4. **UI**: maintenance UI must be mobile friendly (card layout on small screens, table on desktop).

## Terminology

- **Run**: a single execution record (`runs`).
- **Incomplete run**: a run where the target completion marker is missing.
- **Cleanup task**: a persistent row describing the cleanup attempts for a run.
- **Target snapshot**: the minimal target configuration needed to clean up the target location.

## Data Model

### 1) `jobs` (archive support)

Add:
- `archived_at INTEGER` (unix seconds, nullable)

Behavior:
- Archived jobs are not scheduled automatically and are hidden by default in UI lists (with an optional “show archived” filter).
- Hard delete remains available via existing DELETE; archive is a separate operator action.

### 2) `runs` (target snapshot)

Add:
- `target_snapshot_json TEXT` (nullable)

Write timing:
- After parsing & validating job spec and before dispatch/execute, write the snapshot to the run row.

Snapshot schema (JSON):
```json
{
  "node_id": "hub|<agent_id>",
  "target": {
    "type": "webdav",
    "base_url": "https://example/dav",
    "secret_name": "webdav-secret"
  }
}
```
or
```json
{
  "node_id": "hub|<agent_id>",
  "target": {
    "type": "local_dir",
    "base_dir": "/var/lib/bastion/targets"
  }
}
```

Redaction rules for UI/logs:
- for URLs: remove username/password, query, fragment

### 3) `incomplete_cleanup_tasks` (queue)

Purpose: store cleanup scheduling, retries, and operator actions.

Recommended schema:
- `run_id TEXT PRIMARY KEY` (FK `runs(id)` ON DELETE CASCADE)
- `job_id TEXT NOT NULL`
- `node_id TEXT NOT NULL`
- `target_type TEXT NOT NULL` (`webdav` | `local_dir`)
- `target_snapshot_json TEXT NOT NULL`
- `status TEXT NOT NULL`
  - `queued`: eligible to attempt now
  - `running`: currently executing an attempt (claimed)
  - `retrying`: last attempt failed; waiting for `next_attempt_at`
  - `blocked`: config/auth errors; retry slower, primarily operator-driven
  - `done`: no cleanup needed or cleanup succeeded
  - `ignored`: operator chose to ignore
  - `abandoned`: auto-stop after thresholds
- `attempts INTEGER NOT NULL`
- `created_at INTEGER NOT NULL`
- `updated_at INTEGER NOT NULL`
- `last_attempt_at INTEGER` (nullable)
- `next_attempt_at INTEGER NOT NULL`
- `last_error_kind TEXT` (nullable; `network|http|auth|config|unknown`)
- `last_error TEXT` (nullable; short, redacted)
- `ignored_at INTEGER` (nullable)
- `ignored_by_user_id INTEGER` (nullable FK `users(id)` ON DELETE SET NULL)
- `ignore_reason TEXT` (nullable)

Indexes:
- `idx_cleanup_tasks_status_next_attempt` on `(status, next_attempt_at)`
- `idx_cleanup_tasks_job_id` on `(job_id)`
- `idx_cleanup_tasks_node_id` on `(node_id)`

### 4) `incomplete_cleanup_events` (audit trail)

Purpose: show attempt history in UI (and reduce reliance on log scraping).

Schema:
- `run_id TEXT NOT NULL` (FK `incomplete_cleanup_tasks(run_id)` ON DELETE CASCADE)
- `seq INTEGER NOT NULL`
- `ts INTEGER NOT NULL`
- `level TEXT NOT NULL` (`info|warn|error`)
- `kind TEXT NOT NULL`
  - `attempt`, `skip_complete`, `skip_not_found`, `deleted`, `failed`, `blocked`, `ignored`, `unignored`, `abandoned`, `retry_now`
- `message TEXT NOT NULL` (short, redacted)
- `fields_json TEXT` (nullable; details like http status, duration_ms, redacted_url)
- PRIMARY KEY `(run_id, seq)`

## Task State Machine

### Creation / Upsert

When a run becomes eligible for incomplete cleanup (older than cutoff and status != success):
- If no task exists: create `queued`, `attempts=0`, `next_attempt_at=now`, snapshot from `runs.target_snapshot_json`.
- If task exists:
  - If `done/ignored/abandoned`: do nothing.
  - Else: do not change status; keep scheduling fields.

### Claiming & Concurrency

The cleanup worker claims tasks by atomically moving `queued/retrying/blocked` → `running`.
If the process crashes while `running`, a watchdog rule can reset stuck tasks:
- If `status=running` and `last_attempt_at < now - RUNNING_TTL` then set back to `retrying` and schedule a near-future retry.

### Outcomes

On attempt success:
- If cleanup performed or confirmed unnecessary (`complete exists`, `remote missing`, `local absent/not bastion`): set `done`.

On attempt failure:
- Classify error:
  - **network**: DNS, connect, timeout
  - **http**: non-auth HTTP errors
  - **auth/config**: missing secret, invalid URL, 401/403
- Update:
  - `attempts += 1`
  - `last_attempt_at = now`
  - `last_error_kind/last_error`
  - `status = blocked` for auth/config, else `retrying`
  - `next_attempt_at = now + backoff(attempts, kind)`

### Abandon

Auto abandon when:
- `attempts >= MAX_ATTEMPTS` (default 20) OR
- `now - created_at >= MAX_AGE_SECONDS` (default 30 days)

Abandon behavior:
- `status=abandoned`
- stop automatic retries
- UI can still `retry-now` which resets to `queued` (and optionally resets attempts).

### Ignore / Unignore

Ignore:
- `status=ignored`, write event, stop retries.

Unignore:
- set `status=queued`, `next_attempt_at=now`, write event.

## Scheduling Behavior

Current tight loop is replaced with a bounded batch per tick:
1. Reconcile tasks for eligible runs (create missing tasks).
2. Query due tasks: `status in (...) AND next_attempt_at <= now LIMIT N`.
3. Claim and process each task once.
4. End tick. Next tick runs after a fixed interval (e.g. 1h) or when notified by operator actions.

This guarantees:
- no infinite inner loop
- predictable log volume
- bounded work per tick

## HTTP API

Pattern: similar to notification queue endpoints (pagination + actions).

All endpoints require session + CSRF for mutations.

### List tasks
`GET /api/maintenance/incomplete-cleanup`
Query:
- `status` (optional)
- `target_type` (optional)
- `node_id` (optional)
- `job_id` (optional)
- `page`, `page_size`

Response:
```json
{
  "items": [
    {
      "run_id": "...",
      "job_id": "...",
      "job_name": "...",
      "node_id": "hub|...",
      "target_type": "webdav|local_dir",
      "status": "queued|running|retrying|blocked|done|ignored|abandoned",
      "attempts": 3,
      "last_attempt_at": 123,
      "next_attempt_at": 456,
      "last_error_kind": "network",
      "last_error": "error sending request ...",
      "created_at": 111,
      "updated_at": 222
    }
  ],
  "page": 1,
  "page_size": 20,
  "total": 123
}
```

### Get task + events
`GET /api/maintenance/incomplete-cleanup/{run_id}`
Response includes `task` + recent `events` (with pagination optional if needed later).

### Actions
- `POST /api/maintenance/incomplete-cleanup/{run_id}/retry-now`
- `POST /api/maintenance/incomplete-cleanup/{run_id}/ignore` (body `{ reason?: string }`)
- `POST /api/maintenance/incomplete-cleanup/{run_id}/unignore`

These endpoints should notify the cleanup worker (`Notify`) to avoid waiting until the next scheduled tick.

### Job archive (non-cascade delete)
- `POST /api/jobs/{id}/archive`
- `POST /api/jobs/{id}/unarchive` (optional; can be deferred)

Hard delete remains:
- `DELETE /api/jobs/{id}` (existing; cascades runs and therefore cleanup tasks)

UI “Delete” dialog offers:
- Archive (recommended, keeps history)
- Delete permanently (cascade)

## UI Design (Mobile Friendly)

### Navigation
Add a Settings subpage:
`/settings/maintenance/cleanup` (hub scope) and `/n/:nodeId/settings/maintenance/cleanup` (node scope, if needed).

Add to menu:
Settings → Maintenance → Incomplete Cleanup

### Layout
- Desktop (`MQ.mdUp`): `NDataTable` with full columns + actions.
- Mobile: card rows (like existing patterns in Notifications queue) where each item renders as a compact stacked layout:
  - top: job_name + status tag
  - middle: run_id (truncated) + node + target
  - bottom: last_error + next_attempt_at
  - actions: small buttons in a row; overflow menu if needed

### UX Requirements
- Status tags are color coded.
- “Retry now” disabled when `status=running` (or show “busy”).
- “Ignore” prompts confirm with optional reason.
- All URLs displayed are redacted.

## Security & Privacy

- Never expose secret payloads.
- Redact WebDAV URL: remove userinfo, query, fragment.
- Limit `last_error` length and strip embedded credentials (defense in depth).

## Migration & Rollout

1. Add migrations: jobs archive column, runs target snapshot, cleanup tables.
2. Implement writing `runs.target_snapshot_json` at run start.
3. Implement cleanup worker with queue + backoff + abandon.
4. Add HTTP endpoints.
5. Add UI page and delete dialog update.

Backwards compatibility:
- Existing data: runs without snapshot will fall back to current job spec when creating tasks (best-effort).

## Testing Plan

- Storage tests: create/list/claim/update tasks, event sequencing, filtering/pagination.
- Engine tests: backoff computation, abandon thresholds, stuck `running` recovery.
- HTTP tests: list endpoints, auth/CSRF enforcement, action endpoints.
- UI tests (Vitest): list rendering, mobile vs desktop layout smoke tests, action calls.

