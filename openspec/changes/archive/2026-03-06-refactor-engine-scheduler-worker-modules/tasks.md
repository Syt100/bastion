## 1. Spec
- [x] 1.1 Add `backend` spec delta for: scheduler worker modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-engine-scheduler-worker-modules --strict`

## 2. Backend - Scheduler worker modularization
- [x] 2.1 Identify responsibilities and module boundaries (`loop`, `dispatch`, `execute`, `target_store`)
- [x] 2.2 Extract dispatch-to-agent logic into `crates/bastion-engine/src/scheduler/worker/dispatch.rs`
- [x] 2.3 Extract local run execution logic into `crates/bastion-engine/src/scheduler/worker/execute.rs`
- [x] 2.4 Extract target artifact storage into `crates/bastion-engine/src/scheduler/worker/target_store.rs`
- [x] 2.5 Keep `WorkerLoopArgs` and `run_worker_loop` stable in `worker.rs`; update internal imports as needed
- [x] 2.6 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
