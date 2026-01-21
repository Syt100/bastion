## 1. Spec
- [x] 1.1 Add `backend` spec delta for operation subject linking
- [x] 1.2 Run `openspec validate add-operation-subject-link --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Backend
- [x] 2.1 Add DB migration: `operations.subject_kind` + `operations.subject_id` (+ index)
- [x] 2.2 Update storage repo APIs to create operations with a subject reference
- [x] 2.3 Add `GET /api/runs/{run_id}/operations`
- [x] 2.4 Add/update unit tests

## 3. Validation
- [x] 3.1 Run `cargo test --workspace`

## 4. Commits
- [x] 4.1 Commit backend changes (detailed message with Modules/Tests)
