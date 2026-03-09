## 1. Spec
- [x] 1.1 Add `backend` spec delta for: scheduler worker loop submodule split (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-engine-scheduler-worker-loop-submodules --strict`

## 2. Engine - Scheduler worker loop submodule split
- [x] 2.1 Identify phase boundaries (claim/job+spec/agent/local/notifications)
- [x] 2.2 Convert `crates/bastion-engine/src/scheduler/worker/loop.rs` into folder module and keep entrypoints stable
- [x] 2.3 Extract phase logic into focused submodules and helpers
- [x] 2.4 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
