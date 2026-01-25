## 1. Spec & Design
- [ ] Define pin semantics (manual delete requires force; retention excludes pinned)
- [ ] Validate this change with `openspec validate add-backup-snapshots-pinning --strict`

## 2. DB & Storage Layer
- [ ] Add migration to extend `run_artifacts` with:
  - `pinned_at`
  - `pinned_by_user_id`
- [ ] Add repo methods: pin/unpin + query pinned state
- [ ] Add storage tests for pin/unpin behavior

## 3. HTTP API
- [ ] `POST /api/jobs/:job_id/snapshots/:run_id/pin`
- [ ] `POST /api/jobs/:job_id/snapshots/:run_id/unpin`
- [ ] Update deletion enqueue APIs to reject pinned snapshots unless `force=true`
- [ ] Add HTTP tests for pin/unpin + deletion guardrails

## 4. Web UI
- [ ] Show pin state in snapshots list (icon + tooltip)
- [ ] Provide pin/unpin actions (single + bulk optional)
- [ ] Update delete confirm dialog to:
  - highlight pinned items
  - require an extra explicit confirmation for force delete

