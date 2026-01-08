# Change: Refactor HTTP jobs module structure

## Why
`crates/bastion-http/src/http/jobs.rs` is a large module mixing unrelated concerns (CRUD handlers, spec validation, run triggering/listing, and websocket streaming for run events). Splitting it into focused submodules reduces cognitive load and makes future changes safer and easier to review.

## What Changes
- Split HTTP jobs implementation into focused submodules under `crates/bastion-http/src/http/jobs/`
- Keep existing routing and `pub(super)` API stable (`jobs::*` used by the HTTP router and sibling modules)
- Preserve behavior; changes are structural/refactor-focused

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-http/src/http/jobs.rs`, `crates/bastion-http/src/http/jobs/*.rs`

## Compatibility / Non-Goals
- No API route changes and no JSON schema changes for existing clients.
- No behavioral changes intended; refactor only.

