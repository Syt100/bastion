## 1. Spec
- [x] 1.1 Add `backend` + `web-ui` spec deltas for multi-value filters + desktop layout
- [x] 1.2 Run `openspec validate update-filter-bar-layout-and-multiselect --strict`

## 2. Backend
- [x] 2.1 Support multi-value query params for incomplete cleanup list (`status`, `target_type`)
- [x] 2.2 Support multi-value query params for notifications queue list (`status`, `channel`)
- [x] 2.3 Update storage list/count queries to filter using `IN (...)`
- [x] 2.4 Add/adjust tests for multi-value filtering

## 3. UI
- [x] 3.1 Make enum filters multi-select in cleanup + notifications queue pages
- [x] 3.2 Update filter bar layout: mobile full-width, desktop compact row

## 4. Verification
- [x] 4.1 Run `cargo test`
- [x] 4.2 Run `cd ui && npm test` and `npm run type-check`
