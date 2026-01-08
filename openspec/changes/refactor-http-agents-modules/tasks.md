## 1. Spec
- [x] 1.1 Add `backend` spec delta for: HTTP agents modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-http-agents-modules --strict`

## 2. Backend - HTTP agents modularization
- [x] 2.1 Identify responsibilities and module boundaries (`admin`, `enrollment`, `ingest`, `ws`, `snapshots`, `agent_auth`)
- [x] 2.2 Split `crates/bastion-http/src/http/agents.rs` into focused submodules under `crates/bastion-http/src/http/agents/`
- [x] 2.3 Keep `agents` public surface stable; update internal imports/tests and ensure sibling modules compile
- [x] 2.4 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
