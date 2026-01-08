# Change: Refactor agent offline storage module structure

## Why
`crates/bastion/src/agent_client/offline/storage.rs` mixes offline run paths, on-disk file formats, writer state machine, and IO helpers. Splitting it into focused submodules improves readability and makes future changes (e.g., format updates) easier to reason about.

## What Changes
- Convert `offline::storage` into a folder module under `crates/bastion/src/agent_client/offline/storage/`
- Split implementation into focused submodules (paths, types, writer, IO helpers)
- Keep the existing public surface stable (`offline::storage::*` within the agent client) and preserve behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion/src/agent_client/offline/storage.rs`, `crates/bastion/src/agent_client/offline/storage/*.rs`

## Compatibility / Non-Goals
- No changes intended to on-disk formats or offline run semantics beyond structural movement.

