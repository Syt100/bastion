## 1. Spec
- [x] 1.1 Add `backend` spec delta for: scheduler worker execute module split (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-engine-scheduler-worker-execute-modules --strict`

## 2. Engine - Scheduler worker execute module split
- [x] 2.1 Identify module boundaries (shared context vs per-job-type execution)
- [x] 2.2 Convert `worker/execute.rs` into folder module and keep exports stable
- [x] 2.3 Extract filesystem execution into `worker/execute/filesystem.rs`
- [x] 2.4 Extract sqlite execution into `worker/execute/sqlite.rs`
- [x] 2.5 Extract vaultwarden execution into `worker/execute/vaultwarden.rs`
- [x] 2.6 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
