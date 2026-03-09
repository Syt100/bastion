## 1. Spec
- [x] 1.1 Draft proposal, tasks, and `web-ui` spec delta (no omissions)
- [x] 1.2 Run `openspec validate add-picker-path-breadcrumb-navigation --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. UI - Breadcrumb Path Bar
- [x] 2.1 Extend `PickerPathBarInput` with breadcrumb mode + edit mode (Windows-like click-to-jump)
- [x] 2.2 Implement long-path collapsing with `…` (desktop popover, mobile bottom drawer)
- [x] 2.3 Add `navigate` behavior (click segment/ellipsis item updates value and triggers refresh in parents)
- [x] 2.4 Update `FsPathPickerModal` to handle breadcrumb navigation
- [x] 2.5 Update `RunEntriesPickerModal` to handle breadcrumb navigation
- [x] 2.6 Commit UI changes (detailed message)

## 3. Verification
- [x] 3.1 Run `npm test --prefix ui`
- [x] 3.2 Run `npm run type-check --prefix ui`
