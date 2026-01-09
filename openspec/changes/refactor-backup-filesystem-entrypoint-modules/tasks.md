## 1. Spec
- [x] 1.1 Add `backend` spec delta for: backup filesystem entrypoint relocation (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-backup-filesystem-entrypoint-modules --strict`

## 2. Backup - Filesystem entrypoint relocation
- [x] 2.1 Identify module boundaries (entrypoint, tar/util/entries_index, tests)
- [x] 2.2 Move `crates/bastion-backup/src/backup/filesystem.rs` to `crates/bastion-backup/src/backup/filesystem/mod.rs`
- [x] 2.3 Move unit tests into `crates/bastion-backup/src/backup/filesystem/tests.rs`
- [x] 2.4 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
