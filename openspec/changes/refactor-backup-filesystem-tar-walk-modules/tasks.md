## 1. Spec
- [x] 1.1 Add `backend` spec delta for: filesystem tar walk modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-backup-filesystem-tar-walk-modules --strict`

## 2. Backend - Filesystem tar walk modularization
- [x] 2.1 Identify responsibilities and module boundaries (`selection`, `source_entry`, `legacy_root`)
- [x] 2.2 Convert `walk.rs` into a folder module and keep the `walk::write_tar_entries` entrypoint stable
- [x] 2.3 Extract selected-source walking into `crates/bastion-backup/src/backup/filesystem/tar/walk/source_entry.rs`
- [x] 2.4 Extract legacy-root walking into `crates/bastion-backup/src/backup/filesystem/tar/walk/legacy_root.rs`
- [x] 2.5 Keep `tar.rs` integration unchanged (module path stable)
- [x] 2.6 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
