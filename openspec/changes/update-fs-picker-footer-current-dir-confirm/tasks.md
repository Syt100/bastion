## 1. Spec
- [x] 1.1 Draft proposal, tasks, and `web-ui` spec delta (no omissions)
- [x] 1.2 Run `openspec validate update-fs-picker-footer-current-dir-confirm --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. UI - Filesystem Picker Layout
- [ ] 2.1 Replace Up/Refresh with icon-only actions near the current-path input (desktop + mobile)
- [ ] 2.2 Move selected-count display to the modal footer (left side)
- [ ] 2.3 Move “Select current directory” into the footer alongside “Add selected”
- [ ] 2.4 Commit layout changes (detailed message)

## 3. UI - “Select Current Directory” Confirmation
- [ ] 3.1 When selected items exist, show a confirmation prompt that lists current dir + selected items
- [ ] 3.2 Primary/default action is “Only select current directory”
- [ ] 3.3 Implement mobile-friendly confirmation UI (drawer on mobile, modal on desktop)
- [ ] 3.4 Add/adjust unit tests for the confirm logic
- [ ] 3.5 Commit confirmation behavior (detailed message)

## 4. Verification
- [ ] 4.1 Run `npm test --prefix ui`
- [ ] 4.2 Run `npm run type-check --prefix ui`
