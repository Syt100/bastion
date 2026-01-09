# Change: Refactor scheduler worker execute module structure

## Why
`crates/bastion-engine/src/scheduler/worker/execute.rs` contains the execution logic for multiple job types (filesystem/sqlite/vaultwarden) in a single file. Splitting the per-job-type execution paths into focused submodules improves readability and makes it easier to evolve each execution path independently.

## What Changes
- Convert `worker/execute.rs` into a folder module (`worker/execute/mod.rs`)
- Split per-job-type execution logic into focused submodules
- Preserve behavior and the existing internal API (`worker::execute::{ExecuteRunArgs, execute_run}`)

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-engine/src/scheduler/worker/execute.rs` and new `crates/bastion-engine/src/scheduler/worker/execute/` submodules

## Compatibility / Non-Goals
- No behavior changes intended for backup packaging, upload semantics, summaries, or failure handling.

