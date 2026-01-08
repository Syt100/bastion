# Change: Refactor agent task handling module structure

## Why
`crates/bastion/src/agent_client/tasks.rs` mixes multiple responsibilities: task orchestration, per-job backup logic (filesystem/sqlite/vaultwarden), event emission, and task result persistence. Splitting it into focused submodules improves readability and makes future changes safer.

## What Changes
- Convert `tasks` into a folder module under `crates/bastion/src/agent_client/tasks/`
- Extract per-spec backup implementations into focused submodules (filesystem/sqlite/vaultwarden)
- Keep the existing public surface stable (`agent_client::tasks::handle_backup_task`) and preserve behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion/src/agent_client/tasks.rs`, `crates/bastion/src/agent_client/tasks/*.rs`

## Compatibility / Non-Goals
- No behavior changes intended (events, encryption mapping, artifact upload semantics, or error handling).

