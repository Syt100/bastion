## 1. Spec
- [x] 1.1 Add `backend` spec delta for: filesystem backup modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-backup-filesystem-modules --strict`

## 2. Backend - Filesystem backup modularization
- [x] 2.1 Identify filesystem responsibilities and module boundaries
- [x] 2.2 Extract entries index types/helpers into `crates/bastion-backup/src/backup/filesystem/entries_index.rs`
- [x] 2.3 Extract tar writing/orchestration into `crates/bastion-backup/src/backup/filesystem/tar.rs`
- [x] 2.4 Extract shared helpers (paths/globs/hash/json) into `crates/bastion-backup/src/backup/filesystem/util.rs`
- [x] 2.5 Keep public API stable; update internal imports/tests as needed
- [x] 2.6 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
