# Change: Refactor HTTP submodule entrypoint structure

## Why
The HTTP layer uses file entrypoints like `crates/bastion-http/src/http/agents.rs` while their submodules already live in matching directories (e.g. `crates/bastion-http/src/http/agents/`). Converting these entrypoints into directory modules (e.g. `agents/mod.rs`) makes navigation more consistent and keeps related code co-located.

## What Changes
- Move HTTP submodule entrypoints to directory modules:
  - `http/agents.rs` → `http/agents/mod.rs`
  - `http/jobs.rs` → `http/jobs/mod.rs`
  - `http/notifications.rs` → `http/notifications/mod.rs`
  - `http/secrets.rs` → `http/secrets/mod.rs`
- Keep existing submodules as-is under their directories
- Preserve the existing internal API (`http::{agents,jobs,notifications,secrets}::*`) and behavior

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-http/src/http/{agents,jobs,notifications,secrets}.rs` and their new `mod.rs` locations

## Compatibility / Non-Goals
- No behavior changes intended for HTTP routing, request validation, or handlers.

