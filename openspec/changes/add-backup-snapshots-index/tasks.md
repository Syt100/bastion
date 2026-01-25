## 1. Spec & Design
- [ ] Ensure the snapshot management design doc exists and is up to date: `docs/backup-snapshots.md`
- [ ] Define the snapshot index schema and required fields (status, target snapshot, metrics)
- [ ] Validate this change with `openspec validate add-backup-snapshots-index --strict`

## 2. DB & Storage Layer
- [ ] Add a migration to create `run_artifacts` (snapshot index) and required indexes
- [ ] Add a `run_artifacts_repo` in `crates/bastion-storage` (upsert/get/list; filtering + pagination)
- [ ] Add storage-layer tests for insert/upsert and list filters

## 3. Indexing on Successful Runs
- [ ] On run success, upsert a snapshot row:
  - `run_id`, `job_id`, `node_id`, `target_type`, `target_snapshot_json`
  - `artifact_format` and core metrics (files/dirs/bytes; transfer bytes when available)
- [ ] Ensure indexing uses the run-time target snapshot, not the current job spec
- [ ] Keep indexing best-effort: snapshot indexing failure must not fail the run

## 4. HTTP API (Read-only)
- [ ] `GET /api/jobs/:job_id/snapshots` (filters + pagination)
- [ ] `GET /api/jobs/:job_id/snapshots/:run_id` (snapshot detail)
- [ ] Add HTTP tests for list/get + auth checks

## 5. Web UI (Independent Page)
- [ ] Add route: `/jobs/:job_id/snapshots`
- [ ] Add navigation entry from Jobs UI (job row/menu → "备份数据"/Snapshots)
- [ ] Implement a responsive table/list:
  - ended time, status, format, target summary, size/count metrics
  - link to run detail
- [ ] Add UI unit tests for the new route and basic rendering

