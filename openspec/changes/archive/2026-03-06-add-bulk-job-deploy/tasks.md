## 1. Spec
- [x] 1.1 Add spec deltas for bulk job deploy (preview + naming template + per-node validation)
- [x] 1.2 Run `openspec validate add-bulk-job-deploy --type change --strict`

## 2. Backend
- [x] 2.1 Bulk ops: add action “deploy job to nodes”
- [x] 2.2 Implement naming template defaults and collision handling
- [x] 2.3 Implement per-node preflight validation and error summaries
- [x] 2.4 Implement preview capability for UI (dry-run plan)
- [x] 2.5 Add backend tests (validation, naming, partial failures)

## 3. Web UI
- [x] 3.1 Jobs page: add “Deploy to nodes” entry point
- [x] 3.2 Add selector UI (labels AND/OR) + naming template input
- [x] 3.3 Add preview UI (planned names + validation results)
- [x] 3.4 Trigger bulk operation and link to bulk results
- [x] 3.5 Add/adjust unit tests

## 4. Validation
- [x] 4.1 Run `bash scripts/ci.sh`

## 5. Commits
- [x] 5.1 Commit spec proposal (detailed message)
- [x] 5.2 Commit implementation (detailed message)
- [x] 5.3 Mark tasks complete and commit
