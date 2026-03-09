# Change: Refactor storage operations_repo module structure

## Why
`crates/bastion-storage/src/operations_repo.rs` currently mixes operation types (kind/status/events) and repository persistence logic in a single file. Splitting it into focused submodules improves readability and long-term maintainability.

## What Changes
- Convert `operations_repo` into a folder module under `crates/bastion-storage/src/operations_repo/`
- Split implementation into focused submodules (`types`, `repo`, `tests`)
- Keep the existing public surface stable (`bastion_storage::operations_repo::*`) and preserve behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-storage/src/operations_repo.rs`, `crates/bastion-storage/src/operations_repo/*.rs`

## Compatibility / Non-Goals
- No changes intended to database schema, operation semantics, or event ordering beyond structural movement.

