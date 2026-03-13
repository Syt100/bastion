## 1. Spec
- [x] 1.1 Draft proposal/spec delta for page-level modal shell consistency
- [x] 1.2 Run `openspec validate update-page-modal-shell-consistency --strict`

## 2. Implementation
- [x] 2.1 Migrate page-level dialogs in Job Snapshots / Job Workspace / Bulk Operations / Settings Storage / Notifications Destinations / Maintenance Cleanup to `AppModalShell`
- [x] 2.2 Preserve modal header/footer actions and mask-closable semantics where already defined

## 3. Validation
- [x] 3.1 Run `npm run type-check --prefix ui`
- [x] 3.2 Run targeted UI tests for touched views
- [x] 3.3 Update `CHANGELOG.md` for user-visible consistency updates
