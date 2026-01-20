## 1. Spec
- [ ] 1.1 Add `backend` spec delta for run read API + run-scoped operations view needs
- [ ] 1.2 Add `web-ui` spec delta for node-scoped Run Detail page UX
- [ ] 1.3 Run `openspec validate add-run-detail-view --strict`
- [ ] 1.4 Commit the spec proposal (detailed message)

## 2. Backend API
- [ ] 2.1 Add `GET /api/runs/{run_id}` run detail endpoint
- [ ] 2.2 Add/update unit tests for run read endpoint (auth + not-found)

## 3. Web UI
- [ ] 3.1 Add route `/n/:nodeId/runs/:runId` and new RunDetail view
- [ ] 3.2 Show run overview + progress + events stream
- [ ] 3.3 Show run-linked restore/verify operations sub-list
- [ ] 3.4 Wire actions: start restore/verify from Run Detail, and open operation details
- [ ] 3.5 Update runs list UI to deep-link into Run Detail
- [ ] 3.6 Add/adjust router unit tests

## 4. Validation
- [ ] 4.1 Run `pnpm -C ui test` (if present in repo)

## 5. Commits
- [ ] 5.1 Commit implementation changes (detailed message with Modules/Tests)

