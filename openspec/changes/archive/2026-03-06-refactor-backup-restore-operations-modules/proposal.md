# Change: Refactor restore operations module structure

## Why
`crates/bastion-backup/src/restore/operations.rs` currently contains multiple concerns (spawn wrappers, restore operation implementation, verify operation implementation, and shared helpers). Splitting it into focused submodules improves maintainability and makes the restore/verify flows easier to navigate.

## What Changes
- Convert `restore/operations.rs` into a folder module (`restore/operations/mod.rs`)
- Split shared helpers, restore flow, and verify flow into focused submodules
- Preserve existing public API (`restore::spawn_restore_operation`, `restore::spawn_verify_operation`) and behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-backup/src/restore/operations.rs` and new `crates/bastion-backup/src/restore/operations/` submodules

## Compatibility / Non-Goals
- No behavior changes intended for restore/verify execution, events, summaries, or cleanup semantics.

