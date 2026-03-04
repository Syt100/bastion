## 1. Spec
- [x] 1.1 Draft proposal/design for list filter componentization and visibility consistency
- [x] 1.2 Add `web-ui` spec delta for shared list-filter model and cross-page parity
- [x] 1.3 Run `openspec validate refactor-list-filter-componentization-consistency --strict`

## 2. Shared Infrastructure
- [x] 2.1 Add reusable list-filter modeling utilities (`text/single/multi`, active count/chips/clear)
- [x] 2.2 Add reusable list filter select wrapper component for consistent toolbar field sizing/layout
- [x] 2.3 Add reusable active filter chips row wrapper component

## 3. Page Migrations
- [x] 3.1 Migrate Agents filters to shared model/wrappers and surface unified active-filter chips row
- [x] 3.2 Migrate Notifications Queue filters to shared model/wrappers and surface unified active-filter chips row
- [x] 3.3 Migrate Maintenance Cleanup filters to shared model/wrappers and surface unified active-filter chips row
- [x] 3.4 Migrate Job Snapshots filters to shared model/wrappers and surface unified active-filter chips row

## 4. Tests / Validation
- [x] 4.1 Add/update tests covering shared filter chips visibility and clear behavior on migrated pages
- [x] 4.2 Run `npm test --prefix ui`
- [x] 4.3 Run `scripts/ci.sh`
- [x] 4.4 Update `CHANGELOG.md` via `maintain-changelog-release` workflow for user-visible consistency improvements
