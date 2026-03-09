# Design: Jobs workspace overview summary + compact section toolbars

## Context
Jobs workspace already provides stable navigation:

`Node → Jobs → Job → (Overview | History | Data) → Run (drawer)`

This change refines information placement and action density:
- Overview becomes the “status dashboard” for the job.
- History becomes “runs list for investigation”.
- Data becomes “retention + snapshots management”, with compact per-panel actions.

## Goals / Non-Goals
- Goals:
  - Surface the most important run signal on Overview (latest run + 7-day summary).
  - Reduce vertical chrome, especially on mobile, by avoiding standalone action rows.
  - Keep navigation semantics stable (deep linkable; drawer open/close remains route-driven).
- Non-Goals:
  - Adding a new backend aggregation endpoint (can be considered later).
  - Adding arbitrary time-range analytics UI.

## Information Architecture

### Overview
Add a “Run Summary (last 7 days)” block.

Content:
- Latest run row:
  - Status tag + started time (and optional duration)
  - Primary action: open run drawer (deep link to `/history/runs/:runId`)
- 7-day chips:
  - Total, Success, Failed (optional: Rejected)

Empty state:
- If there are no runs in the last 7 days:
  - Show chips with 0 values and a subtle empty hint.
  - Latest run row shows “—” and does not offer the open action.


### History
History focuses on the runs list.

- Remove the large summary card grid currently shown above the list.
- Place actions (Refresh; future filters) into the list panel header.


### Data
Data keeps the two mental buckets but improves action placement.

- Retention panel:
  - Header-left: “Retention” + dirty/saved indicator (optional)
  - Header-right: primary “Save”
  - Secondary actions (Preview/Apply/Reset) go into overflow.
- Snapshots panel:
  - Header-right: Refresh + overflow for secondary actions
  - List content follows directly under the header.


## Mobile Behavior
- Overview:
  - Latest run row uses a compact two-line layout.
  - 7-day metrics render as horizontally scrollable chips or a 2x2 small-card grid.
- Section actions:
  - Prefer icon-only actions in header-right.
  - Use an overflow menu when actions would wrap.
  - No dedicated action row above lists/forms.

## Implementation Notes (UI)
- Prefer a single runs fetch per job and share it between Overview and History when practical.
- 7-day window derived from `started_at` timestamps.
- Keep routes unchanged; this is layout/content rebalancing.
