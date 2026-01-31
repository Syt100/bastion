## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for Job Detail actions toolbar
- [x] 1.2 Run `openspec validate update-job-detail-actions-toolbar --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation (Web UI)
- [x] 2.1 Add a Job Detail toolbar with common job-level actions (run/edit/deploy/archive/delete)
- [x] 2.2 Ensure archive/delete actions use a confirmation modal (and keep cascade option for archive)
- [x] 2.3 Update Job Detail Settings tab to avoid duplicating the job-level actions

## 3. Tests / Validation
- [x] 3.1 Add/update unit tests for Job Detail toolbar behavior
- [x] 3.2 Run `npm test --prefix ui`

## 4. Commits
- [x] 4.1 Commit implementation changes (detailed message with Modules/Tests)
