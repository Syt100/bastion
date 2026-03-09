## 1. Spec
- [x] 1.1 Add `backend` spec delta for: agent task handling module modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-agent-client-tasks-modules --strict`

## 2. Agent - Tasks module modularization
- [x] 2.1 Identify responsibilities and module boundaries (events, encryption mapping, filesystem/sqlite/vaultwarden handlers)
- [x] 2.2 Convert `tasks.rs` into a folder module and keep public entrypoints stable
- [x] 2.3 Extract filesystem backup task handler into `crates/bastion/src/agent_client/tasks/filesystem.rs`
- [x] 2.4 Extract sqlite backup task handler into `crates/bastion/src/agent_client/tasks/sqlite.rs`
- [x] 2.5 Extract vaultwarden backup task handler into `crates/bastion/src/agent_client/tasks/vaultwarden.rs`
- [x] 2.6 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
