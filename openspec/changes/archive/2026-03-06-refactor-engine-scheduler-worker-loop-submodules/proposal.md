# Change: Refactor scheduler worker loop into focused submodules

## Why
`crates/bastion-engine/src/scheduler/worker/loop.rs` currently mixes run claiming/backoff, job loading/spec validation, agent dispatch + polling, and local execution completion logic in one function. Splitting it into focused submodules makes the run lifecycle easier to follow and reduces the risk of regressions when adjusting any one phase.

## What Changes
- Convert `scheduler/worker/loop.rs` into a folder module (`scheduler/worker/loop/mod.rs`)
- Extract the worker loop phases into focused submodules (claim/process/agent/local/notifications helpers)
- Preserve existing behavior, logs, and error codes/messages for runs and events

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-engine/src/scheduler/worker/loop.rs` and new `crates/bastion-engine/src/scheduler/worker/loop/` submodules

## Compatibility / Non-Goals
- No behavior changes intended for run claiming cadence, dispatch retries, agent timeout, local execution semantics, or notification enqueue behavior.

