## 1. Spec
- [x] 1.1 Add spec deltas for bulk operations framework + initial label action
- [x] 1.2 Run `openspec validate add-bulk-operations-framework --type change --strict`

## 2. Backend
- [x] 2.1 Storage: add `bulk_operations` + `bulk_operation_items` tables and repo helpers
- [x] 2.2 Engine: add bulk worker loop with bounded concurrency
- [x] 2.3 HTTP: add authenticated APIs (create/list/get/cancel/retry-failed)
- [x] 2.4 Implement initial bulk action: add/remove agent labels
- [x] 2.5 Add backend unit tests for state transitions and retry semantics

## 3. Web UI
- [x] 3.1 Add store/API wiring for bulk operations
- [x] 3.2 Add bulk operations panel/page (list + detail + retry/cancel)
- [x] 3.3 Agents page: entry point to create bulk label operations (selector-based)
- [x] 3.4 Add/adjust unit tests

## 4. Validation
- [x] 4.1 Run `bash scripts/ci.sh`

## 5. Commits
- [x] 5.1 Commit spec proposal (detailed message)
- [x] 5.2 Commit implementation (detailed message)
- [x] 5.3 Mark tasks complete and commit
