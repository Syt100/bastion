# Change: Refactor storage jobs_repo module structure

## Why
`crates/bastion-storage/src/jobs_repo.rs` mixes job domain types and repository persistence logic in a single file. Splitting it into focused submodules improves readability and long-term maintainability.

## What Changes
- Convert `jobs_repo` into a folder module under `crates/bastion-storage/src/jobs_repo/`
- Split implementation into focused submodules (`types`, `repo`, `tests`)
- Keep the existing public surface stable (`bastion_storage::jobs_repo::*`) and preserve behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-storage/src/jobs_repo.rs`, `crates/bastion-storage/src/jobs_repo/*.rs`

## Compatibility / Non-Goals
- No changes intended to database schema, job CRUD semantics, or scheduling fields beyond structural movement.

