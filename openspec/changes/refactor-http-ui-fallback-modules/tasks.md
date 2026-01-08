## 1. Spec
- [x] 1.1 Add `backend` spec delta for: HTTP UI fallback modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-http-ui-fallback-modules --strict`

## 2. Backend - HTTP UI fallback modularization
- [x] 2.1 Identify module boundary (router vs UI fallback helpers)
- [x] 2.2 Move UI fallback + helper functions into `crates/bastion-http/src/http/ui.rs`
- [x] 2.3 Update `crates/bastion-http/src/http/mod.rs` to call the extracted fallback handler
- [x] 2.4 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
