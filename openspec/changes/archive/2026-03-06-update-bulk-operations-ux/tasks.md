## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for bulk operation auto-refresh + failed-only filter
- [x] 1.2 Run `openspec validate update-bulk-operations-ux --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation (Web UI)
- [x] 2.1 Add auto-refresh timer in the detail modal (only while running)
- [x] 2.2 Add UI filter to show only failed items
- [x] 2.3 Ensure timers are cleaned up on close/unmount

## 3. Tests / Validation
- [x] 3.1 Add/update unit tests for filtering logic
- [x] 3.2 Run `npm test --prefix ui`

## 4. Commits
- [x] 4.1 Commit implementation changes (detailed message)
