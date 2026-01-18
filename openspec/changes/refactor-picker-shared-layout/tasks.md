## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for shared picker layout/composable refactor
- [x] 1.2 Run `openspec validate refactor-picker-shared-layout --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Web UI - Refactor
- [ ] 2.1 Create shared picker layout component/composable (no behavior change)
- [ ] 2.2 Migrate `FsPathPickerModal` to the shared layout
- [ ] 2.3 Migrate `RunEntriesPickerModal` to the shared layout
- [ ] 2.4 Update/add unit tests to cover both pickers after refactor

## 3. Validation
- [ ] 3.1 Run `npm test --prefix ui`
- [ ] 3.2 Run `npm run type-check --prefix ui`
- [ ] 3.3 Run `npm run build-only --prefix ui`

## 4. Commits
- [ ] 4.1 Commit the UI refactor (detailed message with Modules/Tests)
