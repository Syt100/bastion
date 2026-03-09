# Change: Unify error diagnostics consistency and fallback guidance

## Why
Recent commits improved upload-side failure hints, but read/cleanup paths still produce generic errors and sometimes misclassify auth/config issues as network failures. This causes misleading retries and weak run-event guidance for operators.

## What Changes
- Standardize WebDAV read/delete/download failure shaping so HTTP status, response message, and retry metadata are preserved for diagnosis.
- Align driver-layer error-kind mapping with WebDAV diagnostics so auth/config errors are not flattened into `network`.
- Improve run failure fallback hinting to avoid WebDAV-specific guidance for unrelated errors, and add explicit storage-capacity signals.
- Add actionable `hint` fields to cleanup/maintenance failure events so users can troubleshoot directly from run events.
- Localize the run-event detail hint label in Web UI.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code:
  - `crates/bastion-targets/src/webdav_client.rs`
  - `crates/bastion-driver-registry/src/builtins.rs`
  - `crates/bastion-engine/src/scheduler/worker/loop/local.rs`
  - `crates/bastion-engine/src/scheduler/incomplete_cleanup.rs`
  - `crates/bastion-engine/src/scheduler/artifact_delete.rs`
  - `ui/src/components/jobs/RunEventsModal.vue`
  - `ui/src/i18n/locales/en-US.ts`
  - `ui/src/i18n/locales/zh-CN.ts`

## Compatibility / Non-Goals
- No protocol or API route breaking change.
- No introduction of new WebDAV transport mechanisms.
- No change to cleanup scheduling policy itself; this change focuses on diagnostics and operator guidance.
