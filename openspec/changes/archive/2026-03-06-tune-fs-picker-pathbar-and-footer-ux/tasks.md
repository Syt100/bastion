## 1. Spec
- [x] 1.1 Draft proposal, tasks, and `web-ui` spec delta (no omissions)
- [x] 1.2 Run `openspec validate tune-fs-picker-pathbar-and-footer-ux --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. UI - Path Bar
- [x] 2.1 Focus the path input on open to avoid “Up” looking selected
- [x] 2.2 Move Up/Refresh actions into the path input prefix for a single-row mobile layout
- [x] 2.3 Tighten spacing between Up/Refresh and soften icon weight
- [x] 2.4 Remove the “Current path” label (use placeholder / aria label)
- [x] 2.5 Commit path bar improvements (detailed message)

## 3. UI - Mobile Footer Selected Count
- [x] 3.1 Keep desktop selected-count text on the left
- [x] 3.2 On mobile, show selected-count as a badge on “Add selected” (avoid separate text)
- [x] 3.3 Commit footer selected-count behavior (detailed message)

## 4. Verification
- [x] 4.1 Run `npm test --prefix ui`
- [x] 4.2 Run `npm run type-check --prefix ui`
