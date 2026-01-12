## 1. Spec
- [x] 1.1 Draft proposal, tasks, design, and spec deltas (`backend`, `web-ui`)
- [x] 1.2 Run `openspec validate add-job-schedule-timezone-and-builder --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Backend - Model & API
- [x] 2.1 Add Hub timezone config + expose via `/api/system`
- [x] 2.2 Add `jobs.schedule_timezone` column + update jobs repo CRUD
- [x] 2.3 Extend jobs CRUD API to accept/return `schedule_timezone`
- [x] 2.4 Include `schedule_timezone` in agent config snapshots
- [x] 2.5 Tighten cron validation (`5 fields` or `6 fields with seconds==0`) and validate IANA timezone strings
- [x] 2.6 Commit backend changes (detailed message)

## 3. Scheduler - Timezone Semantics
- [ ] 3.1 Hub scheduler: evaluate cron in `schedule_timezone` (gap=skip, fold=run once)
- [ ] 3.2 Agent offline scheduler: evaluate cron in `schedule_timezone` (same semantics)
- [ ] 3.3 Add targeted tests for timezone evaluation helpers
- [ ] 3.4 Commit scheduler changes (detailed message)

## 4. Web UI - Schedule Editor
- [ ] 4.1 Extend job types/mapping to include `schedule_timezone`
- [ ] 4.2 Add schedule editor UI: Manual / Simple / Cron mode, plus timezone selector (default Hub timezone)
- [ ] 4.3 Update validation + i18n strings
- [ ] 4.4 Commit UI changes (detailed message)

## 5. Verification
- [ ] 5.1 Run `cargo test`
- [ ] 5.2 Run `npm test --prefix ui`
- [ ] 5.3 Run `npm run type-check --prefix ui`
- [ ] 5.4 (Optional) Run `npm run lint --prefix ui`
