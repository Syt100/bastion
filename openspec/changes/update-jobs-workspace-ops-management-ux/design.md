# Design: Jobs workspace ops management UX improvements

## Goals
- Increase ops efficiency in **list-only management** without harming the existing split/master-detail workflow.
- Keep UX consistent with other ops pages (e.g. Agents/Clients) in layout, spacing, and control patterns.
- Preserve mobile's single-column navigation model while making job actions easier to reach.

## Key UX Decisions

### 1) Refresh clarity (list vs detail)
- Keep the page-level refresh action in the Jobs header as **"Refresh list"** (tooltip/aria label, keep visible on desktop & mobile list).
- Keep the job detail refresh action in the Job header as **"Refresh job"** (tooltip/aria label).
- Do not auto-refresh the other pane when one refresh is triggered (avoid surprising network activity).

### 2) Filters: visibility + quick removal
- Add a compact **results counter**: `filtered / total` next to the filters area.
- Add an **active filters chips row** that:
  - appears when any filter is active,
  - shows each active filter as a closable chip,
  - provides a single "Clear all" action.
- In split view (narrow list pane), chips row is **single-line horizontal scroll** to avoid consuming vertical space.

### 3) Bulk selection + safe bulk actions
- Bulk selection is supported in list-only management mode:
  - **Table view**: uses `NDataTable` selection column + `checkedRowKeys`.
  - **List view**: provides an explicit "Select" mode toggle that reveals checkboxes on rows.
- Bulk actions are intentionally limited to safe operations:
  - Run now (skips archived jobs)
  - Archive / unarchive (with confirmation; archive supports optional cascade snapshots)
- Bulk permanent delete is out of scope.
- UI uses the existing `SelectionToolbar` pattern to show selection count + actions + clear selection.

### 4) Table view management affordances
- Support header click sorting for Name and Updated At (kept in sync with the sort control).
- Fix **Name** to the left and **Actions** to the right to keep critical controls visible while horizontally scrolling.
- Provide row open affordance (single click on name, double click on row, or explicit "Open details" action).

### 5) Split pane resizing
- Add a drag handle on the right edge of the list pane in split view.
- Persist the list pane width on desktop only in `ui` store (localStorage).
- Clamp width to a reasonable range so it cannot collapse or starve the detail pane.

### 6) Mobile sticky actions
- Extend `MobileTopBar` to optionally render actions via a slot.
- In Job detail on mobile, move primary actions into the top bar actions area and make the bar optionally sticky:
  - Run now (primary)
  - More (dropdown)
  - Refresh (in dropdown or as an icon button, depending on width)

## Persistence
- `bastion.ui.jobsWorkspace.splitListWidthPx`: number (desktop-only)
- Any optional preference toggles (e.g. list view select mode default) SHOULD remain local-only and desktop-only.

## Risks / Mitigations
- **Overcrowding toolbars**: keep bulk selection controls scoped to list-only layout; keep split view compact.
- **Bulk ops mistakes**: confirmations for archive/unarchive; disable run now/edit for archived.
- **Mobile space constraints**: action icons in top bar; dropdown for lower-frequency actions.

