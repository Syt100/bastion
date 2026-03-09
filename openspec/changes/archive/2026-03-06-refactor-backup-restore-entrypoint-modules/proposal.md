# Change: Refactor backup restore entrypoint module structure

## Why
`crates/bastion-backup/src/restore.rs` currently contains both the restore public entrypoint/types and a large inline test module. Moving the entrypoint to `restore/mod.rs` and extracting tests to `restore/tests.rs` keeps the entrypoint focused and improves navigability.

## What Changes
- Convert `crates/bastion-backup/src/restore.rs` into `crates/bastion-backup/src/restore/mod.rs`
- Move restore unit tests into `crates/bastion-backup/src/restore/tests.rs`
- Keep the existing public API stable (`bastion_backup::restore::*`) and preserve behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-backup/src/restore.rs`, `crates/bastion-backup/src/restore/mod.rs`, `crates/bastion-backup/src/restore/tests.rs`

## Compatibility / Non-Goals
- No changes intended to restore semantics, selection rules, conflict policy behavior, or operation orchestration beyond structural movement.

