## 1. Spec
- [x] 1.1 Add `backend` spec delta for: offline storage module modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-agent-client-offline-storage-modules --strict`

## 2. Agent - Offline storage module modularization
- [x] 2.1 Identify responsibilities and module boundaries (paths, types, writer, IO helpers)
- [x] 2.2 Convert `offline/storage.rs` into a folder module and keep public entrypoints stable
- [x] 2.3 Extract path helpers into `crates/bastion/src/agent_client/offline/storage/paths.rs`
- [x] 2.4 Extract on-disk types into `crates/bastion/src/agent_client/offline/storage/types.rs`
- [x] 2.5 Extract writer handle into `crates/bastion/src/agent_client/offline/storage/writer.rs`
- [x] 2.6 Extract IO helpers into `crates/bastion/src/agent_client/offline/storage/io.rs`
- [x] 2.7 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
