## 1. Spec
- [x] 1.1 Add `backend` spec delta for: WebDAV client helper extraction (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-targets-webdav-client-helpers --strict`

## 2. Targets - WebDAV client helper extraction
- [x] 2.1 Identify duplicated request setup patterns in WebDAV client methods
- [x] 2.2 Extract shared authenticated request builder helper and reuse across methods
- [x] 2.3 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
