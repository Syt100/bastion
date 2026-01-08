# Change: Refactor agent websocket connect module structure

## Why
`crates/bastion/src/agent_client/connect.rs` currently mixes websocket connection setup, hello handshake, heartbeat/ping-pong logic, and per-message handling (config/secrets snapshots, tasks, and fs_list). Splitting it into focused submodules improves readability and long-term maintainability.

## What Changes
- Convert `connect` into a folder module under `crates/bastion/src/agent_client/connect/`
- Split implementation into focused submodules (handshake, message handling, heartbeat/util helpers)
- Keep the existing public surface stable (`agent_client::connect::connect_and_run`) and preserve behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion/src/agent_client/connect.rs`, `crates/bastion/src/agent_client/connect/*.rs`

## Compatibility / Non-Goals
- No behavior changes intended (protocol handling, retries/reconnect logic, snapshot persistence, or task execution semantics).

