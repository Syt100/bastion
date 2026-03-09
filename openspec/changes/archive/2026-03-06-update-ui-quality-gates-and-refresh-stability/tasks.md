## 1. Spec
- [x] 1.1 Draft proposal.md (why/what/impact/non-goals)
- [x] 1.2 Add dev-workflow spec delta (UI static checks in CI)
- [x] 1.3 Add web-ui spec delta (latest-refresh-wins behavior)
- [x] 1.4 Run openspec validate update-ui-quality-gates-and-refresh-stability --strict

## 2. Implementation
- [x] 2.1 Add CI-safe UI lint command and run UI lint/type-check in scripts/ci.sh
- [x] 2.2 Fix existing UI lint/type-check failures that block the new checks
- [x] 2.3 Implement latest-refresh-wins guards in agents and jobs stores
- [x] 2.4 Add/adjust store unit tests for stale response ordering and stale failure handling

## 3. Validation
- [x] 3.1 Run npm --prefix ui run lint:check
- [x] 3.2 Run npm --prefix ui run type-check
- [x] 3.3 Run npm --prefix ui run test -- --run
- [x] 3.4 Run bash scripts/ci.sh
