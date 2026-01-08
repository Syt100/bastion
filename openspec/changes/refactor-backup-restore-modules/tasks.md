## 1. Spec
- [x] 1.1 Add `backend` spec delta for: restore module modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-backup-restore-modules --strict`

## 2. Backend - Restore modularization
- [x] 2.1 Extract target/run access resolution into `crates/bastion-backup/src/restore/access.rs`
- [x] 2.2 Extract operation orchestration into `crates/bastion-backup/src/restore/operations.rs`
- [x] 2.3 Extract artifact parts fetch/verify helpers into `crates/bastion-backup/src/restore/parts.rs`
- [x] 2.4 Extract tar unpack + path safety helpers into `crates/bastion-backup/src/restore/unpack.rs`
- [x] 2.5 Extract restore verification (entries + sqlite) into `crates/bastion-backup/src/restore/verify.rs`
- [x] 2.6 Keep public API stable; update internal imports/tests as needed
- [x] 2.7 Run `cargo clippy --workspace --all-targets` and `cargo test --workspace`
