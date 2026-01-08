## 1. Spec
- [x] 1.1 Add `backend` spec delta for: restore entrypoint modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-backup-restore-entrypoint-modules --strict`

## 2. Backend - Restore entrypoint modularization
- [x] 2.1 Convert `crates/bastion-backup/src/restore.rs` into `crates/bastion-backup/src/restore/mod.rs`
- [x] 2.2 Move restore unit tests into `crates/bastion-backup/src/restore/tests.rs`
- [x] 2.3 Keep public API stable (`bastion_backup::restore::*`) and preserve behavior
- [x] 2.4 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
