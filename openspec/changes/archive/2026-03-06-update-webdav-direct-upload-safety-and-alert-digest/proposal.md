# Change: Update WebDAV direct upload safety, explicit config, and alert digest

## Why
Raw-tree backups can generate a very large number of small files. When we upload these files to WebDAV, the naive approach (many `PUT`/`HEAD`/`MKCOL` requests) can overwhelm the target server.

Separately, the current “engine auto-enables WebDAV raw-tree direct upload when conditions match” behavior makes production behavior surprising and hard to operate: users cannot explicitly choose when direct upload is acceptable, nor can they tune safety limits per job.

Finally, warning signals can become noisy. Operators want the job “runs list” to surface only high-signal summaries (fast to scan), while the run detail page retains full samples and evidence for diagnosis.

## What Changes

### 1) WebDAV direct upload: concurrency + rate limiting
- Add first-class upload safety limits for WebDAV requests:
  - **Concurrency limit** (max in-flight requests)
  - **Rate limit** (requests per second), with optional burst
- Apply limits to:
  - raw-tree staged upload to WebDAV (`data/` directory traversal)
  - raw-tree direct upload to WebDAV (during packaging)
- Preserve atomic semantics: `complete.json` MUST still be written last.
- Add best-effort backpressure handling for overloaded servers (e.g. HTTP 429/503 + `Retry-After`).

### 2) Make direct upload an explicit job/pipeline configuration
- Introduce an explicit job spec knob for WebDAV raw-tree direct upload:
  - `mode: off|auto|on` (default: `off`)
- Remove “engine auto-enables” behavior; the engine SHALL only enable direct upload when the job spec explicitly opts in.
- Add validation rules so invalid/unsafe combinations fail fast (e.g. direct upload + “fail on consistency” with upload disabled).

### 3) Alert de-noising / aggregation strategy (runs list vs detail)
- Add a structured “alert digest” signal for `GET /api/jobs/:id/runs`:
  - Keep the existing “early signal for running runs” behavior.
  - Add high-signal breakdown fields (e.g. replaced/deleted/read_error vs changed-only).
- Update the Web UI runs list to show only high-signal summary badges (capped and ordered).
- Keep run detail as the place to drill into samples and evidence.

## Impact
- Affected specs: `targets-webdav`, `backup-jobs`, `control-plane`, `web-ui`
- Affected code (expected):
  - `crates/bastion-core/src/job_spec/*` (job spec types + validation)
  - `crates/bastion-engine/src/scheduler/worker/execute/filesystem.rs` (enablement logic)
  - `crates/bastion-targets/src/webdav_client.rs` and `crates/bastion-targets/src/webdav.rs` (limits + concurrent upload)
  - `crates/bastion-backup/src/backup/filesystem/raw_tree.rs` (direct upload pipeline)
  - `crates/bastion-http/src/http/jobs/runs.rs` (runs list API fields)
  - `ui/src/components/jobs/editor/*` (job editor controls)
  - `ui/src/views/jobs/JobHistorySectionView.vue` (runs list badges)

## Compatibility / Non-Goals
- Backward compatibility is not a goal for this change; job spec and runs list payloads MAY change.
- We do not attempt to implement a generic “server-wide global rate limit across all jobs”; limits are per-run execution context.
- We do not remove evidence: samples and fingerprint evidence remain available in run detail.

