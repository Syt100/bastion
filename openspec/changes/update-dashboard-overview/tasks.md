## 1. Spec
- [x] 1.1 Add `backend` + `web-ui` spec deltas for dashboard overview
- [x] 1.2 Run `openspec validate update-dashboard-overview --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Backend (API)
- [x] 2.1 Add `GET /api/dashboard/overview` route and handler
- [x] 2.2 Implement aggregation queries (stats, 7-day trend, recent runs)
- [x] 2.3 Add `bastion-http` tests for auth + response shape

## 3. Web UI (Dashboard)
- [ ] 3.1 Add a dashboard store/api client for the overview payload
- [ ] 3.2 Update Dashboard view to render:
  - KPI cards
  - 7-day trend chart
  - recent runs list/table with links
- [ ] 3.3 Update i18n keys (en-US + zh-CN) and any existing unit tests

## 4. Validation
- [x] 4.1 Run `cargo test -p bastion-http`
- [ ] 4.2 Run `npm test --prefix ui`
- [ ] 4.3 (Optional) Run `bash scripts/ci.sh` if changes touch multiple layers

## 5. Commits
- [ ] 5.1 Commit implementation changes (detailed message with Modules/Tests)
