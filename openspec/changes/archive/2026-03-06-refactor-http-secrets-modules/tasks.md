## 1. Spec
- [x] 1.1 Add `backend` spec delta for: HTTP secrets modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-http-secrets-modules --strict`

## 2. Backend - HTTP secrets modularization
- [x] 2.1 Identify responsibilities and module boundaries (`node_validation`, `webdav`, `wecom_bot`, `smtp`)
- [x] 2.2 Split `crates/bastion-http/src/http/secrets.rs` into focused submodules under `crates/bastion-http/src/http/secrets/`
- [x] 2.3 Keep `secrets` public surface stable; update internal imports/tests and ensure router compiles
- [x] 2.4 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
