## 1. Spec & Design
- [x] Define cascade semantics:
  - include/exclude pinned snapshots
  - how to handle snapshots already deleting/deleted
- [x] Validate this change with `openspec validate add-job-archive-cascade-snapshots --strict`

## 2. Backend (Jobs API)
- [x] Extend job archive endpoint to accept a cascade option (boolean)
- [x] When cascade is requested:
  - list snapshots for the job (`run_artifacts.status=present`)
  - enqueue deletion tasks in a bounded way (to avoid huge requests)
- [x] Ensure archive succeeds even if enqueue partially fails (best-effort + surfaced errors)
- [x] Add HTTP tests for archive with/without cascade

## 3. Web UI
- [x] Update job archive action UI:
  - add a checkbox/switch "同时删除备份数据"
  - show a warning and (optionally) the count of snapshots affected
- [x] If pinned snapshots exist:
  - require an extra confirmation for force, or
  - clearly state that pinned snapshots are excluded (depending on final policy)
- [x] Add UI tests for archive confirmation behavior
