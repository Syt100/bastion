# Change: Refactor agent managed state module structure

## Why
`crates/bastion/src/agent_client/managed.rs` mixes multiple responsibilities (managed secrets snapshot encryption, managed config snapshot encryption, task result caching/persistence, shared path helpers, and tests). Splitting it into focused submodules improves readability and long-term maintainability.

## What Changes
- Convert `managed` into a folder module under `crates/bastion/src/agent_client/managed/`
- Split implementation into focused submodules (`paths`, `secrets_snapshot`, `config_snapshot`, `task_results`, `io`)
- Keep the existing public surface stable (`agent_client::managed::*`) and preserve behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion/src/agent_client/managed.rs`, `crates/bastion/src/agent_client/managed/*.rs`

## Compatibility / Non-Goals
- No changes intended to on-disk formats, encryption behavior, or task result semantics beyond structural movement.

