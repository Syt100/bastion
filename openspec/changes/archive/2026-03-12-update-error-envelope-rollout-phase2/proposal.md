# Change: Phase 2 rollout for unified error envelope

## Why
The current rollout covers high-frequency run failure paths, but several important diagnostics surfaces still emit or render legacy-only error fields. This creates inconsistent user experience and makes cross-target error handling incomplete.

## What Changes
- Extend canonical `error_envelope` emission to remaining backend event producers that still write legacy-only failure fields (notably Agent-bridged snapshot-delete/task-result failures and execute-stage failures/warnings).
- Standardize `code`/`kind`/`retriable` and transport metadata for those remaining events while keeping legacy fields for compatibility.
- Update maintenance and snapshot management pages to prefer envelope-based diagnostics when available, with fallback to existing legacy task error fields.
- Add regression tests for new backend event envelopes and UI fallback behavior.
- Keep migration backward-compatible; no immediate removal of legacy DB/API fields in this phase.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code (planned):
  - `crates/bastion-http/src/http/agents/ws.rs`
  - `crates/bastion-engine/src/scheduler/worker/execute/*`
  - maintenance/snapshot-related UI views and stores
  - i18n locale files and related UI tests

## Non-Goals
- Replacing the HTTP request-response `AppError` contract in this phase.
- Dropping legacy `last_error_kind/last_error` columns or API fields in this phase.
