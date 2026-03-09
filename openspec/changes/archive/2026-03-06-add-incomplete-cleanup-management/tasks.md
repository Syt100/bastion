## 1. Spec
- [x] 1.1 Add `backend` + `web-ui` spec deltas for: incomplete cleanup queue + UI + job archive delete option
- [x] 1.2 Write `design.md` for data model, API, UI (mobile)
- [x] 1.3 Run `openspec validate add-incomplete-cleanup-management --strict`

## 2. Storage (schema + repos)
- [x] 2.1 Add migrations:
  - `jobs.archived_at`
  - `runs.target_snapshot_json`
  - `incomplete_cleanup_tasks` + `incomplete_cleanup_events`
- [x] 2.2 Add `bastion-storage` repo helpers for tasks/events (create/list/claim/update/ignore/retry)
- [x] 2.3 Add unit tests for the new storage repos

## 3. Engine (snapshot + cleanup worker)
- [x] 3.1 Write `runs.target_snapshot_json` when a run starts (post-validate, pre-dispatch/execute)
- [x] 3.2 Replace incomplete cleanup loop with queue-driven processing (bounded batches)
- [x] 3.3 Implement error classification + backoff + abandon + running-TTL recovery
- [x] 3.4 Ensure logs are rate-limited (no per-run tight-loop spam)

## 4. HTTP API
- [x] 4.1 Add `/api/maintenance/incomplete-cleanup` endpoints (list/get/events/actions) with auth + CSRF
- [x] 4.2 Add job archive endpoints and update jobs listing to hide archived by default (with an optional query to include)

## 5. UI (mobile friendly)
- [x] 5.1 Add a Maintenance page for incomplete cleanup tasks (desktop table + mobile card layout)
- [x] 5.2 Add operator actions (retry/ignore/unignore) and task details modal
- [x] 5.3 Update job “delete” UX to offer Archive vs Delete permanently (cascade)

## 6. Verification
- [x] 6.1 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
- [x] 6.2 Run `cd ui && npm test` and `npm run type-check`
