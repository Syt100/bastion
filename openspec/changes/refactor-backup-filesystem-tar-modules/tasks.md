## 1. Spec
- [x] 1.1 Add `backend` spec delta for: filesystem tar modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-backup-filesystem-tar-modules --strict`

## 2. Backend - Filesystem tar modularization
- [x] 2.1 Identify responsibilities and module boundaries (`parts`, `walk`, `entries`, `hardlinks/meta`)
- [x] 2.2 Extract source walking + selection logic into `crates/bastion-backup/src/backup/filesystem/tar/walk.rs`
- [x] 2.3 Extract entry writing + hardlink helpers into `crates/bastion-backup/src/backup/filesystem/tar/entry.rs`
- [x] 2.4 Keep `write_tar_zstd_parts` stable in `tar.rs`; update internal imports as needed
- [x] 2.5 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
