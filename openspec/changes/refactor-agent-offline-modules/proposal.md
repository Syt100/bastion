# Change: Refactor agent offline module structure

## Why
`crates/bastion/src/agent_client/offline.rs` is a large module that mixes multiple concerns (cron normalization/matching, offline run persistence, offline execution, and offline run ingestion sync). Splitting it into focused submodules improves readability and makes future changes safer.

## What Changes
- Split agent offline implementation into focused submodules under `crates/bastion/src/agent_client/offline/`
- Keep the existing internal API stable (`offline_scheduler_loop`, `sync_offline_runs`)
- Preserve behavior; changes are structural/refactor-focused

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion/src/agent_client/offline.rs`, `crates/bastion/src/agent_client/offline/*.rs`

## Compatibility / Non-Goals
- No changes intended to on-disk offline run formats, HTTP request formats, or runtime behavior.
- No feature additions; refactor only.

