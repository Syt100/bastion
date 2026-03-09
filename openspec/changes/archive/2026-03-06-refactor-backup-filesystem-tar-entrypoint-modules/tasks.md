## 1. Spec
- [x] 1.1 Add `backend` spec delta for: filesystem tar entrypoint relocation (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-backup-filesystem-tar-entrypoint-modules --strict`

## 2. Backup - Filesystem tar entrypoint relocation
- [x] 2.1 Identify module boundaries (tar entrypoint, walk/entry submodules)
- [x] 2.2 Move `crates/bastion-backup/src/backup/filesystem/tar.rs` to `crates/bastion-backup/src/backup/filesystem/tar/mod.rs`
- [x] 2.3 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
