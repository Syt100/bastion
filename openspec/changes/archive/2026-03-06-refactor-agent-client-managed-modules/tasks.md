## 1. Spec
- [x] 1.1 Add `backend` spec delta for: agent managed module modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-agent-client-managed-modules --strict`

## 2. Agent - Managed module modularization
- [x] 2.1 Identify responsibilities and module boundaries (`paths`, `secrets_snapshot`, `config_snapshot`, `task_results`, `io`)
- [x] 2.2 Convert `managed.rs` into a folder module and keep public entrypoints stable
- [x] 2.3 Extract snapshot path helpers into `crates/bastion/src/agent_client/managed/paths.rs`
- [x] 2.4 Extract secrets snapshot logic into `crates/bastion/src/agent_client/managed/secrets_snapshot.rs`
- [x] 2.5 Extract config snapshot logic into `crates/bastion/src/agent_client/managed/config_snapshot.rs`
- [x] 2.6 Extract task result helpers into `crates/bastion/src/agent_client/managed/task_results.rs`
- [x] 2.7 Extract atomic json write helper into `crates/bastion/src/agent_client/managed/io.rs`
- [x] 2.8 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
