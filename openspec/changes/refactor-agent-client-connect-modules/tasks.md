## 1. Spec
- [x] 1.1 Add `backend` spec delta for: agent websocket connect module modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-agent-client-connect-modules --strict`

## 2. Agent - Connect module modularization
- [x] 2.1 Identify responsibilities and module boundaries (handshake, message handling, heartbeat)
- [x] 2.2 Convert `connect.rs` into a folder module and keep public entrypoints stable
- [x] 2.3 Extract hello/handshake helpers into `crates/bastion/src/agent_client/connect/handshake.rs`
- [x] 2.4 Extract per-message handlers into `crates/bastion/src/agent_client/connect/handlers.rs`
- [x] 2.5 Extract heartbeat/ping-pong helpers into `crates/bastion/src/agent_client/connect/heartbeat.rs`
- [x] 2.6 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
