## 1. Spec
- [x] 1.1 Add `backend` spec delta for: webdav secrets handler helper extraction (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-http-secrets-webdav-helpers --strict`

## 2. HTTP - WebDAV secrets handler helper extraction
- [ ] 2.1 Identify duplicated logic across hub/node WebDAV secret handlers
- [ ] 2.2 Extract shared helpers (validation, list/get/delete/upsert wiring, snapshot notify)
- [ ] 2.3 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
