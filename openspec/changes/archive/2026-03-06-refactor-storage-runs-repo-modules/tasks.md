## 1. Spec
- [x] 1.1 Add `backend` spec delta for: storage runs repo modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-storage-runs-repo-modules --strict`

## 2. Backend - Storage runs repo modularization
- [x] 2.1 Identify responsibilities and module boundaries (`types`, `runs`, `events`, `maintenance`)
- [x] 2.2 Convert `runs_repo.rs` into a folder module and keep public re-exports stable
- [x] 2.3 Extract shared types into `crates/bastion-storage/src/runs_repo/types.rs`
- [x] 2.4 Extract run lifecycle helpers into `crates/bastion-storage/src/runs_repo/runs.rs`
- [x] 2.5 Extract run event helpers into `crates/bastion-storage/src/runs_repo/events.rs`
- [x] 2.6 Extract retention/cleanup helpers into `crates/bastion-storage/src/runs_repo/maintenance.rs`
- [x] 2.7 Keep tests passing (adjust module paths only; no behavior changes)
- [x] 2.8 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
