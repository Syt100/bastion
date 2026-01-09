# Change: Refactor storage auth module structure

## Why
`crates/bastion-storage/src/auth.rs` currently mixes user management, session management, password hashing, and login throttling in a single file. Splitting it into focused submodules improves readability and long-term maintainability.

## What Changes
- Convert `auth` into a folder module under `crates/bastion-storage/src/auth/`
- Split implementation into focused submodules (`users`, `sessions`, `password`, `throttle`)
- Keep the existing public surface stable (`bastion_storage::auth::*`) and preserve behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-storage/src/auth.rs`, `crates/bastion-storage/src/auth/*.rs`

## Compatibility / Non-Goals
- No changes intended to password hashing, session semantics, or login throttle behavior beyond structural movement.

