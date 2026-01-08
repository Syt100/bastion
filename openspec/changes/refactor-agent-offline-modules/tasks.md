## 1. Spec
- [x] 1.1 Add `backend` spec delta for: agent offline modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-agent-offline-modules --strict`

## 2. Agent - Offline modularization
- [x] 2.1 Identify responsibilities and module boundaries (`cron`, `storage`, `scheduler`, `sync`)
- [x] 2.2 Split `crates/bastion/src/agent_client/offline.rs` into focused submodules under `crates/bastion/src/agent_client/offline/`
- [x] 2.3 Keep `offline` module surface stable; update internal imports/tests as needed
- [x] 2.4 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
