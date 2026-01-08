## 1. Spec
- [x] 1.1 Add `backend` spec delta for: HTTP jobs modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-http-jobs-modules --strict`

## 2. Backend - HTTP jobs modularization
- [x] 2.1 Identify responsibilities and module boundaries (`validation`, `crud`, `runs`, `ws`)
- [x] 2.2 Split `crates/bastion-http/src/http/jobs.rs` into focused submodules under `crates/bastion-http/src/http/jobs/`
- [x] 2.3 Keep `jobs` public surface stable; update internal imports/tests and ensure sibling modules compile
- [x] 2.4 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
