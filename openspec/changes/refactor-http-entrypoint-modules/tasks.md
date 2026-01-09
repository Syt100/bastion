## 1. Spec
- [x] 1.1 Add `backend` spec delta for: HTTP entrypoint relocation to directory modules (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-http-entrypoint-modules --strict`

## 2. HTTP - Entrypoint relocation to directory modules
- [x] 2.1 Identify module boundaries (`agents`, `jobs`, `notifications`, `secrets` entrypoints and exports)
- [x] 2.2 Move `crates/bastion-http/src/http/{agents,jobs,notifications,secrets}.rs` to the corresponding `*/mod.rs`
- [x] 2.3 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
