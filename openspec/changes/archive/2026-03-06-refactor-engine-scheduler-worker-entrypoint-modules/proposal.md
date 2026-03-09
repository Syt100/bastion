# Change: Refactor scheduler worker module entrypoint structure

## Why
`crates/bastion-engine/src/scheduler/worker.rs` currently serves as the module entrypoint while its submodules already live under `crates/bastion-engine/src/scheduler/worker/`. Converting the entrypoint into `worker/mod.rs` aligns the layout with the existing directory structure and makes navigation more consistent.

## What Changes
- Move `scheduler/worker.rs` to `scheduler/worker/mod.rs`
- Keep submodules (`dispatch`, `execute`, `target_store`) as-is under `scheduler/worker/`
- Preserve the existing internal API (`scheduler::worker::*`) and behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-engine/src/scheduler/worker.rs`, `crates/bastion-engine/src/scheduler/worker/mod.rs`

## Compatibility / Non-Goals
- No behavior changes intended for scheduling, dispatching, or run completion.

