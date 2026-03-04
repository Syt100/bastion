## 1. Spec
- [x] 1.1 Draft proposal/design/spec delta for shared filter/picker infrastructure hardening
- [x] 1.2 Run `openspec validate refactor-web-ui-shared-filter-picker-infrastructure --strict`

## 2. Shared Infrastructure
- [x] 2.1 Add shared filter-chip type definitions under `ui/src/lib` and migrate `listFilters` consumers off component-exported chip types
- [x] 2.2 Add shared picker/list query parameter builder helpers (search/filter/size/sort serialization)
- [x] 2.3 Add shared debounce + abort-error utility helpers
- [x] 2.4 Add shared picker loaded-row selection composable for select-all/invert/range semantics

## 3. Feature Migrations
- [x] 3.1 Migrate `RunEntriesPickerModal` to stale-request-safe refresh/load-more flow
- [x] 3.2 Migrate `RunEntriesPickerModal` to shared query builder + loaded-row selection composable
- [x] 3.3 Migrate `PathPickerModal` to shared loaded-row selection composable
- [x] 3.4 Migrate FS/WebDAV picker data sources to shared query builder helpers
- [x] 3.5 Migrate Jobs workspace filters to `useListFilters` + shared active-filter row
- [x] 3.6 Migrate list views with duplicated debounce/abort logic to shared utilities

## 4. Tests / Validation
- [x] 4.1 Add/update unit tests for new shared helpers/composables
- [x] 4.2 Add/update regression tests for Jobs filters and picker request/selection behavior
- [x] 4.3 Run `npm test --prefix ui -- --run`
- [x] 4.4 Run `scripts/ci.sh`
- [x] 4.5 Update `CHANGELOG.md` for user-visible front-end consistency refactor
