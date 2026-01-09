## 1. Spec
- [x] 1.1 Add `backend` spec delta for: restore entries index module split (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-backup-restore-entries-index-modules --strict`

## 2. Backup - Restore entries index module split
- [ ] 2.1 Identify responsibilities and module boundaries (types, fetch/cache, listing/filtering)
- [ ] 2.2 Convert `restore/entries_index.rs` into folder module and keep exports stable
- [ ] 2.3 Extract listing logic into `restore/entries_index/list.rs`
- [ ] 2.4 Extract index fetch/cache logic into `restore/entries_index/fetch.rs`
- [ ] 2.5 Extract shared types into `restore/entries_index/types.rs`
- [ ] 2.6 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
