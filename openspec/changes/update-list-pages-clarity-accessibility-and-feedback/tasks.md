## 1. Spec
- [x] 1.1 Draft proposal.md (why/what/impact/non-goals)
- [x] 1.2 Draft design.md (interaction semantics + range labels + async feedback)
- [x] 1.3 Add `web-ui` spec delta for list clarity/accessibility/feedback behaviors
- [x] 1.4 Run `openspec validate update-list-pages-clarity-accessibility-and-feedback --strict`

## 2. Implementation (Web UI)
- [x] 2.1 Clarify Jobs results summary using visible-count vs filtered-total semantics
- [x] 2.2 Add Jobs mobile active-filter chips parity with desktop split/list behavior
- [x] 2.3 Refactor Jobs list row activation semantics to explicit row-main controls with isolated action boundaries
- [x] 2.4 Add unified pagination range labels (`start-end / total`) across Jobs/Agents/Notifications Queue
- [x] 2.5 Add Notifications Queue empty states for loading-empty / no-data / filtered-no-results
- [x] 2.6 Reduce Agents mobile list card noise via progressive disclosure of secondary metadata
- [x] 2.7 Add per-row async feedback/disable states for key list actions and standardize list search debounce timing

## 3. Tests / Validation
- [x] 3.1 Update/add Jobs view tests for row-main click semantics and row-action isolation
- [x] 3.2 Update/add Notifications Queue tests for empty-state and pagination-range behavior
- [x] 3.3 Update/add Agents view tests for pagination label and mobile disclosure behavior where practical
- [x] 3.4 Run `npm test --prefix ui`
- [x] 3.5 Run `scripts/ci.sh`
