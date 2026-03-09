## 1. Spec
- [x] 1.1 Draft proposal.md (why/what/impact/non-goals)
- [x] 1.2 Draft design.md (filter unification + action discoverability)
- [x] 1.3 Add `web-ui` spec delta for unified Jobs filters and always-visible primary row actions
- [x] 1.4 Run `openspec validate update-jobs-filters-and-row-actions-discoverability --strict`

## 2. Implementation (Web UI)
- [x] 2.1 Extract Jobs filter state/computations into a dedicated composable
- [x] 2.2 Create shared Jobs filter panel component reused by split/list/mobile containers
- [x] 2.3 Replace duplicated filter rendering blocks in `JobsWorkspaceShellView.vue`
- [x] 2.4 Refactor job row actions to always-visible primary actions + overflow secondary actions
- [x] 2.5 Ensure row action clicks never trigger row navigation
- [x] 2.6 Add/update i18n strings and helper labels if needed

## 3. Tests / Validation
- [x] 3.1 Add/update tests ensuring filter behavior parity across split/list/mobile modes
- [x] 3.2 Add/update tests for active-filter chips/count and clear behavior
- [x] 3.3 Add/update tests for row-action discoverability and click-boundary behavior
- [x] 3.4 Run `npm test --prefix ui`
- [x] 3.5 Run `bash scripts/ci.sh`
