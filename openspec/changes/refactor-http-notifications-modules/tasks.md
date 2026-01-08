## 1. Spec
- [x] 1.1 Add `backend` spec delta for: HTTP notifications modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-http-notifications-modules --strict`

## 2. Backend - HTTP notifications modularization
- [x] 2.1 Identify responsibilities and module boundaries (`settings`, `destinations`, `queue`, `validation`)
- [x] 2.2 Split `crates/bastion-http/src/http/notifications.rs` into focused submodules under `crates/bastion-http/src/http/notifications/`
- [x] 2.3 Keep `notifications` public surface stable; update internal imports/tests and ensure router compiles
- [x] 2.4 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
