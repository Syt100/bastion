# Change: Fix multi-select filter query parsing (`status[]`, `channel[]`, `target_type[]`)

## Why
The web UI uses multi-select filters for several list pages (incomplete run cleanup, notifications queue).
When the client encodes multi-select as repeated `[]` query params (e.g. `status[]=queued&status[]=done`), the backend must correctly interpret those filters.

## What Changes
- Make the backend list endpoints accept multi-value filters encoded as either:
  - repeated non-bracket keys (e.g. `status=queued&status=done`), or
  - repeated bracket keys (e.g. `status[]=queued&status[]=done`)
- Add regression tests to ensure multi-select filters are actually applied (not ignored).

## Impact
- Affected specs: `backend`
- Affected code:
  - `crates/bastion-http/` (query parsing for list endpoints)
  - `crates/bastion-http/` tests (multi-value filter coverage)

## Compatibility / Non-Goals
- No new filter dimensions are introduced.
- Existing single-value query params continue to work.

