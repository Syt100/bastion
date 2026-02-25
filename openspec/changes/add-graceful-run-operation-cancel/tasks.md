## 1. Spec
- [x] 1.1 Add `backend` spec delta for cancel lifecycle/status/API/race-safety behavior
- [x] 1.2 Add `hub-agent-protocol` spec delta for run/operation cancel dispatch semantics
- [x] 1.3 Add `web-ui` spec delta for cancel action and state presentation
- [x] 1.4 Run `openspec validate add-graceful-run-operation-cancel --strict`
- [x] 1.5 Commit the spec proposal (detailed message)

## 2. Phase 1: Backend cancellation foundation
- [x] 2.1 Add DB migration and model updates for `canceled` status + cancel-request metadata (runs/operations)
- [x] 2.2 Add repository helpers for idempotent cancel requests and CAS finalization guards
- [x] 2.3 Add authenticated cancel APIs: `POST /api/runs/{id}/cancel` and `POST /api/operations/{id}/cancel`
- [x] 2.4 Add scheduler/worker cancellation registry and immediate wakeup signaling for active local tasks
- [x] 2.5 Ensure queued tasks canceled before dispatch never start execution

## 3. Phase 2: Cooperative interruption and agent support
- [ ] 3.1 Add cancellation checkpoints in filesystem backup path (scan/package/upload boundaries)
- [ ] 3.2 Add cancellation checkpoints in restore/verify operation loops with cleanup-safe exits
- [x] 3.3 Extend Hub↔Agent protocol with cancel-task messages for run/operation execution
- [x] 3.4 Implement agent-side cancellation token handling and graceful terminal reporting
- [x] 3.5 Ensure late agent/local result writes cannot overwrite `canceled`

## 4. Web UI and docs
- [ ] 4.1 Update run/operation store status unions and cancel mutation APIs
- [ ] 4.2 Add UI states: cancel action availability, "canceling" interim display, and terminal canceled badges
- [ ] 4.3 Update user docs for cancel behavior, guarantees, and caveats (run + restore/verify)
- [ ] 4.4 If user-visible behavior changes, run `maintain-changelog-release` to update `CHANGELOG.md`

## 5. Tests and validation
- [x] 5.1 Add backend regression tests for queued-cancel, running-cancel, idempotency, and late-result race safety
- [ ] 5.2 Add protocol/agent integration coverage for cancel delivery and completion behavior
- [ ] 5.3 Add UI tests for cancel button state transitions and canceled rendering
- [ ] 5.4 Run `bash scripts/ci.sh`

## 6. Commits
- [x] 6.1 Commit implementation milestones with clear summaries and change points
- [ ] 6.2 Mark OpenSpec tasks complete and commit
