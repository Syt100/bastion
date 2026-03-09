# Change: Refactor jobs CRUD handler helpers

## Why
`crates/bastion-http/src/http/jobs/crud.rs` contains repeated request normalization and validation logic (name/schedule normalization, agent validation, cron validation, snapshot notifications). Consolidating this logic into focused helpers reduces duplication and improves maintainability.

## What Changes
- Extract shared jobs CRUD validation/normalization into private helper functions
- Keep HTTP handler behavior and API stable
- Keep existing error codes and messages

## Impact
- Affected specs: `backend`
- Affected code: `crates/bastion-http/src/http/jobs/crud.rs`

## Compatibility / Non-Goals
- No behavior changes intended for job CRUD semantics, validation rules, or notification side effects.

