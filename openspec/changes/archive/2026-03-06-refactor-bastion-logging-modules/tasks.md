## 1. Spec
- [x] 1.1 Add `backend` spec delta for: logging module modularization (no behavior changes)
- [x] 1.2 Run `openspec validate refactor-bastion-logging-modules --strict`

## 2. Bastion - Logging module modularization
- [x] 2.1 Identify responsibilities and module boundaries (filter/init, file config, pruning, suffix parsing, tests)
- [x] 2.2 Convert `logging.rs` into a folder module and keep public entrypoints stable
- [x] 2.3 Extract log file configuration into `crates/bastion/src/logging/file_config.rs`
- [x] 2.4 Extract log pruning helpers into `crates/bastion/src/logging/prune.rs`
- [x] 2.5 Extract rotation suffix parsing into `crates/bastion/src/logging/suffix.rs`
- [x] 2.6 Move unit tests into `crates/bastion/src/logging/tests.rs`
- [x] 2.7 Run `cargo fmt`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`
