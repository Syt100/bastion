## 1. Spec
- [x] 1.1 Add `backend` spec delta for: job_spec module split (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-core-job-spec-modules --strict`

## 2. Core - job_spec module split
- [ ] 2.1 Identify module boundaries (types vs validation/parsing)
- [ ] 2.2 Convert `crates/bastion-core/src/job_spec.rs` into folder module and keep exports stable
- [ ] 2.3 Extract type definitions into `crates/bastion-core/src/job_spec/types.rs`
- [ ] 2.4 Extract parsing/validation into `crates/bastion-core/src/job_spec/validation.rs`
- [ ] 2.5 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
