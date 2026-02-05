# Change: Improve Jobs workspace ops management UX

## Why
Operators use the Jobs workspace in two high-frequency modes:

- **Scan and operate on many jobs**: search, filter, sort, bulk actions, and quick per-row actions.
- **Inspect a single job**: focus on job configuration, recent runs, and data lifecycle details.

The current Jobs workspace covers the core navigation, but it still has several ops friction points:

- It's easy to confuse **refresh list** vs **refresh job detail** when both are visible.
- Active filters (especially in split view where filters are collapsed) are not visible at a glance.
- Ops workflows lack **bulk selection + bulk actions** for common tasks.
- Table view needs stronger management affordances (header sorting, fixed columns, row open).
- Split view uses a fixed list width, which doesn't fit all screens and jobs naming patterns.
- Mobile job detail actions are not persistently accessible while scrolling.

## What Changes
- Clarify the **refresh affordances** for list vs detail (label/tooltip/accessibility).
- Add an **active filters summary** (chips) and a **results count** (filtered/total).
- Add **bulk selection** and safe **bulk actions** for common ops tasks:
  - Run now
  - Archive / unarchive
  - (Explicitly avoid dangerous bulk destructive actions like permanent delete)
- Improve table view:
  - Header click sorting (kept in sync with the sort control)
  - Fixed name/actions columns in horizontal scroll
  - Whole-row open affordance
- Add **quick per-row actions** in list view (hover-revealed) without requiring opening the detail pane.
- Add a **split-pane resizer** for the jobs list width with desktop-only persistence.
- Improve list-only detail access (clear affordance to open detail-only for the selected job).
- Improve keyboard and basic a11y (stable `name`/`aria-label` on filter controls; keyboard-open for rows where applicable).
- Improve mobile job detail by adding a **sticky actions affordance** (without degrading the current mobile navigation model).

## Impact
- Affected specs: `web-ui`
- Affected areas (representative):
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue` (filters, chips, selection, bulk actions, table affordances, split resizer)
  - `ui/src/views/jobs/JobWorkspaceView.vue` (detail actions affordances; refresh clarity)
  - `ui/src/components/MobileTopBar.vue` (optional actions slot / sticky mode support)
  - `ui/src/stores/ui.ts` (persist split list width and any related preferences)
  - i18n strings for new labels/tooltips
- Backend APIs: no changes required.

## Non-Goals
- Large-scale list performance work (virtualization, server-side pagination, debounce/async search).
- Adding a separate backup data management page.
- Bulk permanent delete.
- Changing core job semantics, scheduling behavior, or retention/snapshot behavior.

