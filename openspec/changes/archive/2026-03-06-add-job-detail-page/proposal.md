# Change: Add Job Detail page (runs, snapshots, retention) and simplify Jobs list actions

## Why
Jobs are currently managed from the Jobs list with multiple per-row actions, and run history is shown in a modal.
This makes it harder to:
- deep-link/share a job view
- navigate back/forward naturally
- manage related job resources (runs, snapshots, retention) in one place

We want a job-centric page that reduces action clutter while keeping advanced operations accessible.

## What Changes
- Add a node-scoped Job Detail page at `/n/:nodeId/jobs/:jobId`.
  - Tabs: Runs, Snapshots, Retention, Settings (actions)
  - Links to Run Detail and common actions (restore/verify) from the Runs tab.
- Update the Jobs list UI to:
  - provide a clear “Open”/row click entry point to Job Detail
  - reduce per-row button clutter by moving secondary actions into a “More” menu
  - keep primary actions easy (e.g. Run now)

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/router/index.ts`
  - `ui/src/views/JobsView.vue`
  - new view/component under `ui/src/views/` (Job detail)
  - reuse existing stores/endpoints in `ui/src/stores/jobs.ts`

## Non-Goals
- Changing backend job/run/snapshot schemas or APIs.
- Building a full “job analytics” view (dashboard-level analytics is handled separately).

