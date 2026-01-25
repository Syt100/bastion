## 1. Spec & Design
- [x] Confirm queue/status model (queued/running/retrying/blocked/abandoned/done/ignored)
- [x] Define error classification rules (config/auth/network/http/unknown) aligned with incomplete cleanup
- [x] Validate this change with `openspec validate add-backup-snapshots-delete-queue --strict`

## 2. DB & Storage Layer
- [x] Add migrations:
  - `artifact_delete_tasks`
  - `artifact_delete_events`
- [x] Add `artifact_delete_repo` in `crates/bastion-storage`:
  - enqueue/upsert task
  - claim next due task
  - transitions (mark running/retrying/blocked/abandoned/done/ignored)
  - append event + list events
- [x] Add storage tests for claim/transition correctness and idempotency

## 3. Engine: Delete Worker Loop (Hub)
- [x] Add a scheduler loop to process snapshot delete tasks
- [x] Implement deletion execution for:
  - `webdav`: delete `<base>/<job_id>/<run_id>/` (idempotent)
  - `local_dir` (Hub node only): delete `<base>/<job_id>/<run_id>/` with safety checks
- [x] Apply retry/backoff rules; write events for each attempt and state transition
- [x] Ensure "target not found" is treated as success
- [x] Add engine tests for retry/backoff edge cases (fast failures, 404, auth errors)

## 4. HTTP API
- [x] Enqueue deletion:
  - `POST /api/jobs/:job_id/snapshots/:run_id/delete`
  - `POST /api/jobs/:job_id/snapshots/delete` (bulk)
- [x] Query task state/events:
  - `GET /api/jobs/:job_id/snapshots/:run_id/delete-task`
  - `GET /api/jobs/:job_id/snapshots/:run_id/delete-events`
- [x] Operator actions:
  - `POST /api/jobs/:job_id/snapshots/:run_id/delete/retry-now`
  - `POST /api/jobs/:job_id/snapshots/:run_id/delete/ignore`
- [x] Add HTTP tests for auth + bad input + happy path

## 5. Web UI
- [x] Add delete action(s) to the snapshots page (single + bulk)
- [x] Add confirmation dialog that lists selected snapshots
- [x] Show per-snapshot deletion state (attempts, last error kind + summary)
- [x] Provide an event log viewer (drawer/modal) for delete events
- [x] Add UI tests for delete flow wiring (mock API)
