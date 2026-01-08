# Change: Refactor HTTP notifications module structure

## Why
`crates/bastion-http/src/http/notifications.rs` is a large module mixing unrelated concerns (settings CRUD, destination management, destination testing, and notification queue operations). Splitting it into focused submodules reduces cognitive load and makes future changes safer and easier to review.

## What Changes
- Split HTTP notifications implementation into focused submodules under `crates/bastion-http/src/http/notifications/`
- Keep existing routing and `pub(super)` API stable (`notifications::*` used by the HTTP router)
- Preserve behavior; changes are structural/refactor-focused

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-http/src/http/notifications.rs`, `crates/bastion-http/src/http/notifications/*.rs`

## Compatibility / Non-Goals
- No API route changes and no JSON schema changes for existing clients.
- No behavioral changes intended; refactor only.

