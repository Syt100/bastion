## 1. Spec
- [x] 1.1 Draft proposal.md (why/what/impact/non-goals)
- [x] 1.2 Add web-ui spec delta (first paint, cancellation helper behavior, list pagination)
- [x] 1.3 Add dev-workflow spec delta (shared test stubs + route meta dedup conventions)
- [x] 1.4 Run openspec validate update-ui-first-paint-and-list-scale-ux --strict

## 2. Implementation
- [x] 2.1 Add dashboard first-paint skeletons and viewport-triggered trend chart mount
- [x] 2.2 Add Vite manual chunk splitting for heavy chart dependencies
- [x] 2.3 Introduce shared latest-request cancellation helper and apply to agents/jobs/dashboard stores
- [x] 2.4 Add/adjust store tests for overlap refresh cancellation and stale-response safety
- [x] 2.5 Add client-side pagination in jobs and agents list views
- [x] 2.6 Debounce agents filter-triggered refresh and parallelize safe mount refreshes
- [x] 2.7 Create shared Naive UI test-stub helpers and migrate repeated specs
- [x] 2.8 Refactor router duplicated meta blocks with helper factories

## 3. Validation
- [x] 3.1 Run npm --prefix ui run lint:check
- [x] 3.2 Run npm --prefix ui run type-check
- [x] 3.3 Run npm --prefix ui run test -- --run
- [x] 3.4 Run bash scripts/ci.sh
