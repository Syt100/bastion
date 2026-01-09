## 1. Spec
- [x] 1.1 Add `backend` spec delta for: agent offline entrypoint relocation (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-agent-client-offline-entrypoint-modules --strict`

## 2. Agent - Offline entrypoint relocation
- [ ] 2.1 Identify module boundaries (`cron`, `scheduler`, `storage`, `sync`, entrypoint exports)
- [ ] 2.2 Move `crates/bastion/src/agent_client/offline.rs` to `crates/bastion/src/agent_client/offline/mod.rs`
- [ ] 2.3 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
