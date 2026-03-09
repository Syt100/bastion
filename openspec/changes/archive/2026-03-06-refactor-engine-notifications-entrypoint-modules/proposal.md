# Change: Refactor notifications module entrypoint structure

## Why
`crates/bastion-engine/src/notifications.rs` currently serves as the module entrypoint while its submodules already live under `crates/bastion-engine/src/notifications/`. Converting the entrypoint into `notifications/mod.rs` aligns the layout with the existing directory structure and makes navigation more consistent.

## What Changes
- Move `notifications.rs` to `notifications/mod.rs`
- Keep submodules (`enqueue`, `loop`, `send`, `template`) as-is under `notifications/`
- Preserve the existing public API (`notifications::*`) and behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-engine/src/notifications.rs`, `crates/bastion-engine/src/notifications/mod.rs`

## Compatibility / Non-Goals
- No behavior changes intended for notification enqueueing, formatting, or sending.

