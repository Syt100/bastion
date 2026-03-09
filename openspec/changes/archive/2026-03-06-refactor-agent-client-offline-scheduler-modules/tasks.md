## 1. Spec
- [x] 1.1 Add `backend` spec delta for: agent offline scheduler modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-agent-client-offline-scheduler-modules --strict`

## 2. Agent - Offline scheduler modularization
- [x] 2.1 Identify responsibilities and module boundaries (`types`, `cron_loop`, `worker_loop`, `sink`)
- [x] 2.2 Convert `scheduler.rs` into a folder module and keep `offline_scheduler_loop` stable
- [x] 2.3 Extract cron scheduling loop into `crates/bastion/src/agent_client/offline/scheduler/cron_loop.rs`
- [x] 2.4 Extract worker execution loop into `crates/bastion/src/agent_client/offline/scheduler/worker_loop.rs`
- [x] 2.5 Extract websocket sink + summary helpers into `crates/bastion/src/agent_client/offline/scheduler/sink.rs`
- [x] 2.6 Extract shared structs into `crates/bastion/src/agent_client/offline/scheduler/types.rs`
- [x] 2.7 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
