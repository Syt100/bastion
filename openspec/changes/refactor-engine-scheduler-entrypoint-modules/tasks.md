## 1. Spec
- [x] 1.1 Add `backend` spec delta for: scheduler entrypoint relocation (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-engine-scheduler-entrypoint-modules --strict`

## 2. Engine - Scheduler entrypoint relocation
- [ ] 2.1 Identify module boundaries (`cron`, `worker`, queueing, retention, cleanup, scheduler entrypoint)
- [ ] 2.2 Move `crates/bastion-engine/src/scheduler.rs` to `crates/bastion-engine/src/scheduler/mod.rs`
- [ ] 2.3 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
