## 1. Spec
- [x] 1.1 Add `backend` spec delta for: notifications entrypoint relocation (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-engine-notifications-entrypoint-modules --strict`

## 2. Engine - Notifications entrypoint relocation
- [ ] 2.1 Identify module boundaries (`enqueue`, `loop`, `send`, `template`, entrypoint exports)
- [ ] 2.2 Move `crates/bastion-engine/src/notifications.rs` to `crates/bastion-engine/src/notifications/mod.rs`
- [ ] 2.3 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
