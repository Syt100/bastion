## 1. Spec & Design
- [x] Ensure the snapshot management design doc exists and is up to date: `docs/dev/design/backup-snapshots.md`
- [x] Define the snapshot index schema and required fields (status, target snapshot, metrics)
- [x] Validate this change with `openspec validate add-backup-snapshots-index --strict`

## 2. DB & Storage Layer
- [x] Add a migration to create `run_artifacts` (snapshot index) and required indexes
- [x] Add a `run_artifacts_repo` in `crates/bastion-storage` (upsert/get/list; filtering + pagination)
- [x] Add storage-layer tests for insert/upsert and list filters

## 3. Indexing on Successful Runs
- [x] On run success, upsert a snapshot row:
  - `run_id`, `job_id`, `node_id`, `target_type`, `target_snapshot_json`
  - `artifact_format` and core metrics (files/dirs/bytes; transfer bytes when available)
- [x] Ensure indexing uses the run-time target snapshot, not the current job spec
- [x] Keep indexing best-effort: snapshot indexing failure must not fail the run

## 4. HTTP API (Read-only)
- [x] `GET /api/jobs/:job_id/snapshots` (filters + pagination)
- [x] `GET /api/jobs/:job_id/snapshots/:run_id` (snapshot detail)
- [x] Add HTTP tests for list/get + auth checks

## 5. Web UI (Independent Page)
- [x] Add route: `/jobs/:job_id/snapshots`
- [x] Add navigation entry from Jobs UI (job row/menu → "备份数据"/Snapshots)
- [x] Implement a responsive table/list:
  - ended time, status, format, target summary, size/count metrics
  - link to run detail
- [x] Add UI unit tests for the new route and basic rendering
