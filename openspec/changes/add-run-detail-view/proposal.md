# Change: Add node-scoped Run Detail view (events + restore/verify operations)

## Why
Runs are currently inspected via modals (runs list + events), and restore/verify work is started in separate dialogs. Once the dialog is closed, users lose the “thread” of what happened for that run.

## What Changes
- Add a **Run Detail** page under node context: `/n/:nodeId/runs/:runId`
  - run overview (status/timestamps/error/summary)
  - live run events (existing WS stream)
  - a sub-list of linked operations (restore/verify) for the run
  - actions to start restore/verify from the run context
- Add a run read API (`GET /api/runs/{run_id}`) so the UI can load the page from a run id.
- Update runs list UI to deep-link into Run Detail.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code:
  - Backend API: `crates/bastion-http/src/http/*`
  - UI routing/views: `ui/src/router/*`, `ui/src/views/*`, `ui/src/components/jobs/*`

## Non-Goals
- Replacing all existing modals immediately (they may remain as shortcuts).

