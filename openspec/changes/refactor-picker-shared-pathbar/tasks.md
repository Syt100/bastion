## 1. Spec
- [x] 1.1 Draft proposal, tasks, and `web-ui` spec delta (no omissions)
- [x] 1.2 Run `openspec validate refactor-picker-shared-pathbar --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. UI - Shared Component
- [ ] 2.1 Add `PickerPathBarInput` (prefix actions + softened icons + focus() API)
- [ ] 2.2 Refactor `FsPathPickerModal` to use the shared component (no behavior regressions)
- [ ] 2.3 Refactor `RunEntriesPickerModal` to use the shared component and match the same path bar style
- [ ] 2.4 Commit shared component refactor (detailed message)

## 3. Verification
- [ ] 3.1 Run `npm test --prefix ui`
- [ ] 3.2 Run `npm run type-check --prefix ui`
