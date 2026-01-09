## 1. Spec
- [x] 1.1 Add `backend` spec delta for: offline sync module modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-agent-client-offline-sync-modules --strict`

## 2. Agent - Offline sync module modularization
- [x] 2.1 Identify responsibilities and module boundaries (request DTOs, events loading, http ingest, tests)
- [x] 2.2 Convert `offline/sync.rs` into a folder module and keep entrypoint stable
- [x] 2.3 Extract ingest request types into `crates/bastion/src/agent_client/offline/sync/request.rs`
- [x] 2.4 Extract events jsonl loader into `crates/bastion/src/agent_client/offline/sync/events.rs`
- [x] 2.5 Extract HTTP ingest helper into `crates/bastion/src/agent_client/offline/sync/ingest.rs`
- [x] 2.6 Move unit tests into `crates/bastion/src/agent_client/offline/sync/tests.rs`
- [x] 2.7 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
