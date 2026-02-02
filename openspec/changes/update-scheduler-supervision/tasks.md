## 1. Spec
- [x] 1.1 Add `backend` spec delta for background task supervision and fail-fast behavior
- [x] 1.2 Run `openspec validate update-scheduler-supervision --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Backend - supervision infrastructure
- [ ] 2.1 Introduce a small `spawn_supervised(...)` helper for long-running tasks
- [ ] 2.2 Ensure panics in supervised tasks are logged and cancel the shared shutdown token
- [ ] 2.3 Add unit tests for the supervision helper

## 3. Backend - apply supervision to critical loops
- [ ] 3.1 Apply supervision to scheduler loops (`scheduler::spawn`)
- [ ] 3.2 Apply supervision to notifications/bulk/maintenance long-running loops
- [ ] 3.3 Ensure `cargo test --workspace` passes
- [ ] 3.4 Commit task supervision changes (detailed message)
