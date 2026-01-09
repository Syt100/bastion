## 1. Spec
- [x] 1.1 Add `backend` spec delta for: scheduler worker entrypoint relocation (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-engine-scheduler-worker-entrypoint-modules --strict`

## 2. Engine - Scheduler worker entrypoint relocation
- [x] 2.1 Identify module boundaries (`dispatch`, `execute`, `target_store`, worker loop entrypoint)
- [x] 2.2 Move `crates/bastion-engine/src/scheduler/worker.rs` to `crates/bastion-engine/src/scheduler/worker/mod.rs`
- [x] 2.3 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
