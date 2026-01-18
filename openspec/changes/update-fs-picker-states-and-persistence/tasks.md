## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for filesystem picker states + persistence
- [x] 1.2 Run `openspec validate update-fs-picker-states-and-persistence --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Web UI - FS Picker
- [ ] 2.1 Implement clearer empty state rendering (empty directory vs no matches)
- [ ] 2.2 Implement clearer error state rendering + contextual actions (retry/up/copy path/clear filters)
- [ ] 2.3 Persist per-node filter state in localStorage and restore on open
- [ ] 2.4 Add/adjust unit tests (as feasible)

## 3. Validation
- [ ] 3.1 Run `npm test --prefix ui`
- [ ] 3.2 Run `npm run type-check --prefix ui`
- [ ] 3.3 Run `npm run build-only --prefix ui`

## 4. Commits
- [ ] 4.1 Commit the UI changes (detailed message with Modules/Tests)
