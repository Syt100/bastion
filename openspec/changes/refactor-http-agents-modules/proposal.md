# Change: Refactor HTTP agents module structure

## Why
`crates/bastion-http/src/http/agents.rs` has grown into a large module mixing unrelated concerns (admin CRUD, enrollment, run ingestion, websocket protocol handling, and snapshot push logic). Splitting it into focused submodules reduces cognitive load and makes future changes safer and easier to review.

## What Changes
- Split HTTP agents implementation into focused submodules under `crates/bastion-http/src/http/agents/`
- Keep existing routing and `pub(super)` API stable (`agents::*` used by the HTTP router and sibling modules)
- Preserve behavior; changes are structural/refactor-focused

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-http/src/http/agents.rs`, `crates/bastion-http/src/http/agents/*.rs`

## Compatibility / Non-Goals
- No API route changes and no JSON schema changes for existing clients.
- No behavioral changes intended; refactor only.

