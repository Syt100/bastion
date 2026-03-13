# Change: Refactor Web UI shared filter/picker infrastructure

## Why
Recent list/picker consistency work removed several duplicated filter UI paths, but a few code-level inconsistencies remain: request race protection differs between picker surfaces, Jobs still uses a custom filter model, picker selection/query plumbing is duplicated, and debounce/abort helpers are reimplemented per page.

## What Changes
- Add stale-request protection for `RunEntriesPickerModal` refresh/load-more flows to avoid outdated responses overriding newer filter/path state.
- Extract picker loaded-row selection behavior (`select all` / `invert` / `range`) into a shared composable and migrate both RunEntries/Path picker modals.
- Migrate Jobs workspace filter chip/count/clear modeling from page-local logic to `useListFilters`, and use the shared active-filters row component.
- Decouple filter chip types from picker UI component exports by introducing shared list-filter type definitions in `lib`.
- Extract shared query-building helpers for picker/list request parameters (search/kind/dotfiles/type sort/size-range/sort), then reuse in FS/WebDAV data sources and RunEntries picker.
- Extract shared debounce + abort-error helpers and migrate Jobs/Agents list refresh scheduling and list views with abort guards.
- Split oversized filter/request utility logic from major views into composables where needed to reduce file-level complexity and behavior drift risk.
- Keep front-end bundle splitting/lazy-load tuning out of scope for this change.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/jobs/RunEntriesPickerModal.vue`
  - `ui/src/components/pickers/pathPicker/PathPickerModal.vue`
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue`
  - `ui/src/views/jobs/useJobsFilters.ts` (migration/removal)
  - `ui/src/lib/listFilters.ts` + new shared helper/type modules
  - `ui/src/components/pickers/pathPicker/fsDataSource.ts`
  - `ui/src/components/pickers/pathPicker/webdavDataSource.ts`
  - list/picker related unit tests
