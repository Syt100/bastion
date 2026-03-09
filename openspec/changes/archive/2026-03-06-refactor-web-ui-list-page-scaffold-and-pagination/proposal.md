# Change: Refactor list pages with shared scaffold, lower visual noise, and unified pagination

## Why
The current Web UI list pages (Jobs, Agents, Notifications Queue) have diverged in structure and interaction:

- The same page skeleton (selection bar, toolbar, list/table container, empty state, pagination) is reimplemented multiple times.
- Pagination behavior is inconsistent (for example, Notifications Queue uses previous/next controls while other pages use page + page-size selectors).
- Visual hierarchy is noisy in list contexts, especially where empty states render as nested cards inside existing cards.

This increases maintenance cost, creates UX inconsistency across pages, and makes future page evolution slower.

## What Changes
- Introduce a shared list-page foundation for Web UI pages:
  - `ListPageScaffold` (or equivalent shared composition primitives) for the common page skeleton.
  - Shared slot contract for selection tools, filters/actions, content region, empty/loading states, and footer pagination.
- Standardize pagination into a unified pattern via shared `AppPagination` behavior:
  - page number + page-size picker + total count semantics.
  - consistent disabled/loading behavior and placement.
- Reduce list-view visual noise:
  - extend `AppEmptyState` with context-aware rendering variants (`card` / `inset` / `plain`).
  - use non-card empty states when the parent already provides a card/surface.
  - avoid stacked decorative wrappers for list toolbars and list empty states.
- Migrate representative list pages to the shared pattern:
  - Jobs workspace list panel.
  - Agents list page.
  - Notifications Queue list page.

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/components/AppEmptyState.vue`
  - `ui/src/components/list/ListToolbar.vue`
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue`
  - `ui/src/views/AgentsView.vue`
  - `ui/src/views/settings/notifications/NotificationsQueueView.vue`
  - new shared list scaffolding/pagination components under `ui/src/components/list/`
- Backend APIs: no changes.

## Non-Goals
- Replacing existing table/list business logic, filtering semantics, or backend query models.
- Redesigning dashboard cards or non-list pages in this change.
- Introducing new data capabilities beyond UI structure/interaction consistency.
