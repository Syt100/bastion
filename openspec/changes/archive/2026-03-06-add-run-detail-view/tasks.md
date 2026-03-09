## 1. Spec
- [x] 1.1 Add `backend` spec delta for run read API + run-scoped operations view needs
- [x] 1.2 Add `web-ui` spec delta for node-scoped Run Detail page UX
- [x] 1.3 Run `openspec validate add-run-detail-view --strict`
- [x] 1.4 Commit the spec proposal (detailed message)

## 2. Backend API
- [x] 2.1 Add `GET /api/runs/{run_id}` run detail endpoint
- [x] 2.2 Add/update unit tests for run read endpoint (auth + not-found)

## 3. Web UI
- [x] 3.1 Add route `/n/:nodeId/runs/:runId` and new RunDetail view
- [x] 3.2 Show run overview + progress + events stream
- [x] 3.3 Show run-linked restore/verify operations sub-list
- [x] 3.4 Wire actions: start restore/verify from Run Detail, and open operation details
- [x] 3.5 Update runs list UI to deep-link into Run Detail
- [x] 3.6 Add/adjust router unit tests

## 4. Validation
- [x] 4.1 Run `pnpm -C ui test` (if present in repo)

## 5. Commits
- [x] 5.1 Commit implementation changes (detailed message with Modules/Tests)
