## 1. Spec
- [x] 1.1 Add `backend` + `web-ui` spec deltas for multi-value filters + desktop layout
- [x] 1.2 Run `openspec validate update-filter-bar-layout-and-multiselect --strict`

## 2. Backend
- [ ] 2.1 Support multi-value query params for incomplete cleanup list (`status`, `target_type`)
- [ ] 2.2 Support multi-value query params for notifications queue list (`status`, `channel`)
- [ ] 2.3 Update storage list/count queries to filter using `IN (...)`
- [ ] 2.4 Add/adjust tests for multi-value filtering

## 3. UI
- [ ] 3.1 Make enum filters multi-select in cleanup + notifications queue pages
- [ ] 3.2 Update filter bar layout: mobile full-width, desktop compact row

## 4. Verification
- [ ] 4.1 Run `cargo test`
- [ ] 4.2 Run `cd ui && npm test` and `npm run type-check`
