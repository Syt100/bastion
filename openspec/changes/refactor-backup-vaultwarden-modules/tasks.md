## 1. Spec
- [x] 1.1 Add `backend` spec delta for: vaultwarden backup modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-backup-vaultwarden-modules --strict`

## 2. Backend - Vaultwarden backup modularization
- [x] 2.1 Identify responsibilities and module boundaries (`builder`, `tar`, `io`, `hash`)
- [x] 2.2 Convert `vaultwarden.rs` into a folder module and keep `build_vaultwarden_run` stable
- [x] 2.3 Extract tar/entries writing into `crates/bastion-backup/src/backup/vaultwarden/tar.rs`
- [x] 2.4 Extract hashing + json helpers into `crates/bastion-backup/src/backup/vaultwarden/io.rs` and/or `hash.rs`
- [x] 2.5 Keep tests passing (adjust module paths only; no behavior changes)
- [x] 2.6 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
