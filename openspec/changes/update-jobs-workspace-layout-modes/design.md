# Design: Jobs workspace layout modes + list/table views

## Summary
This change extends the existing desktop Jobs workspace master-detail design to support ops-first workflows:

- **Split**: list + detail (default "workbench" mode)
- **List-only**: full-width list (management mode)
- **Detail-only**: full-width detail (inspection mode)

Additionally, the jobs list supports a **List/Table view toggle**. Table view is only enabled in List-only mode to ensure adequate width and reduce UI complexity.

Mobile keeps the existing single-column navigation behavior.

## Layout Modes (Desktop)

### Split (default)
- Show the jobs list pane on the left and the job workspace pane on the right.
- Keep independent scrolling for list vs content pane.

### List-only
- Hide the job workspace pane.
- The jobs list pane expands to fill available width.
- If a job is currently selected in the URL, it remains "selected" (highlighted), but the detail pane stays hidden until the user switches back.

### Detail-only
- Hide the jobs list pane.
- The job workspace pane expands to fill available width.
- If no job is selected (route is `/n/:nodeId/jobs`), the UI falls back to Split or List-only so the user is not presented with an empty detail page.

## Jobs List Views

### List view (scanability-first)
- Uses the existing row layout in both Split and List-only modes.
- In List-only mode, the same row layout MAY increase spacing slightly and MAY reveal one extra secondary line, but it MUST remain recognizably the same shape (no hard switch to a different card concept).

### Table view (ops management)
- Available only when the list pane is full width (List-only layout mode).
- Optimized for scanning and sorting:
  - Typical columns: Name, Node, Schedule (+TZ), Latest status, Latest run time, Updated time, Actions.
- Row actions remain available (Run now / Edit / More).
- Table view is not shown on mobile-sized screens.

## Controls / Affordances
- Provide clear, discoverable controls to switch layout modes:
  - A list-pane control to hide/show the detail pane (Split <-> List-only).
  - A detail-pane control to hide/show the list pane (Split <-> Detail-only).
  - When in List-only or Detail-only, provide a control to return to Split.
- Provide a list view toggle (List vs Table) inside the list toolbar.
  - Selecting Table view implicitly switches the workspace to List-only mode.

Controls should prefer icon + tooltip patterns on dense toolbars, and MUST be keyboard accessible.

## State and Persistence
- Persist on desktop:
  - Workspace layout mode: split | list | detail
  - List view mode: list | table
- Persistence scope: per-user (browser local storage) with a stable key namespace (implementation detail).
- On mobile, ignore persisted desktop layout mode and use single-column navigation.

## Routing Notes
- Continue to derive "selected job" from the route (`:jobId`) rather than local component state.
- Layout/view toggles SHOULD NOT require route changes, except optional query parameters if needed for deep links (implementation choice).

