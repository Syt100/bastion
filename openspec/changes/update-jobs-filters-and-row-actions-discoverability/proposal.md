# Change: Unify Jobs filters and improve row-action discoverability

## Why
The Jobs workspace currently has two UX/maintainability issues:

- Filter controls are fragmented across split/list/mobile paths, causing duplicated code and inconsistent interaction details.
- Key row actions are primarily hover-revealed, which reduces discoverability for first-time users, touch users, and keyboard-driven workflows.

This makes the Jobs page harder to evolve safely and increases the chance of behavior drift across layouts.

## What Changes
- Consolidate Jobs filter state into a single source of truth (`useJobsFilters` or equivalent), covering:
  - search text
  - archived toggle
  - latest-run status filter
  - schedule filter
  - sort key
- Provide a shared Jobs filter panel renderer used across split/list/mobile containers:
  - layout container may differ (inline, popover, drawer)
  - filter controls, options, and clear behavior remain consistent.
- Improve row action discoverability:
  - keep primary row actions visible by default (not hover-only), with overflow for secondary actions.
  - preserve selection and row-navigation behavior without action-button hit-area conflicts.
- Strengthen keyboard/touch accessibility for row actions and filter controls.

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue`
  - new Jobs-specific composables/components under `ui/src/views/jobs/` or `ui/src/components/jobs/`
  - potential reuse/update of `ui/src/components/list/OverflowActionsButton.vue`
  - i18n strings for any updated action labels/hints
- Backend APIs: no changes.

## Non-Goals
- Changing Jobs backend filtering semantics or adding new backend query fields.
- Redesigning job detail page layouts outside the list panel concerns.
- Introducing additional list modes beyond current split/list/detail model.
