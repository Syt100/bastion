# Change: Refactor agent offline module entrypoint structure

## Why
`crates/bastion/src/agent_client/offline.rs` currently serves as the module entrypoint while its submodules already live under `crates/bastion/src/agent_client/offline/`. Converting the entrypoint into `offline/mod.rs` aligns the layout with the existing directory structure and makes navigation more consistent.

## What Changes
- Move `agent_client/offline.rs` to `agent_client/offline/mod.rs`
- Keep submodules (`cron`, `scheduler`, `storage`, `sync`) as-is under `agent_client/offline/`
- Preserve the existing internal API (`agent_client::offline::*`) and behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion/src/agent_client/offline.rs`, `crates/bastion/src/agent_client/offline/mod.rs`

## Compatibility / Non-Goals
- No behavior changes intended for offline scheduling or sync logic.

