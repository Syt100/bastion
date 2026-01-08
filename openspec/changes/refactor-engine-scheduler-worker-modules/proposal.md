# Change: Refactor scheduler worker module structure

## Why
`crates/bastion-engine/src/scheduler/worker.rs` is a large module mixing concerns (worker loop orchestration, dispatch-to-agent logic, local run execution, and target artifact storage). Splitting it into focused submodules improves readability and long-term maintainability.

## What Changes
- Split scheduler worker implementation into focused submodules under `crates/bastion-engine/src/scheduler/worker/`
- Keep existing public surface stable (`scheduler::worker::run_worker_loop` and behavior)
- Preserve behavior; changes are structural/refactor-focused

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-engine/src/scheduler/worker.rs`, `crates/bastion-engine/src/scheduler/worker/*.rs`

## Compatibility / Non-Goals
- No changes intended to run scheduling semantics, dispatch behavior, run event emission, or artifact upload behavior.

