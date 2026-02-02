## 1. Spec
- [x] 1.1 Add `backend` spec delta for background task supervision and fail-fast behavior
- [x] 1.2 Run `openspec validate update-scheduler-supervision --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Backend - supervision infrastructure
- [x] 2.1 Introduce a small `spawn_supervised(...)` helper for long-running tasks
- [x] 2.2 Ensure panics in supervised tasks are logged and cancel the shared shutdown token
- [x] 2.3 Add unit tests for the supervision helper

## 3. Backend - apply supervision to critical loops
- [x] 3.1 Apply supervision to scheduler loops (`scheduler::spawn`)
- [x] 3.2 Apply supervision to notifications/bulk/maintenance long-running loops
- [x] 3.3 Ensure `cargo test --workspace` passes
- [x] 3.4 Commit task supervision changes (detailed message)
