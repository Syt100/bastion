## 1. Spec
- [x] 1.1 Add `backend` spec delta for: storage notifications repo modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-storage-notifications-repo-modules --strict`

## 2. Backend - Storage notifications repo modularization
- [x] 2.1 Identify responsibilities and module boundaries (`enqueue`, `claim`, `transitions`, `queries`)
- [x] 2.2 Convert `notifications_repo.rs` into a folder module and keep public re-exports stable
- [x] 2.3 Extract enqueue functions into `crates/bastion-storage/src/notifications_repo/enqueue.rs`
- [x] 2.4 Extract claim/scheduling functions into `crates/bastion-storage/src/notifications_repo/claim.rs`
- [x] 2.5 Extract status transition helpers into `crates/bastion-storage/src/notifications_repo/transitions.rs`
- [x] 2.6 Extract queue/query helpers into `crates/bastion-storage/src/notifications_repo/queries.rs`
- [x] 2.7 Keep tests passing (adjust module paths only; no behavior changes)
- [x] 2.8 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
