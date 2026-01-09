## 1. Spec
- [x] 1.1 Add `backend` spec delta for: restore operations module split (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-backup-restore-operations-modules --strict`

## 2. Backup - Restore operations module split
- [ ] 2.1 Identify responsibilities and module boundaries (spawn wrappers, restore flow, verify flow, shared helpers)
- [ ] 2.2 Convert `restore/operations.rs` into folder module and keep exports stable
- [ ] 2.3 Extract shared helpers into `restore/operations/util.rs`
- [ ] 2.4 Extract restore operation implementation into `restore/operations/restore.rs`
- [ ] 2.5 Extract verify operation implementation into `restore/operations/verify.rs`
- [ ] 2.6 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
