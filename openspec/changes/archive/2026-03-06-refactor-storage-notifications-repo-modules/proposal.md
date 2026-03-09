# Change: Refactor storage notifications repository module structure

## Why
`crates/bastion-storage/src/notifications_repo.rs` is a large module mixing concerns (enqueueing per channel, claiming due work, status transitions, queue queries, and tests). Splitting it into focused submodules improves readability and long-term maintainability.

## What Changes
- Convert `notifications_repo` into a folder module under `crates/bastion-storage/src/notifications_repo/`
- Split implementation into focused submodules (`enqueue`, `claim`, `transitions`, `queries`)
- Keep the existing public surface stable (`bastion_storage::notifications_repo::*`) and preserve behavior (no SQL/semantic changes)

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-storage/src/notifications_repo.rs`, `crates/bastion-storage/src/notifications_repo/*.rs`

## Compatibility / Non-Goals
- No changes intended to notification queue semantics, retry logic, or SQL schema/queries beyond structural movement.

