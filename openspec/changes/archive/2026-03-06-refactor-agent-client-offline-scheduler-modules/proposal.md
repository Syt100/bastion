# Change: Refactor agent offline scheduler module structure

## Why
`crates/bastion/src/agent_client/offline/scheduler.rs` is a large module mixing concerns (cron schedule loop, worker execution loop, sink message handling, and run persistence helpers). Splitting it into focused submodules improves readability and long-term maintainability.

## What Changes
- Convert `offline::scheduler` into a folder module under `crates/bastion/src/agent_client/offline/scheduler/`
- Split implementation into focused submodules (`cron_loop`, `worker_loop`, `sink`, `types`)
- Keep the existing public surface stable (`offline::scheduler::offline_scheduler_loop`) and preserve behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion/src/agent_client/offline/scheduler.rs`, `crates/bastion/src/agent_client/offline/scheduler/*.rs`

## Compatibility / Non-Goals
- No changes intended to offline scheduling semantics, overlap-policy enforcement, event persistence, or error handling behavior beyond structural movement.

