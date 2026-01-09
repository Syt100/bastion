# Change: Refactor agent offline sync module structure

## Why
`crates/bastion/src/agent_client/offline/sync.rs` mixes directory scanning, offline run/event file loading, HTTP ingest request building, and tests. Splitting it into focused submodules improves readability and long-term maintainability.

## What Changes
- Convert `offline::sync` into a folder module under `crates/bastion/src/agent_client/offline/sync/`
- Split implementation into focused submodules (request types, events loader, HTTP ingest helper)
- Keep the existing internal entrypoint stable (`offline::sync::sync_offline_runs`) and preserve behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion/src/agent_client/offline/sync.rs`, `crates/bastion/src/agent_client/offline/sync/*.rs`

## Compatibility / Non-Goals
- No behavior changes intended (which runs are uploaded, request shape, auth headers, and cleanup semantics).

