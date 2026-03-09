# Change: Refactor WebDAV secrets HTTP handlers to use shared helpers

## Why
`crates/bastion-http/src/http/secrets/webdav.rs` currently duplicates logic between hub-level and node-level secret handlers (list/upsert/get/delete). Consolidating shared validation and persistence logic into focused helpers reduces duplication and makes future changes safer.

## What Changes
- Extract shared validation and secrets_repo wiring into private helper functions
- Keep HTTP routes, response schemas, and error codes/messages stable
- Preserve current agent config snapshot side effects for node-scoped secrets

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-http/src/http/secrets/webdav.rs`

## Compatibility / Non-Goals
- No behavior changes intended for WebDAV secret CRUD semantics, error handling, or snapshot notification behavior.

