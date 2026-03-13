## 1. Spec
- [x] 1.1 Draft proposal/design/spec delta for filter and state consistency update
- [x] 1.2 Run `openspec validate update-list-filters-and-state-consistency --strict`

## 2. Shared Components
- [x] 2.1 Add shared filter field layout primitives usable by inline and stacked filter panels
- [x] 2.2 Add a shared list-state presenter for loading/base-empty/no-result states
- [x] 2.3 Add a shared filter summary row wrapper for results + active chips consistency

## 3. Page Migration
- [x] 3.1 Refactor `JobsFiltersPanel` to shared filter field primitives
- [x] 3.2 Migrate Jobs workspace list state and filter summary rendering to shared state components
- [x] 3.3 Migrate Agents list state and filter summary rendering to shared state components

## 4. Validation
- [x] 4.1 Add/update unit tests for shared filter/state primitives
- [x] 4.2 Run `npm run type-check --prefix ui`
- [x] 4.3 Run `npm test --prefix ui -- --run`
