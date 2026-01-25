## 1. Spec & Design
- [ ] Confirm queue/status model (queued/running/retrying/blocked/abandoned/done/ignored)
- [ ] Define error classification rules (config/auth/network/http/unknown) aligned with incomplete cleanup
- [ ] Validate this change with `openspec validate add-backup-snapshots-delete-queue --strict`

## 2. DB & Storage Layer
- [ ] Add migrations:
  - `artifact_delete_tasks`
  - `artifact_delete_events`
- [ ] Add `artifact_delete_repo` in `crates/bastion-storage`:
  - enqueue/upsert task
  - claim next due task
  - transitions (mark running/retrying/blocked/abandoned/done/ignored)
  - append event + list events
- [ ] Add storage tests for claim/transition correctness and idempotency

## 3. Engine: Delete Worker Loop (Hub)
- [ ] Add a scheduler loop to process snapshot delete tasks
- [ ] Implement deletion execution for:
  - `webdav`: delete `<base>/<job_id>/<run_id>/` (idempotent)
  - `local_dir` (Hub node only): delete `<base>/<job_id>/<run_id>/` with safety checks
- [ ] Apply retry/backoff rules; write events for each attempt and state transition
- [ ] Ensure "target not found" is treated as success
- [ ] Add engine tests for retry/backoff edge cases (fast failures, 404, auth errors)

## 4. HTTP API
- [ ] Enqueue deletion:
  - `POST /api/jobs/:job_id/snapshots/:run_id/delete`
  - `POST /api/jobs/:job_id/snapshots/delete` (bulk)
- [ ] Query task state/events:
  - `GET /api/jobs/:job_id/snapshots/:run_id/delete-task`
  - `GET /api/jobs/:job_id/snapshots/:run_id/delete-events`
- [ ] Operator actions:
  - `POST /api/jobs/:job_id/snapshots/:run_id/delete/retry-now`
  - `POST /api/jobs/:job_id/snapshots/:run_id/delete/ignore`
- [ ] Add HTTP tests for auth + bad input + happy path

## 5. Web UI
- [ ] Add delete action(s) to the snapshots page (single + bulk)
- [ ] Add confirmation dialog that lists selected snapshots
- [ ] Show per-snapshot deletion state (attempts, last error kind + summary)
- [ ] Provide an event log viewer (drawer/modal) for delete events
- [ ] Add UI tests for delete flow wiring (mock API)

