## 1. Spec
- [x] 1.1 Add spec deltas for bulk job deploy (preview + naming template + per-node validation)
- [x] 1.2 Run `openspec validate add-bulk-job-deploy --type change --strict`

## 2. Backend
- [ ] 2.1 Bulk ops: add action “deploy job to nodes”
- [ ] 2.2 Implement naming template defaults and collision handling
- [ ] 2.3 Implement per-node preflight validation and error summaries
- [ ] 2.4 Implement preview capability for UI (dry-run plan)
- [ ] 2.5 Add backend tests (validation, naming, partial failures)

## 3. Web UI
- [ ] 3.1 Jobs page: add “Deploy to nodes” entry point
- [ ] 3.2 Add selector UI (labels AND/OR) + naming template input
- [ ] 3.3 Add preview UI (planned names + validation results)
- [ ] 3.4 Trigger bulk operation and link to bulk results
- [ ] 3.5 Add/adjust unit tests

## 4. Validation
- [ ] 4.1 Run `bash scripts/ci.sh`

## 5. Commits
- [ ] 5.1 Commit spec proposal (detailed message)
- [ ] 5.2 Commit implementation (detailed message)
- [ ] 5.3 Mark tasks complete and commit
