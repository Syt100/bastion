# Change: Improve rolling upload failure observability and operator guidance

## Why
Rolling archive uploads can fail due to network/proxy/storage constraints, but current run failures often surface as generic `rolling uploader dropped`. This obscures root cause, slows incident triage, and forces operators to inspect external systems manually.

## What Changes
- Preserve uploader root-cause errors across async/sync boundaries in rolling archive mode, instead of collapsing to generic channel-drop text.
- Make executor error join behavior robust: collect both packaging and uploader outcomes, then return a deterministic, root-cause-first failure.
- Add structured run failure diagnostics (error code/kind, hint, HTTP status, part metadata, retry context, error chain) so UI and APIs can surface actionable guidance.
- Improve WebDAV upload error classification and hints (payload too large, timeout/reset, auth/permission, rate limit, upstream/storage issues).
- Extend WebDAV request limit settings with timeout/retry controls suitable for slow links and constrained gateways.
- Improve run events UI readability for failure diagnostics, including explicit hint/status cues without requiring raw JSON inspection.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code:
  - Rolling uploader bridge and execute flows (`crates/bastion-engine/src/scheduler/worker/execute/*`)
  - Run terminal error event emission (`crates/bastion-engine/src/scheduler/worker/loop/local.rs`)
  - Archive writer error propagation (`crates/bastion-backup/src/backup/filesystem/tar/entry.rs`)
  - WebDAV client/target diagnostics and retry behavior (`crates/bastion-targets/src/webdav*.rs`)
  - Job spec + validation + conversions for request tuning (`crates/bastion-core/src/job_spec/*`, driver bridges)
  - Run events UI chips/details (`ui/src/components/jobs/RunEventsModal.vue`)

## Non-Goals
- Introducing resumable multipart uploads at protocol level (server-side upload session semantics).
- Replacing current WebDAV transport stack.
- Adding automatic remote cleanup policies beyond existing explicit cleanup behavior.
