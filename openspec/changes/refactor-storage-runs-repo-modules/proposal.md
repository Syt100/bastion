# Change: Refactor storage runs repository module structure

## Why
`crates/bastion-storage/src/runs_repo.rs` is a large module mixing concerns (run CRUD/status transitions, run event append/query logic, scheduler claiming helpers, retention/cleanup helpers, and tests). Splitting it into focused submodules improves readability and long-term maintainability.

## What Changes
- Convert `runs_repo` into a folder module under `crates/bastion-storage/src/runs_repo/`
- Split implementation into focused submodules (`types`, `runs`, `events`, `maintenance`)
- Keep the existing public surface stable (`bastion_storage::runs_repo::*`) and preserve behavior (no SQL/semantic changes)

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-storage/src/runs_repo.rs`, `crates/bastion-storage/src/runs_repo/*.rs`

## Compatibility / Non-Goals
- No changes intended to run lifecycle semantics, event ordering/sequence allocation, retention rules, or SQL schema/queries beyond structural movement.

