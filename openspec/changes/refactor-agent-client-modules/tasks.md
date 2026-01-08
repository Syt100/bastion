## 1. Spec
- [x] 1.1 Add `backend` spec delta for: agent client modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-agent-client-modules --strict`

## 2. Backend - Agent client modularization
- [x] 2.1 Identify responsibilities and module boundaries (`identity`, `connect`, `tasks`, `targets`, `fs_list`)
- [x] 2.2 Extract identity/enrollment helpers into `crates/bastion/src/agent_client/identity.rs`
- [x] 2.3 Extract websocket connect + message loop into `crates/bastion/src/agent_client/connect.rs`
- [x] 2.4 Extract task handling (backup + control) into `crates/bastion/src/agent_client/tasks.rs`
- [x] 2.5 Extract target storage helpers into `crates/bastion/src/agent_client/targets.rs`
- [x] 2.6 Extract filesystem list helpers into `crates/bastion/src/agent_client/fs_list.rs`
- [x] 2.7 Keep `agent_client::run` stable in `mod.rs`; keep offline scheduler integration working
- [x] 2.8 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
