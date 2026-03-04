# Change: Refactor list filter componentization and visibility consistency

## Why
After recent list-page UX upgrades, filter behavior still diverges across pages outside Jobs. Agents, Notifications Queue, Maintenance Cleanup, and Job Snapshots still use page-local filter state/chip logic and hand-written filter control blocks, which increases duplicate code and raises style/interaction drift risk.

## What Changes
- Introduce shared list-filter modeling utilities for active-state derivation, clear behavior, and active-filter chips.
- Introduce shared list filter field wrappers for select controls to standardize width/layout and reduce repeated toolbar markup.
- Introduce a shared active-filters row wrapper for consistent filter visibility/clear affordances.
- Migrate Agents, Notifications Queue, Maintenance Cleanup, and Job Snapshots list pages to the shared filter model + UI wrappers.
- Keep existing server contracts and route/filter semantics unchanged while unifying presentation and local state plumbing.

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/lib/listFilters.ts` (new)
  - `ui/src/components/list/ListFilterSelectField.vue` (new)
  - `ui/src/components/list/ListActiveFiltersRow.vue` (new)
  - `ui/src/views/AgentsView.vue`
  - `ui/src/views/settings/notifications/NotificationsQueueView.vue`
  - `ui/src/views/settings/maintenance/MaintenanceCleanupView.vue`
  - `ui/src/views/JobSnapshotsView.vue`
  - related view tests under `ui/src/views/**.spec.ts`

## Non-Goals
- Reworking list server-side filtering/pagination APIs.
- Introducing virtual scrolling.
- Redesigning page-specific business filters beyond consistency and componentization.
