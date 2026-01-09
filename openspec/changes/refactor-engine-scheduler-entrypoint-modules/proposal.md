# Change: Refactor scheduler module entrypoint structure

## Why
`crates/bastion-engine/src/scheduler.rs` currently serves as the module entrypoint while its submodules already live under `crates/bastion-engine/src/scheduler/`. Converting the entrypoint into `scheduler/mod.rs` aligns the layout with the existing directory structure and makes navigation more consistent.

## What Changes
- Move `scheduler.rs` to `scheduler/mod.rs`
- Keep submodules (`cron`, `incomplete_cleanup`, `queue`, `retention`, `worker`) as-is under `scheduler/`
- Preserve the existing internal API (`scheduler::*`) and behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-engine/src/scheduler.rs`, `crates/bastion-engine/src/scheduler/mod.rs`

## Compatibility / Non-Goals
- No behavior changes intended for scheduling, dispatching, or retention flows.

