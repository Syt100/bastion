## 1. Spec
- [x] 1.1 Draft proposal, tasks, design, and `web-ui` spec delta (no omissions)
- [x] 1.2 Run `openspec validate update-job-editor-modal-ux-and-maintainability --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Refactor - Mapping + Tests
- [x] 2.1 Extract mapping utilities (`JobDetail` → editor form, editor form → request)
- [x] 2.2 Add unit tests for mapping (legacy fields, encryption, targets, schedule normalization)
- [x] 2.3 Wire `JobEditorModal` to use mapping utilities for edit + preview + save
- [x] 2.4 Commit mapping refactor (detailed message)

## 3. Refactor - Step Components
- [ ] 3.1 Create step subcomponents (Basics/Source/Target/Security/Notifications/Review)
- [ ] 3.2 Move per-step UI into subcomponents and preserve behavior
- [ ] 3.3 Commit step component split (detailed message)

## 4. Validation + Field Focus
- [ ] 4.1 Implement per-step validation module (including cron format check)
- [ ] 4.2 Scroll/focus the first invalid field within the modal
- [ ] 4.3 Prevent skipping ahead to future steps when prior steps are invalid (backward always allowed)
- [ ] 4.4 Commit validation improvements (detailed message)

## 5. UX Enhancements
- [ ] 5.1 Move action bar into modal footer (persistent/sticky)
- [ ] 5.2 Add common cron presets picker
- [ ] 5.3 Add quick links to manage WebDAV secrets (node-scoped) and notification destinations (new tab)
- [ ] 5.4 Commit UX enhancements (detailed message)

## 6. Verification
- [ ] 6.1 Run `npm test --prefix ui`
- [ ] 6.2 Run `npm run type-check --prefix ui`
- [ ] 6.3 (Optional) Run `npm run lint --prefix ui`
