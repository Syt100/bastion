## 1. Design + Types
- [ ] 1.1 Define `PickerDataSource` interface (list, normalize path, parent/join, error mapping)
- [ ] 1.2 Define capability model (supported filters/sorts/columns/pagination/selection modes)
- [ ] 1.3 Define a stable persistence key strategy (per data source + context such as node id)

## 2. Generic Picker Implementation
- [ ] 2.1 Implement generic picker state machine/composable (path, search, filters, sorting, pagination, selection)
- [ ] 2.2 Implement generic picker UI wrapper component (uses existing `Picker*` building blocks)
- [ ] 2.3 Ensure accessibility + keyboard shortcuts remain functional with the new abstraction

## 3. Filesystem Data Source Migration
- [ ] 3.1 Implement filesystem data source adapter that uses the existing `/api/nodes/{node}/fs/list` endpoint
- [ ] 3.2 Refactor `FsPathPickerModal` to use the generic picker while preserving public API and emitted events
- [ ] 3.3 Preserve existing persistence behavior (last dir + filters per node)

## 4. Tests + Regression Guard
- [ ] 4.1 Update/extend unit tests to cover the new abstraction boundaries
- [ ] 4.2 Verify no behavioral regressions in existing picker tests
