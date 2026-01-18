## 1. Spec
- [x] 1.1 Draft proposal, tasks, and `web-ui` spec delta (no omissions)
- [x] 1.2 Run `openspec validate fix-picker-table-fill-height --strict`
- [x] 1.3 Commit spec proposal (detailed message)

## 2. UI - Picker Table Fill Height
- [x] 2.1 Refactor picker modal content to a flex column layout with a single scroll region
- [x] 2.2 Replace hard-coded table `max-height` with a ResizeObserver-derived pixel height
- [x] 2.3 Apply to both filesystem and restore-entry pickers
- [ ] 2.4 Commit UI change (detailed message)

## 3. Verification
- [x] 3.1 Run `npm test --prefix ui`
- [x] 3.2 Run `npm run type-check --prefix ui`
- [x] 3.3 Verify visually (desktop + mobile breakpoints)
