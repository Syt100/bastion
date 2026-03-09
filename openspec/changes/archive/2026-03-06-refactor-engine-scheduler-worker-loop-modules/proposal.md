# Change: Refactor scheduler worker loop module structure

## Why
`crates/bastion-engine/src/scheduler/worker/mod.rs` currently mixes worker loop orchestration, argument types, and submodule wiring. Extracting the worker loop into a focused submodule improves maintainability and keeps the entrypoint file small and easy to navigate.

## What Changes
- Extract the worker loop implementation into a dedicated submodule
- Keep the worker module entrypoints stable (`scheduler::worker::{WorkerLoopArgs, run_worker_loop}`)
- Preserve existing behavior for run claiming, dispatching, execution, and notifications

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-engine/src/scheduler/worker/mod.rs` and new `crates/bastion-engine/src/scheduler/worker/loop.rs`

## Compatibility / Non-Goals
- No behavior changes intended for scheduling, retries, timeouts, or run completion/notification flows.

