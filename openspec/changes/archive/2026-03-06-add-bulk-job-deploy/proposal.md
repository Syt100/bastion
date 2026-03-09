# Change: Bulk Deploy (Clone) Jobs to Nodes

## Why
Operators often want to roll out the same backup job across a fleet of agents.
Doing this manually is slow and increases the chance of inconsistent naming, schedules, or missing prerequisites.

This change adds a bulk “job deploy” operation that:
- Clones an existing job to multiple target nodes selected by labels.
- Uses a naming template (default includes node id) to avoid confusion.
- Performs per-node validation and reports missing prerequisites (e.g., missing WebDAV secret).
- Continues on per-node failures and records results.

## What Changes
- Backend:
  - Add a bulk operation action to deploy (clone) an existing job to multiple target nodes.
  - Preserve job fields (spec, schedule, schedule timezone, overlap policy) unless overridden.
  - Validate per-node prerequisites (e.g., node-scoped secrets referenced by the job spec) and fail those nodes with a clear error.
  - Provide a preview capability for the UI (dry-run plan).
- Web UI:
  - Add a “Deploy to nodes” action for a job.
  - Allow selecting target nodes via labels (AND/OR) or explicit selection.
  - Provide a naming template input (default `"{name} ({node})"`).
  - Show a preview of planned job names and per-node validation results before execution.
  - Reuse the bulk operations panel for execution status and errors.

## Dependencies
- Uses the bulk operations framework from `add-bulk-operations-framework`.
- Uses agent labels from `add-agent-labels`.
- Node-scoped credential prerequisites can be satisfied via `add-bulk-webdav-distribution` (separate change).

## Decisions
- Name template default includes node id to reduce ambiguity.
- If name template still collides, the system auto-suffixes (e.g., `… (node) #2`).
- The operation continues on failures and records per-node outcomes.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code (expected): `crates/bastion-http` (bulk action + jobs), `crates/bastion-storage` (bulk + jobs), `ui` (jobs UI + bulk panel)

