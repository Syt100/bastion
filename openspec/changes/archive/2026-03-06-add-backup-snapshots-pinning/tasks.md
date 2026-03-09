## 1. Spec & Design
- [x] Define pin semantics (manual delete requires force; retention excludes pinned)
- [x] Validate this change with `openspec validate add-backup-snapshots-pinning --strict`

## 2. DB & Storage Layer
- [x] Add migration to extend `run_artifacts` with:
  - `pinned_at`
  - `pinned_by_user_id`
- [x] Add repo methods: pin/unpin + query pinned state
- [x] Add storage tests for pin/unpin behavior

## 3. HTTP API
- [x] `POST /api/jobs/:job_id/snapshots/:run_id/pin`
- [x] `POST /api/jobs/:job_id/snapshots/:run_id/unpin`
- [x] Update deletion enqueue APIs to reject pinned snapshots unless `force=true`
- [x] Add HTTP tests for pin/unpin + deletion guardrails

## 4. Web UI
- [x] Show pin state in snapshots list (icon + tooltip)
- [x] Provide pin/unpin actions (single + bulk optional)
- [x] Update delete confirm dialog to:
  - highlight pinned items
  - require an extra explicit confirmation for force delete
