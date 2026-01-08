## 1. Spec
- [x] 1.1 Add `backend` spec delta for: engine notifications modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-engine-notifications-modules --strict`

## 2. Backend - Engine notifications modularization
- [x] 2.1 Identify responsibilities and module boundaries (`enqueue`, `loop`, `send`, `template`)
- [x] 2.2 Extract enqueue selection logic into `crates/bastion-engine/src/notifications/enqueue.rs`
- [x] 2.3 Extract worker loop + retry/backoff into `crates/bastion-engine/src/notifications/loop.rs`
- [x] 2.4 Extract channel send logic into `crates/bastion-engine/src/notifications/send.rs`
- [x] 2.5 Extract template context + rendering into `crates/bastion-engine/src/notifications/template.rs`
- [x] 2.6 Keep `notifications` public surface stable in `notifications.rs`; update internal imports as needed
- [x] 2.7 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
