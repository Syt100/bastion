# Change: Refactor web UI shared list foundation (phase 2)

## Why
Recent consistency refactors fixed major UX drift, but several front-end code-level issues still increase maintenance cost: oversized page files, duplicated Jobs row rendering, route-filter parsing inconsistency, pagination behavior divergence, duplicated picker open/reset logic, repeated per-id busy-state maps, and repeated store-side query serialization.

## What Changes
- Extract shared view/composable infrastructure so large list pages and pickers can split business state from rendering concerns.
- Split Jobs workspace row/table rendering into reusable subcomponents to remove desktop/mobile duplication while keeping current interaction semantics.
- Add a shared route-query filter parsing layer and migrate list pages to consistent query-to-filter hydration behavior.
- Standardize pagination behavior by reusing shared pagination primitives/constants across list pages; migrate pages still using ad-hoc prev/next pagination controls.
- Add shared picker session/reset modeling and migrate pickers with large open/reset blocks to reduce repeated lifecycle boilerplate.
- Add a shared per-id busy-state composable and migrate pages that currently duplicate `Record<string, boolean>` busy maps.
- Add shared list-query builders for common store request serialization patterns and migrate stores currently hand-building repeated `URLSearchParams` logic.
- Keep front-end code splitting/lazy loading and virtual scrolling out of scope.

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue`
  - `ui/src/views/AgentsView.vue`
  - `ui/src/views/settings/maintenance/MaintenanceCleanupView.vue`
  - `ui/src/views/settings/notifications/NotificationsQueueView.vue`
  - `ui/src/components/pickers/pathPicker/PathPickerModal.vue`
  - `ui/src/components/jobs/RunEntriesPickerModal.vue`
  - `ui/src/lib/listFilters.ts`
  - `ui/src/lib/listUi.ts`
  - `ui/src/lib/listQuery.ts`
  - `ui/src/lib/asyncControl.ts`
  - related view/component/store tests under `ui/src/**`

## Non-Goals
- Changing API contracts or backend pagination semantics.
- Introducing virtual scrolling strategies.
- Introducing front-end bundle split / lazy loading changes.
