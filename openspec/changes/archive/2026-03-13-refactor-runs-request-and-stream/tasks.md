## 1. Spec
- [x] 1.1 Draft proposal/spec delta for run request race-safety and shared stream refactor
- [x] 1.2 Run `openspec validate refactor-runs-request-and-stream --strict`

## 2. Implementation
- [x] 2.1 Add request abort/latest guards for `JobRunsModal.open` and `jobs.listRuns`
- [x] 2.2 Add shared run-events stream controller and migrate `RunEventsModal` + `RunDetailPanel`
- [x] 2.3 Keep follow/unseen UI behavior unchanged in `RunEventsModal`

## 3. Validation
- [x] 3.1 Run `npm run type-check --prefix ui`
- [x] 3.2 Run targeted UI tests for run modal and stream logic
