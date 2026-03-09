## 1. Spec
- [x] 1.1 Draft proposal.md (why/what/impact/non-goals)
- [x] 1.2 Add web-ui spec delta (dashboard lazy-load + refresh cancellation)
- [x] 1.3 Add dev-workflow spec delta (warning-clean UI tests for touched stubs)
- [x] 1.4 Run openspec validate update-ui-dashboard-lazy-load-and-refresh-cancellation --strict

## 2. Implementation
- [x] 2.1 Lazy-load dashboard trend chart with fallback UI
- [x] 2.2 Cancel superseded refresh requests in agents/jobs stores while preserving latest-request-wins state semantics
- [x] 2.3 Add store regression tests for cancellation of stale refresh requests
- [x] 2.4 Fix AgentsView unit test input stub to avoid invalid native input prop warnings

## 3. Validation
- [x] 3.1 Run npm --prefix ui run lint:check
- [x] 3.2 Run npm --prefix ui run type-check
- [x] 3.3 Run npm --prefix ui run test -- --run
- [x] 3.4 Run bash scripts/ci.sh
