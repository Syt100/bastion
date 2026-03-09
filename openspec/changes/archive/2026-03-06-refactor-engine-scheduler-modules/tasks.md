## 1. Spec
- [x] 1.1 Add `backend` spec delta for: scheduler modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-engine-scheduler-modules --strict`

## 2. Backend - Scheduler modularization
- [x] 2.1 Identify scheduler responsibilities and module boundaries
- [x] 2.2 Extract cron helpers/types into `crates/bastion-engine/src/scheduler/cron.rs`
- [x] 2.3 Extract queue/DB helpers into `crates/bastion-engine/src/scheduler/queue.rs` (or similar)
- [x] 2.4 Extract scheduling policy/orchestration helpers into focused modules
- [x] 2.5 Keep public API stable; update internal imports/tests as needed
- [x] 2.6 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
