# Change: Refactor backup restore module structure

## Why
`crates/bastion-backup/src/restore.rs` contains multiple responsibilities (access resolution, entries index listing, unpacking, verification, operation orchestration). As it grows, it becomes harder to navigate and to make safe changes without unintended coupling.

## What Changes
- Split restore implementation into focused submodules under `crates/bastion-backup/src/restore/`
- Keep the existing public API stable for current callers
- Reduce duplicated “resolve run → validate → open target access” logic by centralizing it

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-backup/src/restore.rs`, `crates/bastion-backup/src/restore/*.rs`

## Compatibility / Non-Goals
- No API route changes and no JSON schema changes for existing clients.
- No behavior changes intended; this change is structural/refactor-focused.

