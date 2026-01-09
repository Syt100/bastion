## 1. Spec
- [x] 1.1 Add `backend` spec delta for: storage auth module modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-storage-auth-modules --strict`

## 2. Storage - Auth module modularization
- [x] 2.1 Identify responsibilities and module boundaries (users, sessions, password, throttle)
- [x] 2.2 Convert `auth.rs` into a folder module and keep public entrypoints stable
- [x] 2.3 Extract user operations into `crates/bastion-storage/src/auth/users.rs`
- [x] 2.4 Extract session operations into `crates/bastion-storage/src/auth/sessions.rs`
- [x] 2.5 Extract password helpers into `crates/bastion-storage/src/auth/password.rs`
- [x] 2.6 Extract login throttle logic into `crates/bastion-storage/src/auth/throttle.rs`
- [x] 2.7 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
