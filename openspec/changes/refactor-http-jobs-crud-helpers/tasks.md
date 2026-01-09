## 1. Spec
- [x] 1.1 Add `backend` spec delta for: jobs CRUD helper extraction (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-http-jobs-crud-helpers --strict`

## 2. HTTP - Jobs CRUD helper extraction
- [ ] 2.1 Identify duplicated validation/normalization logic in jobs CRUD handlers
- [ ] 2.2 Extract shared helpers (name, schedule, agent validation, cron validation, snapshot notify)
- [ ] 2.3 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
