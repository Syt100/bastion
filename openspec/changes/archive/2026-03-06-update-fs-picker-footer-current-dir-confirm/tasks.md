## 1. Spec
- [x] 1.1 Draft proposal, tasks, and `web-ui` spec delta (no omissions)
- [x] 1.2 Run `openspec validate update-fs-picker-footer-current-dir-confirm --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. UI - Filesystem Picker Layout
- [x] 2.1 Replace Up/Refresh with icon-only actions near the current-path input (desktop + mobile)
- [x] 2.2 Move selected-count display to the modal footer (left side)
- [x] 2.3 Move “Select current directory” into the footer alongside “Add selected”
- [x] 2.4 Commit layout changes (detailed message)

## 3. UI - “Select Current Directory” Confirmation
- [x] 3.1 When selected items exist, show a confirmation prompt that lists current dir + selected items
- [x] 3.2 Primary/default action is “Only select current directory”
- [x] 3.3 Implement mobile-friendly confirmation UI (drawer on mobile, modal on desktop)
- [x] 3.4 Add/adjust unit tests for the confirm logic
- [x] 3.5 Commit confirmation behavior (detailed message)

## 4. Verification
- [x] 4.1 Run `npm test --prefix ui`
- [x] 4.2 Run `npm run type-check --prefix ui`
