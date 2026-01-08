# Change: Refactor HTTP UI fallback module structure

## Why
`crates/bastion-http/src/http/mod.rs` mixes API routing setup with UI fallback handling (embedded/static UI serving, ETag/cache-control logic, and path safety checks). Extracting UI fallback into a focused module improves readability and maintainability.

## What Changes
- Extract UI fallback + asset serving helpers into `crates/bastion-http/src/http/ui.rs`
- Keep the router behavior stable (same routes, middleware, and fallback behavior)
- Preserve behavior; changes are structural/refactor-focused

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-http/src/http/mod.rs`, `crates/bastion-http/src/http/ui.rs`

## Compatibility / Non-Goals
- No changes intended to routing, cache-control semantics, ETag behavior, or UI path safety rules.

