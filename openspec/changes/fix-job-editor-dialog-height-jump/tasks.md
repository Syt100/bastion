## 1. Spec
- [x] 1.1 Draft proposal, tasks, and `web-ui` spec delta (no omissions)
- [x] 1.2 Run `openspec validate fix-job-editor-dialog-height-jump --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. UI - Job Editor Modal
- [ ] 2.1 Make the desktop job editor modal use a stable shell height (keep current width)
- [ ] 2.2 Ensure footer actions do not shift vertically when switching steps (body scrolls instead)
- [ ] 2.3 Verify desktop + mobile breakpoints visually
- [ ] 2.4 Commit job editor modal layout fix (detailed message)

## 3. UI - Browser Modals (Filesystem / Archive)
- [ ] 3.1 Make the desktop filesystem path picker modal use a stable shell height + scrollable body
- [ ] 3.2 Summarize long selections (single-line ellipsis + `+N`) to avoid header reflow
- [ ] 3.3 Reserve inline warning/error space to avoid reflow
- [ ] 3.4 Verify desktop + mobile breakpoints visually
- [ ] 3.5 Commit browser modal layout fix (detailed message)

## 4. Verification
- [ ] 4.1 Run `npm test --prefix ui`
- [ ] 4.2 Run `npm run type-check --prefix ui`
