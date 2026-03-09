# Design: Jobs workspace (master-detail) + job-scoped Run Detail drawer

## Summary
This change turns Jobs into a “workspace” with a stable hierarchy:

`Node → Jobs → Job → (Overview | History | Data) → Run (drawer overlay)`

The key design goals are:
- keep job context stable while browsing runs/snapshots,
- reduce full-page navigation hops,
- support deep links and mobile-first navigation without forking behavior.

## Route Map (New)

### Workspace entry
- `/n/:nodeId/jobs` — Jobs workspace shell.
  - Desktop: list + detail split view.
  - Mobile: list-only (no job selected).

### Job workspace sections
- `/n/:nodeId/jobs/:jobId` → redirects to `/n/:nodeId/jobs/:jobId/overview`
- `/n/:nodeId/jobs/:jobId/overview`
- `/n/:nodeId/jobs/:jobId/history`
- `/n/:nodeId/jobs/:jobId/data`

### Run drawer overlay (job-scoped)
The drawer is opened by navigating to a run route under the currently active section so that closing the drawer returns to the same section.

- `/n/:nodeId/jobs/:jobId/overview/runs/:runId`
- `/n/:nodeId/jobs/:jobId/history/runs/:runId`
- `/n/:nodeId/jobs/:jobId/data/runs/:runId`

Canonical external links (from non-jobs pages) SHOULD use the History section:
- `/n/:nodeId/jobs/:jobId/history/runs/:runId`

## Layout

### Desktop (master-detail)
- Left pane: Jobs list (search/filter/sort, primary “Create job” action).
- Right pane: Job workspace for `:jobId`, with:
  - a stable job header (name, node context, archived state),
  - section tabs (Overview/History/Data),
  - content area (section-specific),
  - and a run drawer overlay when the route includes `/runs/:runId`.

The jobs list and job workspace should scroll independently to avoid losing context.

### Mobile (single-column)
- `/n/:nodeId/jobs`: jobs list full-screen.
- Selecting a job navigates to `/n/:nodeId/jobs/:jobId/overview`.
- Job workspace uses a mobile top bar with a clear “Back to Jobs” affordance.
- Run drawer uses a full-screen drawer, opened via the run overlay route.

## Section Content

### Overview
Purpose: a fast “health + actions” view.
Recommended content:
- primary actions (Run now, Edit, Deploy) + overflow “More”
- latest run status (and quick link to open that run)
- schedule + timezone + overlap policy summary
- retention summary (enabled? keep rules?) with link to Data section

### History
Purpose: runs list and operational investigation.
Recommended content:
- runs list (table on desktop, cards on mobile)
- opening a run navigates to the run drawer overlay route (no full-page switch)

### Data
Purpose: snapshots management + retention in one mental bucket.
Recommended content:
- snapshots list (reuse existing Snapshots list component in embedded mode)
- retention policy editor/panel co-located on this page (not a separate tab/page)

## Actions & Safety
- Job header actions:
  - primary: “Run now”
  - secondary: “Edit”, “Deploy”
  - destructive actions (Archive, Delete permanently) MUST be behind “More” and MUST have explicit confirmations.
- Run drawer actions remain (Restore, Verify, Copy Run ID, etc.).

## Implementation Notes (UI)
- Derive “selected job” from the route (`:jobId`) rather than local component state.
- The Jobs list should highlight the selected job on desktop based on `:jobId`.
- Closing the run drawer is navigation back to the parent section route (no ad-hoc state).
- All existing links to `/n/:nodeId/runs/:runId` must be updated to include `job_id` and use the canonical History run route.

## Removal / Cleanup
- Remove `/n/:nodeId/runs/:runId` route and replace usages.
- Remove or repurpose the existing Job Detail shell view and its tab routing.
- Move JSON/inspect view behind “More” (not a top-level section).

