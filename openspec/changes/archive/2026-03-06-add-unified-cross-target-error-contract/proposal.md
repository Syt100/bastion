# Change: Add unified cross-target error contract

## Why
The current error experience is improved but still fragmented across API, driver, run-event, and UI layers. Some paths rely on message text matching, hints are still partly hardcoded in English, and protocol-specific diagnostics (for example HTTP status) are not generalized for future target types such as SFTP or cloud-drive APIs.

## What Changes
- Define a transport-agnostic error envelope for target-side failures and operation failures.
- Standardize required fields (`code`, `kind`, `retriable`, localization keys, context, and diagnostics identity) across backend emissions.
- Add protocol extension rules so HTTP/WebDAV, SFTP, and drive-style APIs can report diagnostics without forcing HTTP-only fields.
- Add support for async-operation and partial-failure diagnostics to cover cloud-drive and batch-delete style targets.
- Define backward-compatible rollout rules so legacy event readers continue working during migration.
- Add a target capability matrix in design guidance to reduce ambiguity when introducing new target adapters.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code (planned):
  - `crates/bastion-core`
  - `crates/bastion-engine`
  - `crates/bastion-driver-api`
  - `crates/bastion-driver-registry`
  - `crates/bastion-targets`
  - `ui/src/lib/errors.ts`
  - `ui/src/components/jobs/RunEventsModal.vue`
  - maintenance/snapshot-delete related UI detail pages

## Compatibility / Non-Goals
- No immediate removal of existing event fields in the first rollout phase.
- No commitment to support every target protocol in this change; this change defines the contract required for future adapters.
- No breaking API contract in this stage; old fields remain readable until migration is complete.
