## 1. Spec
- [x] 1.1 Draft proposal and spec delta for picker open/refresh pipeline refactor
- [x] 1.2 Get proposal approval before implementation
- [x] 1.3 Run `openspec validate refactor-picker-open-refresh-pipeline --strict`

## 2. Implementation
- [x] 2.1 Refactor picker open flow to stage open transition and first refresh
- [x] 2.2 Update table-body measurement composable to remove redundant open-frame measurement churn
- [x] 2.3 Keep picker behavior parity for filters, shortcuts, pagination, and selected-path workflows

## 3. Validation
- [x] 3.1 Add/extend unit tests for picker open sequence and refresh triggering behavior
- [x] 3.2 Add/extend unit tests for picker table-height measurement lifecycle
- [x] 3.3 Run `npm run test:unit --prefix ui` for touched picker suites
- [x] 3.4 Run `npm run type-check --prefix ui`
