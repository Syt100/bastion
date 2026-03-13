## 1. Spec
- [x] 1.1 Draft proposal/spec delta for Agents modal-shell consistency update
- [x] 1.2 Run `openspec validate update-agents-modal-shell-consistency --strict`

## 2. Implementation
- [x] 2.1 Migrate Agents labels/bulk modals to shared `AppModalShell`
- [x] 2.2 Keep modal action semantics and existing form payload behavior unchanged

## 3. Validation
- [x] 3.1 Run `npm run type-check --prefix ui`
- [x] 3.2 Run `npm test --prefix ui -- src/views/AgentsView.spec.ts --run`
- [x] 3.3 Update `CHANGELOG.md` for user-visible consistency improvements
