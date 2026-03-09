## 1. Spec
- [x] 1.1 Add `backend` spec delta for: jobs repo module modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-storage-jobs-repo-modules --strict`

## 2. Storage - Jobs repo modularization
- [x] 2.1 Identify responsibilities and module boundaries (types, repo, tests)
- [x] 2.2 Convert `jobs_repo.rs` into a folder module and keep public entrypoints stable
- [x] 2.3 Extract job/overlap types into `crates/bastion-storage/src/jobs_repo/types.rs`
- [x] 2.4 Extract repository functions into `crates/bastion-storage/src/jobs_repo/repo.rs`
- [x] 2.5 Move unit tests into `crates/bastion-storage/src/jobs_repo/tests.rs`
- [x] 2.6 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
