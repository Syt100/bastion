## 1. Spec
- [ ] 1.1 Add `web-ui` spec delta for breakpoint/constants centralization, mobile header overflow menu, wizard step mobile UI, and Beta label
- [ ] 1.2 Run `openspec validate update-web-ui-mobile-polish --strict`

## 2. Web UI
- [ ] 2.1 Centralize breakpoint media queries and other repeated UI constants into shared modules
- [ ] 2.2 Fix `/undefined` navigation warning by guarding menu navigation inputs
- [ ] 2.3 Mobile header: replace inline action buttons with a “More” dropdown menu
- [ ] 2.4 Jobs wizard: use compact step indicator on mobile (x/total + progress bar), keep `NSteps` on desktop
- [ ] 2.5 Update tag label from MVP → Beta (desktop + mobile)
- [ ] 2.6 Update/extend unit tests as needed

## 3. Validation
- [ ] 3.1 Run `npm test` (ui)
- [ ] 3.2 Run `npm run build` (ui)

## 4. Commits
- [ ] 4.1 Commit the spec proposal (detailed message)
- [ ] 4.2 Commit the UI changes (detailed message with Modules/Tests)

