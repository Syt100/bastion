## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for Job Detail page + Jobs list action simplification
- [x] 1.2 Run `openspec validate add-job-detail-page --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation (Web UI)
- [x] 2.1 Add new route for `/n/:nodeId/jobs/:jobId`
- [x] 2.2 Implement Job Detail view:
  - Runs tab (list + link to Run Detail + restore/verify actions)
  - Snapshots tab (list + pin/delete + delete log)
  - Retention tab (view/edit + preview/apply)
  - Settings tab (edit/deploy/archive/delete)
- [x] 2.3 Simplify Jobs list actions and add “Open” entry point

## 3. Tests / Validation
- [x] 3.1 Add/update unit tests for routing + Jobs list changes
- [x] 3.2 Run `npm test --prefix ui`

## 4. Commits
- [x] 4.1 Commit implementation changes (detailed message)
