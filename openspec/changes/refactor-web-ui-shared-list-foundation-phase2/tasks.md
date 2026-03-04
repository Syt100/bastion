## 1. Spec
- [x] 1.1 Draft proposal/design/spec delta for shared list foundation phase-2 refactor
- [x] 1.2 Run `openspec validate refactor-web-ui-shared-list-foundation-phase2 --strict`

## 2. Shared Infrastructure
- [x] 2.1 Add shared route query parsing helpers for single/multi filter hydration
- [x] 2.2 Add shared pagination constants/helpers and align `AppPagination` consumers
- [x] 2.3 Add shared per-id busy-state composable and migrate pages using duplicated busy maps
- [x] 2.4 Add shared list query serialization helpers for store list APIs and migrate duplicated `URLSearchParams` assembly
- [x] 2.5 Add shared picker open/reset state composable for repeated picker lifecycle boilerplate

## 3. View Refactors
- [x] 3.1 Split Jobs workspace list/table row rendering into reusable components/composables
- [x] 3.2 Migrate Agents route-filter hydration and related list logic to shared route-filter parsing
- [x] 3.3 Migrate Maintenance Cleanup pagination interaction to shared `AppPagination` behavior
- [x] 3.4 Migrate PathPicker and RunEntries picker open/reset flows to shared picker reset model

## 4. Tests / Validation
- [x] 4.1 Add/update unit tests for new shared helpers/composables
- [x] 4.2 Add/update regression tests for Jobs/Agents/Maintenance/picker migrations
- [x] 4.3 Run `npm test --prefix ui -- --run`
- [x] 4.4 Run `scripts/ci.sh`
- [x] 4.5 Update `CHANGELOG.md` via `maintain-changelog-release` for user-visible front-end consistency improvements
