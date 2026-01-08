# Change: Refactor agent client module structure

## Why
`crates/bastion/src/agent_client/mod.rs` is a large module that mixes concerns (identity/enrollment, websocket connection loop, task handling, target storage, and filesystem listing). Splitting it into focused submodules improves readability and long-term maintainability.

## What Changes
- Split agent client implementation into focused submodules under `crates/bastion/src/agent_client/`
- Keep the existing public surface stable (`agent_client::run` and behavior)
- Preserve behavior; changes are structural/refactor-focused

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion/src/agent_client/mod.rs`, `crates/bastion/src/agent_client/*.rs`

## Compatibility / Non-Goals
- No changes intended to protocol behavior, task semantics, or target storage behavior.
- No changes intended to offline scheduling behavior other than updated internal imports.

