# Change: Refactor engine scheduler module structure

## Why
`crates/bastion-engine/src/scheduler.rs` is a large module combining multiple responsibilities (cron parsing/normalization, scheduling policy, DB queueing, notifications, and orchestration). This makes it harder to navigate, review, and safely change individual parts.

## What Changes
- Split scheduler implementation into focused submodules under `crates/bastion-engine/src/scheduler/`
- Keep the existing scheduler public API stable for current callers
- Preserve behavior; changes are structural/refactor-focused

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-engine/src/scheduler.rs`, `crates/bastion-engine/src/scheduler/*.rs`

## Compatibility / Non-Goals
- No API route changes and no JSON schema changes for existing clients.
- No behavior changes intended; refactor only.

