# Change: Fix long-page navigation loss with a fixed App Shell and pane-scoped scrolling

## Why
On long pages, the current layout allows the entire page to scroll.
This causes key navigation/context UI (top bar, left navigation / list panes) to scroll out of view, forcing users to scroll back to regain orientation and access primary actions.

This is especially painful in the Jobs workspace where operators frequently scan long History/Data content while needing persistent access to:
- the jobs list (to switch tasks quickly), and
- the job header (to confirm which job they are operating on).

## What Changes
- Introduce a **fixed App Shell** on desktop:
  - App header stays visible.
  - Main navigation (left sider) stays visible.
  - Only the main content region scrolls.
- Refine the **Jobs workspace** on desktop into a true “workbench”:
  - Jobs list pane and job workspace pane become **independent scroll containers**.
  - The jobs list pane keeps filters visible (sticky/pinned within the pane).
  - The job workspace pane keeps job context (job header + section tabs) visible while section content scrolls.
- Mobile behavior remains consistent with existing patterns:
  - Primary navigation uses a drawer.
  - Jobs workspace remains single-column, with clear back navigation.

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/layouts/AppShell.vue`
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue`
  - `ui/src/views/jobs/JobWorkspaceView.vue`
  - Shared layout styles under `ui/src/styles/*` (if needed)

## Non-Goals
- Preserving legacy routes or page-level URL compatibility (explicitly out of scope).
- Redesigning non-Jobs pages beyond adopting the fixed shell scrolling behavior.
- Changing backend behavior, APIs, or data models.
