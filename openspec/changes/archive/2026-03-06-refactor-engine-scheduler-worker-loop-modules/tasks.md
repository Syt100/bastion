## 1. Spec
- [x] 1.1 Add `backend` spec delta for: scheduler worker loop module split (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-engine-scheduler-worker-loop-modules --strict`

## 2. Engine - Scheduler worker loop module split
- [x] 2.1 Identify responsibilities and module boundaries (args, worker loop orchestration, dispatch/execute submodules)
- [x] 2.2 Extract worker loop implementation into `crates/bastion-engine/src/scheduler/worker/loop.rs`
- [x] 2.3 Keep `WorkerLoopArgs` and `run_worker_loop` entrypoints stable in `worker/mod.rs`
- [x] 2.4 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
