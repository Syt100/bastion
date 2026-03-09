# Change: Refactor HTTP secrets module structure

## Why
`crates/bastion-http/src/http/secrets.rs` is a large module mixing unrelated concerns (node validation, WebDAV secret CRUD, WeCom bot secret CRUD, and SMTP secret CRUD). Splitting it into focused submodules reduces cognitive load and makes future changes safer and easier to review.

## What Changes
- Split HTTP secrets implementation into focused submodules under `crates/bastion-http/src/http/secrets/`
- Keep existing routing and `pub(super)` API stable (`secrets::*` used by the HTTP router)
- Preserve behavior; changes are structural/refactor-focused

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-http/src/http/secrets.rs`, `crates/bastion-http/src/http/secrets/*.rs`

## Compatibility / Non-Goals
- No API route changes and no JSON schema changes for existing clients.
- No behavioral changes intended; refactor only.

