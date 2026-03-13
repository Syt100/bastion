## 1. Spec
- [x] 1.1 Draft proposal/spec delta for component-level modal shell consistency
- [x] 1.2 Run `openspec validate update-component-modal-shell-consistency --strict`

## 2. Implementation
- [x] 2.1 Migrate remaining reusable Jobs/Run dialog components from direct `NModal` to `AppModalShell`
- [x] 2.2 Preserve modal header/footer actions, body scrolling behavior, and emit semantics

## 3. Validation
- [x] 3.1 Run `npm run type-check --prefix ui`
- [x] 3.2 Run targeted UI tests for touched components
- [x] 3.3 Update `CHANGELOG.md` for user-visible consistency updates
