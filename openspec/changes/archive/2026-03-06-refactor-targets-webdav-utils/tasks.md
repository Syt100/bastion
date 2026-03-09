## 1. Spec
- [x] 1.1 Add `backend` spec delta for: WebDAV helper deduplication (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-targets-webdav-utils --strict`

## 2. Targets - WebDAV helper deduplication
- [x] 2.1 Identify shared helpers between `webdav` and `webdav_client`
- [x] 2.2 Centralize URL redaction helper and reuse it from both modules
- [x] 2.3 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
